// src-tauri/src/extension/database/permissions.rs

use crate::database::core::{
    extract_table_names_from_sql, parse_single_statement, with_connection,
};
use crate::database::error::DatabaseError;
use crate::database::DbConnection;
use crate::extension::error::ExtensionError;

use serde::{Deserialize, Serialize};
use sqlparser::ast::{Statement, TableFactor, TableObject};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbExtensionPermission {
    pub id: String,
    pub extension_id: String,
    pub resource: String,
    pub operation: String,
}

/// Prüft Leseberechtigungen für eine Extension
pub async fn check_read_permission(
    connection: &DbConnection,
    extension_id: &str,
    sql: &str,
) -> Result<(), ExtensionError> {
    let statement = parse_single_statement(sql).map_err(|e| DatabaseError::ParseError {
        reason: e.to_string(),
        sql: sql.to_string(),
    })?;

    match statement {
        Statement::Query(query) => {
            let tables = extract_table_names_from_sql(&query.to_string())?;
            check_table_permissions(connection, extension_id, &tables, "read").await
        }
        _ => Err(DatabaseError::UnsupportedStatement {
            reason: "Only SELECT statements are allowed for read operations".to_string(),
            sql: sql.to_string(),
        }
        .into()),
    }
}

/// Prüft Schreibberechtigungen für eine Extension
pub async fn check_write_permission(
    connection: &DbConnection,
    extension_id: &str,
    sql: &str,
) -> Result<(), ExtensionError> {
    let statement = parse_single_statement(sql).map_err(|e| DatabaseError::ParseError {
        reason: e.to_string(),
        sql: sql.to_string(),
    })?;

    match statement {
        Statement::Insert(insert) => {
            let table_name = extract_table_name_from_insert(&insert)?;
            check_single_table_permission(connection, extension_id, &table_name, "write").await
        }
        Statement::Update { table, .. } => {
            let table_name = extract_table_name_from_table_factor(&table.relation)?;
            check_single_table_permission(connection, extension_id, &table_name, "write").await
        }
        Statement::Delete(delete) => {
            // DELETE wird durch CRDT-Transform zu UPDATE mit tombstone = 1
            let table_name = extract_table_name_from_delete(&delete)?;
            check_single_table_permission(connection, extension_id, &table_name, "write").await
        }
        Statement::CreateTable(create_table) => {
            let table_name = create_table.name.to_string();
            check_single_table_permission(connection, extension_id, &table_name, "create").await
        }
        Statement::AlterTable { name, .. } => {
            let table_name = name.to_string();
            check_single_table_permission(connection, extension_id, &table_name, "alter").await
        }
        Statement::Drop { names, .. } => {
            // Für DROP können mehrere Tabellen angegeben sein
            let table_names: Vec<String> = names.iter().map(|name| name.to_string()).collect();
            check_table_permissions(connection, extension_id, &table_names, "drop").await
        }
        _ => Err(DatabaseError::UnsupportedStatement {
            reason: "SQL Statement is not allowed".to_string(),
            sql: sql.to_string(),
        }
        .into()),
    }
}

/// Extrahiert Tabellenname aus INSERT-Statement
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

/// Extrahiert Tabellenname aus DELETE-Statement
fn extract_table_name_from_delete(
    delete: &sqlparser::ast::Delete,
) -> Result<String, ExtensionError> {
    use sqlparser::ast::FromTable;

    let table_name = match &delete.from {
        FromTable::WithFromKeyword(tables) | FromTable::WithoutKeyword(tables) => {
            if !tables.is_empty() {
                extract_table_name_from_table_factor(&tables[0].relation)?
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

/// Prüft Berechtigung für eine einzelne Tabelle
async fn check_single_table_permission(
    connection: &DbConnection,
    extension_id: &str,
    table_name: &str,
    operation: &str,
) -> Result<(), ExtensionError> {
    check_table_permissions(
        connection,
        extension_id,
        &[table_name.to_string()],
        operation,
    )
    .await
}

/// Prüft Berechtigungen für mehrere Tabellen
async fn check_table_permissions(
    connection: &DbConnection,
    extension_id: &str,
    table_names: &[String],
    operation: &str,
) -> Result<(), ExtensionError> {
    let permissions =
        get_extension_permissions(connection, extension_id, "database", operation).await?;

    for table_name in table_names {
        let has_permission = permissions
            .iter()
            .any(|perm| perm.resource.contains(table_name));

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                operation,
                &format!("table '{}'", table_name),
            ));
        }
    }

    Ok(())
}

/// Ruft die Berechtigungen einer Extension aus der Datenbank ab
pub async fn get_extension_permissions(
    connection: &DbConnection,
    extension_id: &str,
    resource: &str,
    operation: &str,
) -> Result<Vec<DbExtensionPermission>, DatabaseError> {
    with_connection(connection, |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id, extension_id, resource, operation, path 
                 FROM haex_vault_extension_permissions 
                 WHERE extension_id = ?1 AND resource = ?2 AND operation = ?3",
            )
            .map_err(|e| DatabaseError::PrepareError {
                reason: e.to_string(),
            })?;

        let rows = stmt
            .query_map([extension_id, resource, operation], |row| {
                Ok(DbExtensionPermission {
                    id: row.get(0)?,
                    extension_id: row.get(1)?,
                    resource: row.get(2)?,
                    operation: row.get(3)?,
                })
            })
            .map_err(|e| DatabaseError::QueryError {
                reason: e.to_string(),
            })?;

        let mut permissions = Vec::new();
        for row_result in rows {
            let permission = row_result.map_err(|e| DatabaseError::DatabaseError {
                reason: e.to_string(),
            })?;
            permissions.push(permission);
        }

        Ok(permissions)
    })
}

#[cfg(test)]
mod tests {
    use crate::extension::error::ExtensionError;

    use super::*;

    #[test]
    fn test_parse_single_statement() {
        let sql = "SELECT * FROM users";
        let result = parse_single_statement(sql);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Query(_)));
    }

    #[test]
    fn test_parse_invalid_sql() {
        let sql = "INVALID SQL";
        let result = parse_single_statement(sql);
        // parse_single_statement gibt DatabaseError zurück, nicht DatabaseError
        assert!(result.is_err());
        // Wenn du spezifischer sein möchtest, kannst du den DatabaseError-Typ prüfen:
        match result {
            Err(DatabaseError::ParseError { .. }) => {
                // Test erfolgreich - wir haben einen ParseError erhalten
            }
            Err(other) => {
                // Andere DatabaseError-Varianten sind auch akzeptabel für ungültiges SQL
                println!("Received other DatabaseError: {:?}", other);
            }
            Ok(_) => panic!("Expected error for invalid SQL"),
        }
    }

    /* #[test]
    fn test_permission_error_access_denied() {
        let error = ExtensionError::access_denied("ext1", "read", "table1", "not allowed");
        match error {
            ExtensionError::AccessDenied {
                extension_id,
                operation,
                resource,
                reason,
            } => {
                assert_eq!(extension_id, "ext1");
                assert_eq!(operation, "read");
                assert_eq!(resource, "table1");
                assert_eq!(reason, "not allowed");
            }
            _ => panic!("Expected AccessDenied error"),
        }
    } */
}
