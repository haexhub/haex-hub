// database/permissions.rs
use crate::database::core::extract_tables_from_query;
use crate::database::DbConnection;
use crate::models::DbExtensionPermission;
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;
use tauri::{AppHandle, Manager};

/// Prüft Leseberechtigungen für eine Extension basierend auf Datenbankeinträgen
pub async fn check_read_permission(
    app: &AppHandle,
    extension_id: &str,
    sql: &str,
) -> Result<(), String> {
    // SQL-Statement parsen
    let dialect = SQLiteDialect {};
    let statements = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;
    let statement = statements
        .into_iter()
        .next()
        .ok_or("Keine SQL-Anweisung gefunden")?;

    // Berechtigungsprüfung für SELECT-Statements
    if let sqlparser::ast::Statement::Query(query) = statement {
        let tables = extract_tables_from_query(&query);

        // Berechtigungen aus der Datenbank abrufen
        let db_state = app.state::<DbConnection>();
        let permissions =
            get_extension_permissions(db_state, extension_id, "database", "read").await?;

        // Prüfen, ob alle benötigten Tabellen in den Berechtigungen enthalten sind
        for table in tables {
            let has_permission = permissions.iter().any(|perm| perm.path.contains(&table));

            if !has_permission {
                return Err(format!("Keine Leseberechtigung für Tabelle {}", table));
            }
        }

        Ok(())
    } else {
        Err("Nur SELECT-Anweisungen erlaubt".into())
    }
}

/// Prüft Schreibberechtigungen für eine Extension basierend auf Datenbankeinträgen
pub async fn check_write_permission(
    app: &AppHandle,
    extension_id: &str,
    sql: &str,
) -> Result<(), String> {
    // SQL-Statement parsen
    let dialect = SQLiteDialect {};
    let statements = Parser::parse_sql(&dialect, sql).map_err(|e| e.to_string())?;
    let statement = statements
        .into_iter()
        .next()
        .ok_or("Keine SQL-Anweisung gefunden")?;

    // Berechtigungsprüfung basierend auf Statement-Typ
    match statement {
        sqlparser::ast::Statement::Insert(insert) => {
            let table_name = match insert.table {
                sqlparser::ast::TableObject::TableName(name) => name.to_string(),
                _ => return Err("Ungültige Tabellenangabe in INSERT".into()),
            };

            // Berechtigungen aus der Datenbank abrufen
            let db_state = app.state::<DbConnection>();
            let permissions =
                get_extension_permissions(db_state, extension_id, "database", "write").await?;

            // Prüfen, ob die Tabelle in den Berechtigungen enthalten ist
            let has_permission = permissions
                .iter()
                .any(|perm| perm.path.contains(&table_name));

            if !has_permission {
                return Err(format!(
                    "Keine Schreibberechtigung für Tabelle {}",
                    table_name
                ));
            }
        }
        sqlparser::ast::Statement::Update { table, .. } => {
            let table_name = table.relation.to_string();

            // Berechtigungen aus der Datenbank abrufen
            let db_state = app.state::<DbConnection>();
            let permissions =
                get_extension_permissions(db_state, extension_id, "database", "write").await?;

            // Prüfen, ob die Tabelle in den Berechtigungen enthalten ist
            let has_permission = permissions
                .iter()
                .any(|perm| perm.path.contains(&table_name));

            if !has_permission {
                return Err(format!(
                    "Keine Schreibberechtigung für Tabelle {}",
                    table_name
                ));
            }
        }
        sqlparser::ast::Statement::Delete(delete) => {
            let from_tables = match delete.from {
                sqlparser::ast::FromTable::WithFromKeyword(tables) => tables,
                sqlparser::ast::FromTable::WithoutKeyword(tables) => tables,
            };
            if from_tables.is_empty() && delete.tables.is_empty() {
                return Err("Keine Tabelle in DELETE angegeben".into());
            }
            let table_name = if !from_tables.is_empty() {
                from_tables[0].relation.to_string()
            } else {
                delete.tables[0].to_string()
            };

            // Berechtigungen aus der Datenbank abrufen
            let db_state = app.state::<DbConnection>();
            let permissions =
                get_extension_permissions(db_state, extension_id, "database", "write").await?;

            // Prüfen, ob die Tabelle in den Berechtigungen enthalten ist
            let has_permission = permissions
                .iter()
                .any(|perm| perm.path.contains(&table_name));

            if !has_permission {
                return Err(format!(
                    "Keine Schreibberechtigung für Tabelle {}",
                    table_name
                ));
            }
        }
        sqlparser::ast::Statement::CreateTable(create_table) => {
            let table_name = create_table.name.to_string();

            // Berechtigungen aus der Datenbank abrufen
            let db_state = app.state::<DbConnection>();
            let permissions =
                get_extension_permissions(db_state, extension_id, "database", "create").await?;

            // Prüfen, ob die Tabelle in den Berechtigungen enthalten ist
            let has_permission = permissions
                .iter()
                .any(|perm| perm.path.contains(&table_name));

            if !has_permission {
                return Err(format!(
                    "Keine Erstellungsberechtigung für Tabelle {}",
                    table_name
                ));
            }
        }
        _ => return Err("Nur Schreiboperationen erlaubt (nutze 'select' für Abfragen)".into()),
    }

    Ok(())
}

/// Ruft die Berechtigungen einer Extension aus der Datenbank ab
async fn get_extension_permissions(
    db_state: tauri::State<'_, DbConnection>,
    extension_id: &str,
    resource: &str,
    operation: &str,
) -> Result<Vec<DbExtensionPermission>, String> {
    let db = db_state
        .0
        .lock()
        .map_err(|e| format!("Mutex-Fehler: {}", e))?;

    let conn = db.as_ref().ok_or("Keine Datenbankverbindung vorhanden")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, extension_id, resource, operation, path 
         FROM haex_vault_extension_permissions 
         WHERE extension_id = ? AND resource = ? AND operation = ?",
        )
        .map_err(|e| format!("SQL-Vorbereitungsfehler: {}", e))?;

    let rows = stmt
        .query_map(&[extension_id, resource, operation], |row| {
            Ok(DbExtensionPermission {
                id: row.get(0)?,
                extension_id: row.get(1)?,
                resource: row.get(2)?,
                operation: row.get(3)?,
                path: row.get(4)?,
            })
        })
        .map_err(|e| format!("SQL-Abfragefehler: {}", e))?;

    let mut permissions = Vec::new();
    for row in rows {
        permissions.push(row.map_err(|e| format!("Fehler beim Lesen der Berechtigungen: {}", e))?);
    }

    Ok(permissions)
}
