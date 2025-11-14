// src-tauri/src/extension/database/mod.rs

pub mod executor;
use crate::crdt::transformer::CrdtTransformer;
use crate::crdt::trigger;
use crate::database::core::{parse_sql_statements, with_connection, ValueConverter};
use crate::database::error::DatabaseError;
use crate::extension::database::executor::SqlExecutor;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::validator::SqlPermissionValidator;
use crate::AppState;

use rusqlite::params_from_iter;
use rusqlite::types::Value as SqlValue;
use rusqlite::Transaction;
use serde_json::Value as JsonValue;
use sqlparser::ast::{Statement, TableFactor, TableObject};
use tauri::State;

/// Führt Statements mit korrekter Parameter-Bindung aus
pub struct StatementExecutor<'a> {
    transaction: &'a Transaction<'a>,
}

impl<'a> StatementExecutor<'a> {
    fn new(transaction: &'a Transaction<'a>) -> Self {
        Self { transaction }
    }

    /// Führt ein einzelnes Statement mit Parametern aus
    fn execute_statement_with_params(
        &self,
        statement: &Statement,
        params: &[SqlValue],
    ) -> Result<(), DatabaseError> {
        let sql = statement.to_string();
        let expected_params = count_sql_placeholders(&sql);

        if expected_params != params.len() {
            return Err(DatabaseError::ParameterMismatchError {
                expected: expected_params,
                provided: params.len(),
                sql,
            });
        }

        self.transaction
            .execute(&sql, params_from_iter(params.iter()))
            .map_err(|e| DatabaseError::ExecutionError {
                sql,
                table: Some(
                    self.extract_table_name_from_statement(statement)
                        .unwrap_or_else(|| "unknown".to_string()),
                ),
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Extrahiert den Tabellennamen aus einem Statement für bessere Fehlermeldungen
    fn extract_table_name_from_statement(&self, statement: &Statement) -> Option<String> {
        match statement {
            Statement::Insert(insert) => {
                if let TableObject::TableName(name) = &insert.table {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            Statement::Update { table, .. } => {
                if let TableFactor::Table { name, .. } = &table.relation {
                    Some(name.to_string())
                } else {
                    None
                }
            }
            Statement::Delete(delete) => {
                // Verbessertes Extrahieren für DELETE
                use sqlparser::ast::FromTable;
                match &delete.from {
                    FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => {
                        if !tables.is_empty() {
                            if let TableFactor::Table { name, .. } = &tables[0].relation {
                                Some(name.to_string())
                            } else {
                                None
                            }
                        } else if !delete.tables.is_empty() {
                            Some(delete.tables[0].to_string())
                        } else {
                            None
                        }
                    }
                }
            }
            Statement::CreateTable(create) => Some(create.name.to_string()),
            Statement::AlterTable { name, .. } => Some(name.to_string()),
            Statement::Drop { names, .. } => names.first().map(|name| name.to_string()),
            _ => None,
        }
    }
}

#[tauri::command]
pub async fn extension_sql_execute(
    sql: &str,
    params: Vec<JsonValue>,
    public_key: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, ExtensionError> {
    // Get extension to retrieve its ID
    let extension = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
        .ok_or_else(|| ExtensionError::NotFound {
            public_key: public_key.clone(),
            name: name.clone(),
        })?;

    // Permission check
    SqlPermissionValidator::validate_sql(&state, &extension.id, sql).await?;

    // Parameter validation
    validate_params(sql, &params)?;

    // SQL parsing
    let mut ast_vec = parse_sql_statements(sql)?;

    if ast_vec.len() != 1 {
        return Err(ExtensionError::Database {
            source: DatabaseError::ExecutionError {
                sql: sql.to_string(),
                reason: "extension_sql_execute should only receive a single SQL statement"
                    .to_string(),
                table: None,
            },
        });
    }

    let mut statement = ast_vec.pop().unwrap();

    // If this is a SELECT statement, delegate to extension_sql_select
    if matches!(statement, Statement::Query(_)) {
        return extension_sql_select(sql, params, public_key, name, state).await;
    }

    // Check if statement has RETURNING clause
    let has_returning = crate::database::core::statement_has_returning(&statement);

    // Database operation
    with_connection(&state.db, |conn| {
        let tx = conn.transaction().map_err(DatabaseError::from)?;

        let transformer = CrdtTransformer::new();

        // Get HLC service reference
        let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
            reason: "Failed to lock HLC service".to_string(),
        })?;

        // Generate HLC timestamp
        let hlc_timestamp =
            hlc_service
                .new_timestamp_and_persist(&tx)
                .map_err(|e| DatabaseError::HlcError {
                    reason: e.to_string(),
                })?;

        // Transform statement
        transformer.transform_execute_statement(&mut statement, &hlc_timestamp)?;

        // Convert parameters to references
        let sql_values = ValueConverter::convert_params(&params)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = sql_values
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        let result = if has_returning {
            // Use query_internal for statements with RETURNING
            let (_, rows) = SqlExecutor::query_internal_typed(
                &tx,
                &hlc_service,
                &statement.to_string(),
                &param_refs,
            )?;
            rows
        } else {
            // Use execute_internal for statements without RETURNING
            SqlExecutor::execute_internal_typed(
                &tx,
                &hlc_service,
                &statement.to_string(),
                &param_refs,
            )?;
            vec![]
        };

        // Handle CREATE TABLE trigger setup
        if let Statement::CreateTable(ref create_table_details) = statement {
            // Extract table name and remove quotes (both " and `)
            let raw_name = create_table_details.name.to_string();
            println!("DEBUG: Raw table name from AST: {raw_name:?}");
            println!(
                "DEBUG: Raw table name chars: {:?}",
                raw_name.chars().collect::<Vec<_>>()
            );

            let table_name_str = raw_name.trim_matches('"').trim_matches('`').to_string();

            println!("DEBUG: Cleaned table name: {table_name_str:?}");
            println!(
                "DEBUG: Cleaned table name chars: {:?}",
                table_name_str.chars().collect::<Vec<_>>()
            );

            println!("Table '{table_name_str}' created by extension, setting up CRDT triggers...");
            trigger::setup_triggers_for_table(&tx, &table_name_str, false)?;
            println!("Triggers for table '{table_name_str}' successfully created.");
        }

        // Commit transaction
        tx.commit().map_err(DatabaseError::from)?;

        Ok(result)
    })
    .map_err(ExtensionError::from)
}

#[tauri::command]
pub async fn extension_sql_select(
    sql: &str,
    params: Vec<JsonValue>,
    public_key: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, ExtensionError> {
    // Get extension to retrieve its ID
    let extension = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
        .ok_or_else(|| ExtensionError::NotFound {
            public_key: public_key.clone(),
            name: name.clone(),
        })?;

    // Permission check
    SqlPermissionValidator::validate_sql(&state, &extension.id, sql).await?;

    // Parameter validation
    validate_params(sql, &params)?;

    // SQL parsing
    let mut ast_vec = parse_sql_statements(sql)?;

    if ast_vec.is_empty() {
        return Ok(vec![]);
    }

    // Validate that all statements are queries
    for stmt in &ast_vec {
        if !matches!(stmt, Statement::Query(_)) {
            return Err(ExtensionError::Database {
                source: DatabaseError::ExecutionError {
                    sql: sql.to_string(),
                    reason: "Only SELECT statements are allowed in extension_sql_select"
                        .to_string(),
                    table: None,
                },
            });
        }
    }

    // Database operation - return Vec<Vec<JsonValue>> like sql_select_with_crdt
    with_connection(&state.db, |conn| {
        let sql_params = ValueConverter::convert_params(&params)?;
        let stmt_to_execute = ast_vec.pop().unwrap();
        let transformed_sql = stmt_to_execute.to_string();

        // Prepare and execute query
        let mut prepared_stmt =
            conn.prepare(&transformed_sql)
                .map_err(|e| DatabaseError::ExecutionError {
                    sql: transformed_sql.clone(),
                    reason: e.to_string(),
                    table: None,
                })?;

        let num_columns = prepared_stmt.column_count();
        let mut rows = prepared_stmt
            .query(params_from_iter(sql_params.iter()))
            .map_err(|e| DatabaseError::QueryError {
                reason: e.to_string(),
            })?;

        let mut result_vec: Vec<Vec<JsonValue>> = Vec::new();

        while let Some(row) = rows.next().map_err(|e| DatabaseError::QueryError {
            reason: e.to_string(),
        })? {
            let mut row_values: Vec<JsonValue> = Vec::new();
            for i in 0..num_columns {
                let value_ref = row.get_ref(i).map_err(|e| DatabaseError::QueryError {
                    reason: e.to_string(),
                })?;
                let json_value = crate::database::core::convert_value_ref_to_json(value_ref)?;
                row_values.push(json_value);
            }
            result_vec.push(row_values);
        }

        Ok(result_vec)
    })
    .map_err(ExtensionError::from)
}

/// Validiert Parameter gegen SQL-Platzhalter
fn validate_params(sql: &str, params: &[JsonValue]) -> Result<(), DatabaseError> {
    let total_placeholders = count_sql_placeholders(sql);

    if total_placeholders != params.len() {
        return Err(DatabaseError::ParameterMismatchError {
            expected: total_placeholders,
            provided: params.len(),
            sql: sql.to_string(),
        });
    }

    Ok(())
}

/// Zählt SQL-Platzhalter (verbesserte Version)
fn count_sql_placeholders(sql: &str) -> usize {
    sql.matches('?').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_sql_placeholders() {
        assert_eq!(
            count_sql_placeholders("SELECT * FROM users WHERE id = ?"),
            1
        );
        assert_eq!(
            count_sql_placeholders("SELECT * FROM users WHERE id = ? AND name = ?"),
            2
        );
        assert_eq!(count_sql_placeholders("SELECT * FROM users"), 0);
    }
}
