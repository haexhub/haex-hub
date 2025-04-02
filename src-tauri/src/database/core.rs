// database/core.rs
use crate::database::DbConnection;
use rusqlite::{Connection, OpenFlags};
use serde_json::json;
use std::fs;
use std::path::Path;
use tauri::State;

/// Führt SQL-Schreiboperationen (INSERT, UPDATE, DELETE, CREATE) ohne Berechtigungsprüfung aus
pub async fn execute(
    sql: &str,
    params: &[String],
    state: &State<'_, DbConnection>,
) -> Result<String, String> {
    let db = state.0.lock().map_err(|e| format!("Mutex-Fehler: {}", e))?;
    let conn = db.as_ref().ok_or("Keine Datenbankverbindung vorhanden")?;

    let rows_affected = conn
        .execute(sql, rusqlite::params_from_iter(params.iter()))
        .map_err(|e| format!("SQL-Ausführungsfehler: {}", e))?;

    let last_id = conn.last_insert_rowid();

    Ok(serde_json::to_string(&json!({
        "rows_affected": rows_affected,
        "last_insert_id": last_id
    }))
    .map_err(|e| format!("JSON-Serialisierungsfehler: {}", e))?)
}

/// Führt SQL-Leseoperationen (SELECT) ohne Berechtigungsprüfung aus
pub async fn select(
    sql: &str,
    params: &[String],
    state: &State<'_, DbConnection>,
) -> Result<Vec<Vec<String>>, String> {
    let db = state.0.lock().map_err(|e| format!("Mutex-Fehler: {}", e))?;
    let conn = db.as_ref().ok_or("Keine Datenbankverbindung vorhanden")?;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL-Vorbereitungsfehler: {}", e))?;
    let columns = stmt.column_count();
    let mut rows = stmt
        .query(rusqlite::params_from_iter(params.iter()))
        .map_err(|e| format!("SQL-Abfragefehler: {}", e))?;

    let mut result = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Zeilenabruffehler: {}", e))?
    {
        let mut row_data = Vec::new();
        for i in 0..columns {
            let value: String = row
                .get(i)
                .map_err(|e| format!("Datentypfehler in Spalte {}: {}", i, e))?;
            row_data.push(value);
        }
        result.push(row_data);
    }

    Ok(result)
}

/// Öffnet und initialisiert eine Datenbank mit Verschlüsselung
pub fn open_and_init_db(path: &str, key: &str, create: bool) -> Result<Connection, String> {
    let flags = if create {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    };

    let conn = Connection::open_with_flags(path, flags).map_err(|e| e.to_string())?;
    conn.pragma_update(None, "key", key)
        .map_err(|e| e.to_string())?;

    Ok(conn)
}

/// Kopiert eine Datei von einem Pfad zu einem anderen
pub fn copy_file<S: AsRef<Path>, T: AsRef<Path>>(
    source_path: S,
    target_path: T,
) -> Result<(), String> {
    let source = source_path.as_ref();
    let target = target_path.as_ref();

    // Check if source file exists
    if !source.exists() {
        return Err(format!("Source file '{}' does not exist", source.display()));
    }

    // Check if source is a file (not a directory)
    if !source.is_file() {
        return Err(format!("Source '{}' is not a file", source.display()));
    }

    // Copy the file and preserve metadata (permissions, timestamps)
    fs::copy(source, target)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(())
}

// Hilfsfunktionen für SQL-Parsing
pub fn extract_tables_from_query(query: &sqlparser::ast::Query) -> Vec<String> {
    let mut tables = Vec::new();
    extract_tables_from_set_expr(&query.body, &mut tables);
    tables
}

fn extract_tables_from_set_expr(set_expr: &sqlparser::ast::SetExpr, tables: &mut Vec<String>) {
    match set_expr {
        sqlparser::ast::SetExpr::Select(select) => {
            for from in &select.from {
                extract_tables_from_table_with_joins(from, tables);
            }
        }
        sqlparser::ast::SetExpr::Query(query) => {
            extract_tables_from_set_expr(&query.body, tables);
        }
        sqlparser::ast::SetExpr::SetOperation { left, right, .. } => {
            extract_tables_from_set_expr(left, tables);
            extract_tables_from_set_expr(right, tables);
        }
        _ => (), // Andere Fälle wie Values oder Insert ignorieren
    }
}

fn extract_tables_from_table_with_joins(
    table_with_joins: &sqlparser::ast::TableWithJoins,
    tables: &mut Vec<String>,
) {
    extract_tables_from_table_factor(&table_with_joins.relation, tables);
    for join in &table_with_joins.joins {
        extract_tables_from_table_factor(&join.relation, tables);
    }
}

fn extract_tables_from_table_factor(
    table_factor: &sqlparser::ast::TableFactor,
    tables: &mut Vec<String>,
) {
    match table_factor {
        sqlparser::ast::TableFactor::Table { name, .. } => {
            tables.push(name.to_string());
        }
        sqlparser::ast::TableFactor::Derived { subquery, .. } => {
            extract_tables_from_set_expr(&subquery.body, tables);
        }
        sqlparser::ast::TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            extract_tables_from_table_with_joins(table_with_joins, tables);
        }
        _ => (), // Andere Fälle wie TableFunction ignorieren
    }
}
