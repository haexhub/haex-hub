// src-tauri/src/extension/permissions/validator.rs

use crate::database::core::{extract_table_names_from_sql, parse_single_statement};
use crate::database::error::DatabaseError;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::Action;
use crate::AppState;
use sqlparser::ast::{Statement, TableFactor, TableObject};
use tauri::State;

pub struct SqlPermissionValidator;

impl SqlPermissionValidator {
    /// Validiert ein SQL-Statement gegen die Permissions einer Extension
    pub async fn validate_sql(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        sql: &str,
    ) -> Result<(), ExtensionError> {
        let statement = parse_single_statement(sql).map_err(|e| DatabaseError::ParseError {
            reason: e.to_string(),
            sql: sql.to_string(),
        })?;

        match &statement {
            Statement::Query(_) => {
                Self::validate_read_statement(app_state, extension_id, sql).await
            }
            Statement::Insert(_) | Statement::Update { .. } | Statement::Delete(_) => {
                Self::validate_write_statement(app_state, extension_id, &statement).await
            }
            Statement::CreateTable(_) => {
                Self::validate_create_statement(app_state, extension_id, &statement).await
            }
            Statement::AlterTable { .. } | Statement::Drop { .. } => {
                Self::validate_schema_statement(app_state, extension_id, &statement).await
            }
            _ => Err(ExtensionError::ValidationError {
                reason: format!("Statement type not allowed: {}", sql),
            }),
        }
    }

    /// Validiert READ-Operationen (SELECT)
    async fn validate_read_statement(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        sql: &str,
    ) -> Result<(), ExtensionError> {
        let tables = extract_table_names_from_sql(sql)?;

        for table_name in tables {
            PermissionManager::check_database_permission(
                app_state,
                extension_id,
                Action::Read,
                &table_name,
            )
            .await?;
        }

        Ok(())
    }

    /// Validiert WRITE-Operationen (INSERT, UPDATE, DELETE)
    async fn validate_write_statement(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        statement: &Statement,
    ) -> Result<(), ExtensionError> {
        let table_names = Self::extract_table_names_from_statement(statement)?;

        for table_name in table_names {
            PermissionManager::check_database_permission(
                app_state,
                extension_id,
                Action::Write,
                &table_name,
            )
            .await?;
        }

        Ok(())
    }

    /// Validiert CREATE TABLE
    async fn validate_create_statement(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        statement: &Statement,
    ) -> Result<(), ExtensionError> {
        if let Statement::CreateTable(create_table) = statement {
            let table_name = create_table.name.to_string();

            // Prüfe ob Extension überhaupt CREATE-Rechte hat (z.B. auf "*")
            PermissionManager::check_database_permission(
                app_state,
                extension_id,
                Action::Write,
                &table_name,
            )
            .await?;
        }

        Ok(())
    }

    /// Validiert Schema-Änderungen (ALTER, DROP)
    async fn validate_schema_statement(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        statement: &Statement,
    ) -> Result<(), ExtensionError> {
        let table_names = Self::extract_table_names_from_statement(statement)?;

        for table_name in table_names {
            // ALTER/DROP benötigen WRITE-Rechte
            PermissionManager::check_database_permission(
                app_state,
                extension_id,
                Action::Write,
                &table_name,
            )
            .await?;
        }

        Ok(())
    }

    /// Extrahiert alle Tabellennamen aus einem Statement
    fn extract_table_names_from_statement(
        statement: &Statement,
    ) -> Result<Vec<String>, ExtensionError> {
        match statement {
            Statement::Insert(insert) => Ok(vec![Self::extract_table_name_from_insert(insert)?]),
            Statement::Update { table, .. } => {
                Ok(vec![Self::extract_table_name_from_table_factor(
                    &table.relation,
                )?])
            }
            Statement::Delete(delete) => Ok(vec![Self::extract_table_name_from_delete(delete)?]),
            Statement::CreateTable(create_table) => Ok(vec![create_table.name.to_string()]),
            Statement::AlterTable { name, .. } => Ok(vec![name.to_string()]),
            Statement::Drop { names, .. } => {
                Ok(names.iter().map(|name| name.to_string()).collect())
            }
            _ => Ok(vec![]),
        }
    }

    /// Extrahiert Tabellenname aus INSERT
    fn extract_table_name_from_insert(
        insert: &sqlparser::ast::Insert,
    ) -> Result<String, ExtensionError> {
        match &insert.table {
            TableObject::TableName(name) => Ok(name.to_string()),
            _ => Err(DatabaseError::NoTableError {
                sql: insert.to_string(),
            }
            .into()),
        }
    }

    /// Extrahiert Tabellenname aus TableFactor
    fn extract_table_name_from_table_factor(
        table_factor: &TableFactor,
    ) -> Result<String, ExtensionError> {
        match table_factor {
            TableFactor::Table { name, .. } => Ok(name.to_string()),
            _ => Err(DatabaseError::StatementError {
                reason: "Complex table references not supported".to_string(),
            }
            .into()),
        }
    }

    /// Extrahiert Tabellenname aus DELETE
    fn extract_table_name_from_delete(
        delete: &sqlparser::ast::Delete,
    ) -> Result<String, ExtensionError> {
        use sqlparser::ast::FromTable;

        let table_name = match &delete.from {
            FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => {
                if !tables.is_empty() {
                    Self::extract_table_name_from_table_factor(&tables[0].relation)?
                } else if !delete.tables.is_empty() {
                    delete.tables[0].to_string()
                } else {
                    return Err(DatabaseError::NoTableError {
                        sql: delete.to_string(),
                    }
                    .into());
                }
            }
        };

        Ok(table_name)
    }
}
