// src-tauri/src/database/core.rs

use std::collections::HashMap;

use crate::database::error::DatabaseError;
use crate::database::DbConnection;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rusqlite::types::Value as SqlValue;
use rusqlite::{
    types::{Value as RusqliteValue, ValueRef},
    Connection, OpenFlags, ToSql,
};
use serde_json::Value as JsonValue;
use sqlparser::ast::{Query, Select, SetExpr, Statement, TableFactor, TableObject};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

/// Öffnet und initialisiert eine Datenbank mit Verschlüsselung
pub fn open_and_init_db(path: &str, key: &str, create: bool) -> Result<Connection, DatabaseError> {
    let flags = if create {
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    };

    let conn =
        Connection::open_with_flags(path, flags).map_err(|e| DatabaseError::ConnectionFailed {
            path: path.to_string(),
            reason: e.to_string(),
        })?;

    conn.pragma_update(None, "key", key)
        .map_err(|e| DatabaseError::PragmaError {
            pragma: "key".to_string(),
            reason: e.to_string(),
        })?;

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
        .map_err(|e| DatabaseError::PragmaError {
            pragma: "journal_mode=WAL".to_string(),
            reason: e.to_string(),
        })?;

    if journal_mode.eq_ignore_ascii_case("wal") {
        println!("WAL mode successfully enabled.");
    } else {
        eprintln!(
            "Failed to enable WAL mode, journal_mode is '{}'.",
            journal_mode
        );
    }

    Ok(conn)
}

/// Utility für SQL-Parsing - parst ein einzelnes SQL-Statement
pub fn parse_single_statement(sql: &str) -> Result<Statement, DatabaseError> {
    let dialect = SQLiteDialect {};
    let statements = Parser::parse_sql(&dialect, sql).map_err(|e| DatabaseError::ParseError {
        reason: e.to_string(),
        sql: sql.to_string(),
    })?;

    statements
        .into_iter()
        .next()
        .ok_or(DatabaseError::ParseError {
            reason: "No SQL statement found".to_string(),
            sql: sql.to_string(),
        })
}

/// Utility für SQL-Parsing - parst mehrere SQL-Statements
pub fn parse_sql_statements(sql: &str) -> Result<Vec<Statement>, DatabaseError> {
    let dialect = SQLiteDialect {};
    Parser::parse_sql(&dialect, sql).map_err(|e| DatabaseError::ParseError {
        reason: e.to_string(),
        sql: sql.to_string(),
    })
}

pub struct ValueConverter;

impl ValueConverter {
    pub fn json_to_rusqlite_value(json_val: &JsonValue) -> Result<SqlValue, DatabaseError> {
        match json_val {
            JsonValue::Null => Ok(SqlValue::Null),
            JsonValue::Bool(b) => {
                // SQLite hat keinen Bool-Typ; verwende Integer 0/1
                Ok(SqlValue::Integer(if *b { 1 } else { 0 }))
            }
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(SqlValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(SqlValue::Real(f))
                } else {
                    // Fallback: als Text
                    Ok(SqlValue::Text(n.to_string()))
                }
            }
            JsonValue::String(s) => Ok(SqlValue::Text(s.clone())),
            JsonValue::Array(_) | JsonValue::Object(_) => {
                // Arrays/Objects als JSON-Text speichern
                serde_json::to_string(json_val)
                    .map(SqlValue::Text)
                    .map_err(|e| DatabaseError::SerializationError {
                        reason: format!("Failed to serialize JSON param: {}", e),
                    })
            }
        }
    }

    pub fn convert_params(params: &[JsonValue]) -> Result<Vec<SqlValue>, DatabaseError> {
        params.iter().map(Self::json_to_rusqlite_value).collect()
    }
}

pub fn execute(
    sql: String,
    params: Vec<JsonValue>,
    connection: &DbConnection,
) -> Result<usize, DatabaseError> {
    // Konvertiere Parameter
    let params_converted: Vec<RusqliteValue> = params
        .iter()
        .map(ValueConverter::json_to_rusqlite_value)
        .collect::<Result<Vec<_>, _>>()?;
    let params_sql: Vec<&dyn ToSql> = params_converted.iter().map(|v| v as &dyn ToSql).collect();

    with_connection(connection, |conn| {
        let affected_rows = conn.execute(&sql, &params_sql[..]).map_err(|e| {
            // "Lazy Parsing": Extrahiere den Tabellennamen nur, wenn ein Fehler auftritt,
            // um den Overhead bei erfolgreichen Operationen zu vermeiden.
            let table_name = extract_primary_table_name_from_sql(&sql).unwrap_or(None);

            DatabaseError::ExecutionError {
                sql: sql.clone(),
                reason: e.to_string(),
                table: table_name,
            }
        })?;

        Ok(affected_rows)
    })
}

pub fn select(
    sql: String,
    params: Vec<JsonValue>,
    connection: &DbConnection,
) -> Result<Vec<HashMap<String, JsonValue>>, DatabaseError> {
    // Validiere SQL-Statement
    let statement = parse_single_statement(&sql)?;

    // Stelle sicher, dass es eine Query ist
    if !matches!(statement, Statement::Query(_)) {
        return Err(DatabaseError::UnsupportedStatement {
            statement_type: "Non-Query".to_string(),
            description: "Only SELECT statements are allowed in select function".to_string(),
        });
    }

    // Konvertiere Parameter
    let params_converted: Vec<RusqliteValue> = params
        .iter()
        .map(ValueConverter::json_to_rusqlite_value)
        .collect::<Result<Vec<_>, _>>()?;

    let params_sql: Vec<&dyn ToSql> = params_converted.iter().map(|v| v as &dyn ToSql).collect();

    with_connection(connection, |conn| {
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| DatabaseError::PrepareError {
                reason: e.to_string(),
            })?;

        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let num_columns = column_names.len();

        let mut rows = stmt
            .query(&params_sql[..])
            .map_err(|e| DatabaseError::QueryError {
                reason: e.to_string(),
            })?;

        let mut result_vec: Vec<HashMap<String, JsonValue>> = Vec::new();

        while let Some(row) = rows.next().map_err(|e| DatabaseError::RowProcessingError {
            reason: format!("Row iteration error: {}", e),
        })? {
            let mut row_map: HashMap<String, JsonValue> = HashMap::with_capacity(num_columns);

            for i in 0..num_columns {
                let col_name = &column_names[i];

                /* println!(
                    "--- Processing Column --- Index: {}, Name: '{}'",
                    i, col_name
                ); */

                let value_ref = row
                    .get_ref(i)
                    .map_err(|e| DatabaseError::RowProcessingError {
                        reason: format!("Failed to get column {} ('{}'): {}", i, col_name, e),
                    })?;

                let json_val = convert_value_ref_to_json(value_ref)?;

                //println!("Column: {} = {}", column_names[i], json_val);

                row_map.insert(col_name.clone(), json_val);
            }
            result_vec.push(row_map);
        }

        Ok(result_vec)
    })
}

/// Konvertiert rusqlite ValueRef zu JSON
fn convert_value_ref_to_json(value_ref: ValueRef) -> Result<JsonValue, DatabaseError> {
    let json_val = match value_ref {
        ValueRef::Null => JsonValue::Null,
        ValueRef::Integer(i) => JsonValue::Number(i.into()),
        ValueRef::Real(f) => JsonValue::Number(
            serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        ValueRef::Text(t) => {
            let s = String::from_utf8_lossy(t).to_string();
            JsonValue::String(s)
        }
        ValueRef::Blob(b) => {
            // BLOBs als Base64-String zurückgeben
            JsonValue::String(STANDARD.encode(b))
        }
    };
    Ok(json_val)
}
// Extrahiert alle Tabellennamen aus einem SQL-Statement über AST-Parsing
pub fn extract_table_names_from_sql(sql: &str) -> Result<Vec<String>, DatabaseError> {
    let statement = parse_single_statement(sql)?;
    Ok(extract_table_names_from_statement(&statement))
}

/// Extrahiert den ersten/primären Tabellennamen aus einem SQL-Statement
pub fn extract_primary_table_name_from_sql(sql: &str) -> Result<Option<String>, DatabaseError> {
    let table_names = extract_table_names_from_sql(sql)?;
    Ok(table_names.into_iter().next())
}

/// Extrahiert alle Tabellennamen aus einem AST Statement
pub fn extract_table_names_from_statement(statement: &Statement) -> Vec<String> {
    let mut tables = Vec::new();

    match statement {
        Statement::Query(query) => {
            extract_tables_from_query_recursive(query, &mut tables);
        }
        Statement::Insert(insert) => {
            if let TableObject::TableName(name) = &insert.table {
                tables.push(name.to_string());
            }
        }
        Statement::Update { table, .. } => {
            extract_tables_from_table_factor(&table.relation, &mut tables);
        }
        Statement::Delete(delete) => {
            use sqlparser::ast::FromTable;
            match &delete.from {
                FromTable::WithFromKeyword(table_refs) | FromTable::WithoutKeyword(table_refs) => {
                    for table_ref in table_refs {
                        extract_tables_from_table_factor(&table_ref.relation, &mut tables);
                    }
                }
            }
            // Fallback für DELETE-Syntax ohne FROM
            for table_name in &delete.tables {
                tables.push(table_name.to_string());
            }
        }
        Statement::CreateTable(create) => {
            tables.push(create.name.to_string());
        }
        Statement::AlterTable { name, .. } => {
            tables.push(name.to_string());
        }
        Statement::Drop { names, .. } => {
            for name in names {
                tables.push(name.to_string());
            }
        }
        Statement::CreateIndex(create_index) => {
            tables.push(create_index.table_name.to_string());
        }
        Statement::Truncate { table_names, .. } => {
            for table_name in table_names {
                tables.push(table_name.to_string());
            }
        }
        // Weitere Statement-Typen können hier hinzugefügt werden
        _ => {
            // Für unbekannte Statement-Typen geben wir eine leere Liste zurück
        }
    }

    tables
}

/// Extrahiert Tabellennamen rekursiv aus Query-Strukturen
fn extract_tables_from_query_recursive(query: &Query, tables: &mut Vec<String>) {
    extract_tables_from_set_expr_recursive(&query.body, tables);
}

/// Extrahiert Tabellennamen aus SELECT-Statements
fn extract_tables_from_select(select: &Select, tables: &mut Vec<String>) {
    // FROM clause
    for table_ref in &select.from {
        extract_tables_from_table_factor(&table_ref.relation, tables);

        // JOINs
        for join in &table_ref.joins {
            extract_tables_from_table_factor(&join.relation, tables);
        }
    }
}

/// Extrahiert Tabellennamen aus TableFactor-Strukturen
fn extract_tables_from_table_factor(table_factor: &TableFactor, tables: &mut Vec<String>) {
    match table_factor {
        TableFactor::Table { name, .. } => {
            tables.push(name.to_string());
        }
        TableFactor::Derived { subquery, .. } => {
            extract_tables_from_query_recursive(subquery, tables);
        }
        TableFactor::TableFunction { .. } => {
            // Table functions haben normalerweise keine direkten Tabellennamen
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            extract_tables_from_table_factor(&table_with_joins.relation, tables);
            for join in &table_with_joins.joins {
                extract_tables_from_table_factor(&join.relation, tables);
            }
        }
        _ => {
            // TableFunction, UNNEST, JsonTable, etc. haben normalerweise keine direkten Tabellennamen
            // oder sind nicht relevant für SQLite
        }
    }
}

/// Extrahiert Tabellennamen rekursiv aus SetExpr-Strukturen.
/// Diese Funktion enthält die eigentliche rekursive Logik.
fn extract_tables_from_set_expr_recursive(set_expr: &SetExpr, tables: &mut Vec<String>) {
    match set_expr {
        SetExpr::Select(select) => {
            extract_tables_from_select(select, tables);
        }
        SetExpr::Query(sub_query) => {
            extract_tables_from_set_expr_recursive(&sub_query.body, tables);
        }
        SetExpr::SetOperation { left, right, .. } => {
            extract_tables_from_set_expr_recursive(left, tables);
            extract_tables_from_set_expr_recursive(right, tables);
        }

        SetExpr::Values(_)
        | SetExpr::Table(_)
        | SetExpr::Insert(_)
        | SetExpr::Update(_)
        | SetExpr::Delete(_) => {}
    }
}

pub fn with_connection<T, F>(connection: &DbConnection, f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&mut Connection) -> Result<T, DatabaseError>,
{
    let mut db_lock = connection
        .0
        .lock()
        .map_err(|e| DatabaseError::MutexPoisoned {
            reason: e.to_string(),
        })?;

    let conn = db_lock.as_mut().ok_or(DatabaseError::ConnectionError {
        reason: "Connection to vault failed".to_string(),
    })?;

    f(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_simple_select() {
        let sql = "SELECT * FROM users";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_extract_select_with_join() {
        let sql = "SELECT u.name, p.title FROM users u JOIN posts p ON u.id = p.user_id";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users", "posts"]);
    }

    #[test]
    fn test_extract_insert() {
        let sql = "INSERT INTO users (name, email) VALUES (?, ?)";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_extract_update() {
        let sql = "UPDATE users SET name = ? WHERE id = ?";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_extract_delete() {
        let sql = "DELETE FROM users WHERE id = ?";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_extract_create_table() {
        let sql = "CREATE TABLE new_table (id INTEGER, name TEXT)";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["new_table"]);
    }

    #[test]
    fn test_extract_subquery() {
        let sql = "SELECT * FROM (SELECT id FROM users) AS sub";
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users"]);
    }

    #[test]
    fn test_extract_primary_table() {
        let sql = "SELECT u.name FROM users u JOIN posts p ON u.id = p.user_id";
        let primary_table = extract_primary_table_name_from_sql(sql).unwrap();
        assert_eq!(primary_table, Some("users".to_string()));
    }

    #[test]
    fn test_extract_complex_query() {
        let sql = r#"
            SELECT u.name, COUNT(p.id) as post_count 
            FROM users u 
            LEFT JOIN posts p ON u.id = p.user_id 
            WHERE u.created_at > (SELECT MIN(created_at) FROM sessions)
            GROUP BY u.id
        "#;
        let tables = extract_table_names_from_sql(sql).unwrap();
        assert_eq!(tables, vec!["users", "posts", "sessions"]);
    }

    #[test]
    fn test_invalid_sql() {
        let sql = "INVALID SQL";
        let result = extract_table_names_from_sql(sql);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_single_statement() {
        let sql = "SELECT * FROM users WHERE id = ?";
        let result = parse_single_statement(sql);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Statement::Query(_)));
    }

    #[test]
    fn test_parse_invalid_sql() {
        let sql = "INVALID SQL STATEMENT";
        let result = parse_single_statement(sql);
        assert!(matches!(result, Err(DatabaseError::ParseError { .. })));
    }

    #[test]
    fn test_convert_value_ref_to_json() {
        use rusqlite::types::ValueRef;

        assert_eq!(
            convert_value_ref_to_json(ValueRef::Null).unwrap(),
            JsonValue::Null
        );
        assert_eq!(
            convert_value_ref_to_json(ValueRef::Integer(42)).unwrap(),
            JsonValue::Number(42.into())
        );
        assert_eq!(
            convert_value_ref_to_json(ValueRef::Text(b"hello")).unwrap(),
            JsonValue::String("hello".to_string())
        );
    }

    // Test für die neuen AST-basierten Funktionen
    #[test]
    fn test_extract_table_names_comprehensive() {
        // Test verschiedene SQL-Statement-Typen
        assert_eq!(
            extract_primary_table_name_from_sql("SELECT * FROM users WHERE id = 1").unwrap(),
            Some("users".to_string())
        );
        assert_eq!(
            extract_primary_table_name_from_sql("INSERT INTO products (name) VALUES ('test')")
                .unwrap(),
            Some("products".to_string())
        );
        assert_eq!(
            extract_primary_table_name_from_sql("UPDATE orders SET status = 'completed'").unwrap(),
            Some("orders".to_string())
        );
        assert_eq!(
            extract_primary_table_name_from_sql("DELETE FROM customers").unwrap(),
            Some("customers".to_string())
        );
    }
}
