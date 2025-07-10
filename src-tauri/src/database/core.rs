// database/core.rs
use crate::database::DbConnection;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rusqlite::{
    types::{Value as RusqliteValue, ValueRef},
    Connection, OpenFlags, ToSql,
};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::Path;
use tauri::State;
// --- Hilfsfunktion: Konvertiert JSON Value zu etwas, das rusqlite versteht ---
// Diese Funktion ist etwas knifflig wegen Ownership und Lifetimes.
// Eine einfachere Variante ist oft, direkt rusqlite::types::Value zu erstellen.
// Hier ein Beispiel, das owned Values erstellt (braucht evtl. Anpassung je nach rusqlite-Version/Nutzung)
fn json_to_rusqlite_value(json_val: &JsonValue) -> Result<RusqliteValue, String> {
    match json_val {
        JsonValue::Null => Ok(RusqliteValue::Null),
        JsonValue::Bool(b) => Ok(RusqliteValue::Integer(*b as i64)), // SQLite hat keinen BOOLEAN
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(RusqliteValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(RusqliteValue::Real(f))
            } else {
                Err("Ungültiger Zahlenwert".to_string())
            }
        }
        JsonValue::String(s) => Ok(RusqliteValue::Text(s.clone())),
        JsonValue::Array(_) | JsonValue::Object(_) => {
            // SQLite kann Arrays/Objects nicht direkt speichern (außer als TEXT/BLOB)
            // Konvertiere sie zu JSON-Strings, wenn das gewünscht ist
            Ok(RusqliteValue::Text(
                serde_json::to_string(json_val).map_err(|e| e.to_string())?,
            ))
            // Oder gib einen Fehler zurück, wenn Arrays/Objekte nicht erlaubt sind
            // Err("Arrays oder Objekte werden nicht direkt als Parameter unterstützt".to_string())
        }
    }
}

// --- Tauri Command für INSERT/UPDATE/DELETE ---
#[tauri::command]
pub async fn execute(
    sql: String,
    params: Vec<JsonValue>,
    state: &State<'_, DbConnection>,
) -> Result<usize, String> {
    // Gibt Anzahl betroffener Zeilen zurück

    let params_converted: Vec<RusqliteValue> = params
        .iter()
        .map(json_to_rusqlite_value)
        .collect::<Result<Vec<_>, _>>()?;
    let params_sql: Vec<&dyn ToSql> = params_converted.iter().map(|v| v as &dyn ToSql).collect();

    let db_lock = state
        .0
        .lock()
        .map_err(|e| format!("Mutex Lock Fehler: {}", e))?;
    let conn = db_lock.as_ref().ok_or("Keine Datenbankverbindung")?;

    let affected_rows = conn
        .execute(&sql, &params_sql[..])
        .map_err(|e| format!("SQL Execute Fehler: {}", e))?;

    Ok(affected_rows)
}

#[tauri::command]
pub async fn select(
    sql: String,
    params: Vec<JsonValue>,
    state: &State<'_, DbConnection>,
) -> Result<Vec<Vec<JsonValue>>, String> {
    // Ergebnis als Vec<RowObject>

    // Konvertiere JSON Params zu rusqlite Values für die Abfrage
    // Wir sammeln sie als owned Values, da `params_from_iter` Referenzen braucht,
    // was mit lokalen Konvertierungen schwierig ist.
    let params_converted: Vec<RusqliteValue> = params
        .iter()
        .map(json_to_rusqlite_value)
        .collect::<Result<Vec<_>, _>>()?; // Sammle Ergebnisse, gibt Fehler weiter

    // Konvertiere zu Slice von ToSql-Referenzen (erfordert, dass die Values leben)
    let params_sql: Vec<&dyn ToSql> = params_converted.iter().map(|v| v as &dyn ToSql).collect();

    // Zugriff auf die Verbindung (blockierend, okay für SQLite in vielen Fällen)
    let db_lock = state
        .0
        .lock()
        .map_err(|e| format!("Mutex Lock Fehler: {}", e))?;
    let conn = db_lock.as_ref().ok_or("Keine Datenbankverbindung")?;

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("SQL Prepare Fehler: {}", e))?;
    let column_names: Vec<String> = stmt
        .column_names()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let num_columns = column_names.len();

    let mut rows = stmt
        .query(&params_sql[..])
        .map_err(|e| format!("SQL Query Fehler: {}", e))?;
    let mut result_vec: Vec<Vec<JsonValue>> = Vec::new();

    println!();
    println!();
    println!();
    println!();

    while let Some(row) = rows.next().map_err(|e| format!("Row Next Fehler: {}", e))? {
        //let mut row_map = HashMap::new();
        let mut row_data: Vec<JsonValue> = Vec::with_capacity(num_columns);
        for i in 0..num_columns {
            let col_name = &column_names[i];

            println!(
                "--- Processing Column --- Index: {}, Name: '{}'",
                i, col_name
            );
            let value_ref = row
                .get_ref(i)
                .map_err(|e| format!("Get Ref Fehler Spalte {}: {}", i, e))?;

            // Wandle rusqlite ValueRef zurück zu serde_json Value
            let json_val = match value_ref {
                ValueRef::Null => JsonValue::Null,
                ValueRef::Integer(i) => JsonValue::Number(i.into()),
                ValueRef::Real(f) => JsonValue::Number(
                    serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
                ), // Fallback für NaN/Infinity
                ValueRef::Text(t) => {
                    let s = String::from_utf8_lossy(t).to_string();
                    // Versuche, als JSON zu parsen, falls es ursprünglich ein Array/Objekt war
                    //serde_json::from_str(&s).unwrap_or(JsonValue::String(s))
                    JsonValue::String(s)
                }
                ValueRef::Blob(b) => {
                    // BLOBs z.B. als Base64-String zurückgeben
                    JsonValue::String(STANDARD.encode(b))
                }
            };
            println!(
                "new row: name: {} with value: {}",
                column_names[i].clone(),
                json_val,
            );
            row_data.push(json_val);
            //row_map.insert(column_names[i].clone(), json_val);
        }
        //result_vec.push(row_map);
        result_vec.push(row_data);
    }

    Ok(result_vec)
}

/// Öffnet und initialisiert eine Datenbank mit Verschlüsselung
pub fn open_and_init_db(path: &str, key: &str, create: bool) -> Result<Connection, String> {
    let flags = if create {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    };

    let conn = Connection::open_with_flags(path, flags).map_err(|e| {
        format!(
            "Dateiii gibt es nicht: {}. Habe nach {} gesucht",
            e.to_string(),
            path
        )
    })?;
    conn.pragma_update(None, "key", key)
        .map_err(|e| e.to_string())?;

    conn.execute_batch("SELECT count(*) from haex_extensions")
        .map_err(|e| e.to_string())?;

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if journal_mode.eq_ignore_ascii_case("wal") {
        println!("WAL mode successfully enabled.");
    } else {
        eprintln!("Failed to enable WAL mode.");
    }

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
