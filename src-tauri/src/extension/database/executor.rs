// src-tauri/src/extension/database/executor.rs (neu)

use crate::crdt::hlc::HlcService;
use crate::crdt::transformer::CrdtTransformer;
use crate::crdt::trigger;
use crate::database::core::{parse_sql_statements, ValueConverter};
use crate::database::error::DatabaseError;
use rusqlite::{params_from_iter, Params, Transaction};
use serde_json::Value as JsonValue;
use sqlparser::ast::Statement;
use std::collections::HashSet;

/// SQL-Executor OHNE Berechtigungsprüfung - für interne Nutzung
pub struct SqlExecutor;

impl SqlExecutor {
    pub fn execute_internal_typed<P>(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: P, // Akzeptiert jetzt alles, was rusqlite als Parameter versteht
    ) -> Result<HashSet<String>, DatabaseError>
    where
        P: Params,
    {
        let mut ast_vec = parse_sql_statements(sql)?;

        // Wir stellen sicher, dass wir nur EIN Statement verarbeiten. Das ist sicherer.
        if ast_vec.len() != 1 {
            return Err(DatabaseError::ExecutionError {
                sql: sql.to_string(),
                reason: "execute_internal_typed sollte nur ein einzelnes SQL-Statement erhalten"
                    .to_string(),
                table: None,
            });
        }
        // Wir nehmen das einzige Statement aus dem Vektor.
        let mut statement = ast_vec.pop().unwrap();

        let transformer = CrdtTransformer::new();
        let hlc_timestamp =
            hlc_service
                .new_timestamp_and_persist(tx)
                .map_err(|e| DatabaseError::HlcError {
                    reason: e.to_string(),
                })?;

        let mut modified_schema_tables = HashSet::new();
        if let Some(table_name) =
            transformer.transform_execute_statement(&mut statement, &hlc_timestamp)?
        {
            modified_schema_tables.insert(table_name);
        }

        // Führe das transformierte Statement aus.
        // `params` wird jetzt nur noch einmal hierher bewegt, was korrekt ist.
        let sql_str = statement.to_string();
        tx.execute(&sql_str, params)
            .map_err(|e| DatabaseError::ExecutionError {
                sql: sql_str.clone(),
                table: None,
                reason: e.to_string(),
            })?;

        // Die Trigger-Logik für CREATE TABLE bleibt erhalten.
        if let Statement::CreateTable(create_table_details) = statement {
            let table_name_str = create_table_details.name.to_string();
            trigger::setup_triggers_for_table(tx, &table_name_str, false)?;
        }

        Ok(modified_schema_tables)
    }
    /// Führt SQL aus (mit CRDT-Transformation) - OHNE Permission-Check
    pub fn execute_internal(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<HashSet<String>, DatabaseError> {
        // Parameter validation
        let total_placeholders = sql.matches('?').count();
        if total_placeholders != params.len() {
            return Err(DatabaseError::ParameterMismatchError {
                expected: total_placeholders,
                provided: params.len(),
                sql: sql.to_string(),
            });
        }

        // SQL parsing
        let mut ast_vec = parse_sql_statements(sql)?;

        let transformer = CrdtTransformer::new();

        // Generate HLC timestamp
        let hlc_timestamp =
            hlc_service
                .new_timestamp_and_persist(tx)
                .map_err(|e| DatabaseError::HlcError {
                    reason: e.to_string(),
                })?;

        // Transform statements
        let mut modified_schema_tables = HashSet::new();
        for statement in &mut ast_vec {
            if let Some(table_name) =
                transformer.transform_execute_statement(statement, &hlc_timestamp)?
            {
                modified_schema_tables.insert(table_name);
            }
        }

        // Convert parameters
        let sql_values = ValueConverter::convert_params(params)?;

        // Execute statements
        for statement in ast_vec {
            let sql_str = statement.to_string();

            tx.execute(&sql_str, params_from_iter(sql_values.iter()))
                .map_err(|e| DatabaseError::ExecutionError {
                    sql: sql_str.clone(),
                    table: None,
                    reason: e.to_string(),
                })?;

            if let Statement::CreateTable(create_table_details) = statement {
                let table_name_str = create_table_details.name.to_string();
                trigger::setup_triggers_for_table(tx, &table_name_str, false)?;
            }
        }

        Ok(modified_schema_tables)
    }

    /// Führt SELECT aus (mit CRDT-Transformation) - OHNE Permission-Check
    pub fn select_internal(
        conn: &rusqlite::Connection,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<Vec<JsonValue>, DatabaseError> {
        // Parameter validation
        let total_placeholders = sql.matches('?').count();
        if total_placeholders != params.len() {
            return Err(DatabaseError::ParameterMismatchError {
                expected: total_placeholders,
                provided: params.len(),
                sql: sql.to_string(),
            });
        }

        let mut ast_vec = parse_sql_statements(sql)?;

        if ast_vec.is_empty() {
            return Ok(vec![]);
        }

        // Validate that all statements are queries
        for stmt in &ast_vec {
            if !matches!(stmt, Statement::Query(_)) {
                return Err(DatabaseError::ExecutionError {
                    sql: sql.to_string(),
                    reason: "Only SELECT statements are allowed".to_string(),
                    table: None,
                });
            }
        }

        let sql_params = ValueConverter::convert_params(params)?;
        let transformer = CrdtTransformer::new();

        let last_statement = ast_vec.pop().unwrap();
        let mut stmt_to_execute = last_statement;

        transformer.transform_select_statement(&mut stmt_to_execute)?;
        let transformed_sql = stmt_to_execute.to_string();

        let mut prepared_stmt =
            conn.prepare(&transformed_sql)
                .map_err(|e| DatabaseError::ExecutionError {
                    sql: transformed_sql.clone(),
                    reason: e.to_string(),
                    table: None,
                })?;

        let column_names: Vec<String> = prepared_stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let rows = prepared_stmt
            .query_map(params_from_iter(sql_params.iter()), |row| {
                crate::extension::database::row_to_json_value(row, &column_names)
            })
            .map_err(|e| DatabaseError::QueryError {
                reason: e.to_string(),
            })?;

        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result.map_err(|e| DatabaseError::RowProcessingError {
                reason: e.to_string(),
            })?);
        }

        Ok(results)
    }
}
