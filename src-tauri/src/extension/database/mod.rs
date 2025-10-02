// src-tauri/src/extension/database/mod.rs

pub mod executor;
use crate::crdt::transformer::CrdtTransformer;
use crate::crdt::trigger;
use crate::database::core::{parse_sql_statements, with_connection, ValueConverter};
use crate::database::error::DatabaseError;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::validator::SqlPermissionValidator;
use crate::AppState;

use rusqlite::params_from_iter;
use rusqlite::types::Value as SqlValue;
use rusqlite::Transaction;
use serde_json::json;
use serde_json::Value as JsonValue;
use sqlparser::ast::{Statement, TableFactor, TableObject};
use std::collections::HashSet;
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
    extension_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, ExtensionError> {
    // Permission check
    SqlPermissionValidator::validate_sql(&state, &extension_id, sql).await?;

    // Parameter validation
    validate_params(sql, &params)?;

    // SQL parsing
    let mut ast_vec = parse_sql_statements(sql)?;

    // Database operation
    with_connection(&state.db, |conn| {
        let tx = conn.transaction().map_err(DatabaseError::from)?;

        let transformer = CrdtTransformer::new();
        let executor = StatementExecutor::new(&tx);

        // Generate HLC timestamp
        let hlc_timestamp = state
            .hlc
            .lock()
            .unwrap()
            .new_timestamp_and_persist(&tx)
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
        let sql_values = ValueConverter::convert_params(&params)?;

        // Execute statements
        for statement in ast_vec {
            executor.execute_statement_with_params(&statement, &sql_values)?;

            if let Statement::CreateTable(create_table_details) = statement {
                let table_name_str = create_table_details.name.to_string();
                println!(
                    "Table '{}' created by extension, setting up CRDT triggers...",
                    table_name_str
                );
                trigger::setup_triggers_for_table(&tx, &table_name_str, false)?;
                println!(
                    "Triggers for table '{}' successfully created.",
                    table_name_str
                );
            }
        }

        // Commit transaction
        tx.commit().map_err(DatabaseError::from)?;

        Ok(modified_schema_tables.into_iter().collect())
    })
    .map_err(ExtensionError::from)
}

#[tauri::command]
pub async fn extension_sql_select(
    sql: &str,
    params: Vec<JsonValue>,
    extension_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<JsonValue>, ExtensionError> {
    // Permission check
    SqlPermissionValidator::validate_sql(&state, &extension_id, sql).await?;

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

    // Database operation
    with_connection(&state.db, |conn| {
        let sql_params = ValueConverter::convert_params(&params)?;
        let transformer = CrdtTransformer::new();

        // Use the last statement for result set
        let last_statement = ast_vec.pop().unwrap();
        let mut stmt_to_execute = last_statement;

        // Transform the statement
        transformer.transform_select_statement(&mut stmt_to_execute)?;
        let transformed_sql = stmt_to_execute.to_string();

        // Prepare and execute query
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
                row_to_json_value(row, &column_names)
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
    })
    .map_err(ExtensionError::from)
}

/// Konvertiert eine SQLite-Zeile zu JSON
fn row_to_json_value(
    row: &rusqlite::Row,
    columns: &[String],
) -> Result<JsonValue, rusqlite::Error> {
    let mut map = serde_json::Map::new();
    for (i, col_name) in columns.iter().enumerate() {
        let value = row.get::<usize, rusqlite::types::Value>(i)?;
        let json_value = match value {
            rusqlite::types::Value::Null => JsonValue::Null,
            rusqlite::types::Value::Integer(i) => json!(i),
            rusqlite::types::Value::Real(f) => json!(f),
            rusqlite::types::Value::Text(s) => json!(s),
            rusqlite::types::Value::Blob(blob) => json!(blob.to_vec()),
        };
        map.insert(col_name.clone(), json_value);
    }
    Ok(JsonValue::Object(map))
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

/// Kürzt SQL für Fehlermeldungen
/* fn truncate_sql(sql: &str, max_length: usize) -> String {
    if sql.len() <= max_length {
        sql.to_string()
    } else {
        format!("{}...", &sql[..max_length])
    }
} */

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

    /* #[test]
    fn test_truncate_sql() {
        let sql = "SELECT * FROM very_long_table_name";
        assert_eq!(truncate_sql(sql, 10), "SELECT * F...");
        assert_eq!(truncate_sql(sql, 50), sql);
    } */

    #[test]
    fn test_validate_params() {
        let params = vec![json!(1), json!("test")];

        assert!(validate_params("SELECT * FROM users WHERE id = ? AND name = ?", &params).is_ok());
        assert!(validate_params("SELECT * FROM users WHERE id = ?", &params).is_err());
        assert!(validate_params("SELECT * FROM users", &params).is_err());
    }
}
