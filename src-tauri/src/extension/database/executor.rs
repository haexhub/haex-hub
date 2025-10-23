// src-tauri/src/extension/database/executor.rs (neu)

use crate::crdt::hlc::HlcService;
use crate::crdt::transformer::CrdtTransformer;
use crate::crdt::trigger;
use crate::database::core::{convert_value_ref_to_json, parse_sql_statements, ValueConverter};
use crate::database::error::DatabaseError;
use rusqlite::Connection;
use rusqlite::{params_from_iter, types::Value as SqliteValue, ToSql, Transaction};
use serde_json::{Map, Value as JsonValue};
use sqlparser::ast::{Insert, Statement, TableObject};
use std::collections::{HashMap, HashSet};

/// Repräsentiert PK-Werte für eine Zeile (kann single oder composite key sein)
#[derive(Debug, Clone, PartialEq, Eq)]
struct PkValues {
    /// column_name -> value
    values: HashMap<String, String>,
}

impl PkValues {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn insert(&mut self, column: String, value: String) {
        self.values.insert(column, value);
    }

    fn get(&self, column: &str) -> Option<&String> {
        self.values.get(column)
    }
}

/// Context für PK-Remapping während einer Transaktion
/// Trackt für jede Tabelle: welche PKs sollten eingefügt werden vs. welche sind tatsächlich in der DB
#[derive(Debug, Default)]
pub struct PkRemappingContext {
    /// Für jede Tabelle: Liste von (original_pk_values, actual_pk_values) Mappings
    /// Wird nur gespeichert wenn original != actual (d.h. ON CONFLICT hat PK geändert)
    mappings: HashMap<String, Vec<(PkValues, PkValues)>>,
}

impl PkRemappingContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt ein Mapping für eine Tabelle hinzu, aber nur wenn original != actual
    /// original und actual sind die PK-Werte vor und nach dem INSERT
    fn add_mapping(&mut self, table: String, original: PkValues, actual: PkValues) {
        // Nur speichern wenn tatsächlich unterschiedlich (ON CONFLICT hat stattgefunden)
        if original != actual {
            eprintln!(
                "DEBUG: PK Remapping for table '{}': {:?} -> {:?}",
                table, original.values, actual.values
            );
            self.mappings
                .entry(table)
                .or_insert_with(Vec::new)
                .push((original, actual));
        }
    }

    /// Versucht einen FK-Wert zu remappen
    /// referenced_table: Die Tabelle auf die der FK zeigt
    /// referenced_column: Die PK-Spalte in der referenced_table
    /// value: Der FK-Wert der ersetzt werden soll
    fn remap_fk_value(
        &self,
        referenced_table: &str,
        referenced_column: &str,
        value: &str,
    ) -> String {
        self.mappings
            .get(referenced_table)
            .and_then(|mappings| {
                mappings.iter().find_map(|(original, actual)| {
                    if original.get(referenced_column)? == value {
                        let actual_val = actual.get(referenced_column)?.clone();
                        eprintln!(
                            "DEBUG: FK Remapping for {}.{}: {} -> {}",
                            referenced_table, referenced_column, value, actual_val
                        );
                        Some(actual_val)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| value.to_string())
    }
}

/// SQL-Executor OHNE Berechtigungsprüfung - für interne Nutzung
pub struct SqlExecutor;

impl SqlExecutor {
    /// Führt ein SQL Statement OHNE RETURNING aus (mit CRDT und PK-Remapping)
    /// Unterstützt automatisches FK-Remapping wenn vorherige INSERTs ON CONFLICT getriggert haben
    ///
    /// Diese Variante akzeptiert &[&dyn ToSql] direkt (wie von rusqlite::params![] erzeugt)
    /// Returns: modified_schema_tables
    pub fn execute_internal_typed_with_context(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: &[&dyn ToSql],
        pk_context: &mut PkRemappingContext,
    ) -> Result<HashSet<String>, DatabaseError> {
        let mut ast_vec = parse_sql_statements(sql)?;

        if ast_vec.len() != 1 {
            return Err(DatabaseError::ExecutionError {
                sql: sql.to_string(),
                reason: "execute_internal_typed_with_context sollte nur ein einzelnes SQL-Statement erhalten"
                    .to_string(),
                table: None,
            });
        }

        let mut statement = ast_vec.pop().unwrap();

        let transformer = CrdtTransformer::new();
        let hlc_timestamp =
            hlc_service
                .new_timestamp_and_persist(tx)
                .map_err(|e| DatabaseError::HlcError {
                    reason: e.to_string(),
                })?;

        let mut modified_schema_tables = HashSet::new();
        if let Some(table_name) = transformer.transform_execute_statement_with_table_info(
            &mut statement,
            &hlc_timestamp,
        )? {
            modified_schema_tables.insert(table_name);
        }

        let sql_str = statement.to_string();
        eprintln!("DEBUG: Transformed SQL (execute path): {}", sql_str);

        // Spezielle Behandlung für INSERT Statements (mit FK-Remapping, OHNE RETURNING)
        if let Statement::Insert(ref insert_stmt) = statement {
            if let TableObject::TableName(ref table_name) = insert_stmt.table {
                let table_name_str = table_name
                    .to_string()
                    .trim_matches('`')
                    .trim_matches('"')
                    .to_string();

                // Konvertiere Params zu Vec für Manipulation
                let mut param_vec = params_to_vec(params, tx)?;

                // Hole Foreign Key Informationen
                let fk_info = get_fk_info(tx, &table_name_str)?;

                // Remap FK-Werte in params (falls Mappings existieren)
                remap_fk_params(insert_stmt, &mut param_vec, &fk_info, pk_context)?;

                let param_refs: Vec<&dyn ToSql> =
                    param_vec.iter().map(|v| v as &dyn ToSql).collect();

                let mut stmt = tx
                    .prepare(&sql_str)
                    .map_err(|e| DatabaseError::ExecutionError {
                        sql: sql_str.clone(),
                        table: Some(table_name_str.clone()),
                        reason: format!("Prepare failed: {}", e),
                    })?;

                let mut rows = stmt
                    .query(params_from_iter(param_refs.iter()))
                    .map_err(|e| DatabaseError::ExecutionError {
                        sql: sql_str.clone(),
                        table: Some(table_name_str.clone()),
                        reason: format!("INSERT query execution failed: {}", e),
                    })?;

                let _ = rows.next()?;
            }
        } else {
            // Nicht-INSERT Statements normal ausführen
            tx.execute(&sql_str, params)
                .map_err(|e| DatabaseError::ExecutionError {
                    sql: sql_str.clone(),
                    table: None,
                    reason: format!("Execute failed: {}", e),
                })?;
        }

        // Trigger-Logik für CREATE TABLE
        if let Statement::CreateTable(create_table_details) = statement {
            let table_name_str = create_table_details.name.to_string();
            trigger::setup_triggers_for_table(tx, &table_name_str, false)?;
        }

        Ok(modified_schema_tables)
    }

    /// Führt ein SQL Statement MIT RETURNING aus (mit CRDT und PK-Remapping)
    /// Unterstützt automatisches FK-Remapping wenn vorherige INSERTs ON CONFLICT getriggert haben
    ///
    /// Diese Variante akzeptiert &[&dyn ToSql] direkt (wie von rusqlite::params![] erzeugt)
    /// Returns: (modified_schema_tables, returning_results)
    /// returning_results enthält ALLE RETURNING-Spalten für INSERT/UPDATE/DELETE mit RETURNING
    pub fn query_internal_typed_with_context(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: &[&dyn ToSql],
        pk_context: &mut PkRemappingContext,
    ) -> Result<(HashSet<String>, Vec<Vec<JsonValue>>), DatabaseError> {
        let mut ast_vec = parse_sql_statements(sql)?;

        if ast_vec.len() != 1 {
            return Err(DatabaseError::ExecutionError {
                sql: sql.to_string(),
                reason: "query_internal_typed_with_context sollte nur ein einzelnes SQL-Statement erhalten"
                    .to_string(),
                table: None,
            });
        }

        let mut statement = ast_vec.pop().unwrap();

        let transformer = CrdtTransformer::new();
        let hlc_timestamp =
            hlc_service
                .new_timestamp_and_persist(tx)
                .map_err(|e| DatabaseError::HlcError {
                    reason: e.to_string(),
                })?;

        let mut modified_schema_tables = HashSet::new();
        if let Some(table_name) = transformer.transform_execute_statement_with_table_info(
            &mut statement,
            &hlc_timestamp,
        )? {
            modified_schema_tables.insert(table_name);
        }

        let sql_str = statement.to_string();
        eprintln!("DEBUG: Transformed SQL (with RETURNING): {}", sql_str);

        // Spezielle Behandlung für INSERT Statements (mit PK-Remapping + RETURNING)
        if let Statement::Insert(ref insert_stmt) = statement {
            if let TableObject::TableName(ref table_name) = insert_stmt.table {
                let table_name_str = table_name
                    .to_string()
                    .trim_matches('`')
                    .trim_matches('"')
                    .to_string();

                // Konvertiere Params zu Vec für Manipulation
                let mut param_vec = params_to_vec(params, tx)?;

                // Hole Table Schema um PKs und FKs zu identifizieren
                let table_columns =
                    trigger::get_table_schema(tx, &table_name_str).map_err(|e| {
                        DatabaseError::ExecutionError {
                            sql: format!("PRAGMA table_info('{}')", table_name_str),
                            reason: e.to_string(),
                            table: Some(table_name_str.clone()),
                        }
                    })?;

                let pk_columns: Vec<String> = table_columns
                    .iter()
                    .filter(|c| c.is_pk)
                    .map(|c| c.name.clone())
                    .collect();

                // Hole Foreign Key Informationen
                let fk_info = get_fk_info(tx, &table_name_str)?;

                // 1. Extrahiere Original PK-Werte aus params (vor FK-Remapping)
                let original_pk =
                    extract_pk_values_from_params(insert_stmt, &param_vec, &pk_columns)?;

                // 2. Remap FK-Werte in params (falls Mappings existieren)
                remap_fk_params(insert_stmt, &mut param_vec, &fk_info, pk_context)?;

                // 3. Führe INSERT mit query() aus um RETURNING zu lesen
                let mut stmt = tx
                    .prepare(&sql_str)
                    .map_err(|e| DatabaseError::ExecutionError {
                        sql: sql_str.clone(),
                        table: Some(table_name_str.clone()),
                        reason: e.to_string(),
                    })?;

                let column_names: Vec<String> = stmt
                    .column_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                let num_columns = column_names.len();

                let param_refs: Vec<&dyn ToSql> =
                    param_vec.iter().map(|v| v as &dyn ToSql).collect();

                let mut rows = stmt
                    .query(params_from_iter(param_refs.iter()))
                    .map_err(|e| DatabaseError::ExecutionError {
                        sql: sql_str.clone(),
                        table: Some(table_name_str.clone()),
                        reason: e.to_string(),
                    })?;

                let mut result_vec: Vec<Vec<JsonValue>> = Vec::new();

                // 4. Lese ALLE RETURNING Werte und speichere PK-Mapping
                while let Some(row) = rows.next().map_err(|e| DatabaseError::ExecutionError {
                    sql: sql_str.clone(),
                    table: Some(table_name_str.clone()),
                    reason: e.to_string(),
                })? {
                    // Extrahiere PK-Werte für PK-Remapping
                    let actual_pk = extract_pk_values_from_row(&row, &pk_columns)?;
                    pk_context.add_mapping(
                        table_name_str.clone(),
                        original_pk.clone(),
                        actual_pk.clone(),
                    );

                    // Extrahiere ALLE Spalten für RETURNING-Ergebnis
                    let mut row_values: Vec<JsonValue> = Vec::with_capacity(num_columns);

                    for i in 0..num_columns {
                        let value_ref =
                            row.get_ref(i)
                                .map_err(|e| DatabaseError::RowProcessingError {
                                    reason: format!("Failed to get column {}: {}", i, e),
                                })?;
                        let json_val = convert_value_ref_to_json(value_ref)?;
                        row_values.push(json_val);
                    }
                    result_vec.push(row_values);
                }

                return Ok((modified_schema_tables, result_vec));
            }
        }

        // Für UPDATE/DELETE mit RETURNING: query() verwenden (kein PK-Remapping nötig)
        let mut stmt = tx
            .prepare(&sql_str)
            .map_err(|e| DatabaseError::PrepareError {
                reason: e.to_string(),
            })?;

        let num_columns = stmt.column_count();

        let mut rows = stmt.query(params).map_err(|e| DatabaseError::QueryError {
            reason: e.to_string(),
        })?;

        let mut result_vec: Vec<Vec<JsonValue>> = Vec::new();

        while let Some(row) = rows.next().map_err(|e| DatabaseError::RowProcessingError {
            reason: format!("Row iteration error: {}", e),
        })? {
            let mut row_values: Vec<JsonValue> = Vec::with_capacity(num_columns);

            for i in 0..num_columns {
                let value_ref = row
                    .get_ref(i)
                    .map_err(|e| DatabaseError::RowProcessingError {
                        reason: format!("Failed to get column {}: {}", i, e),
                    })?;

                let json_val = convert_value_ref_to_json(value_ref)?;
                row_values.push(json_val);
            }
            result_vec.push(row_values);
        }

        Ok((modified_schema_tables, result_vec))
    }

    /// Legacy-Methode ohne PK-Remapping Context
    pub fn execute_internal_typed(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<HashSet<String>, DatabaseError> {
        let mut context = PkRemappingContext::new();
        Self::execute_internal_typed_with_context(tx, hlc_service, sql, params, &mut context)
    }
    /// Führt SQL aus (mit CRDT-Transformation) - OHNE Permission-Check
    /// Wrapper um execute_internal_typed für JsonValue-Parameter
    /// Nutzt PK-Remapping Logik für INSERT mit ON CONFLICT
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

        // Convert JsonValue params to SqliteValue
        let params_converted: Vec<SqliteValue> = params
            .iter()
            .map(ValueConverter::json_to_rusqlite_value)
            .collect::<Result<Vec<_>, _>>()?;

        // Convert to &dyn ToSql references
        let param_refs: Vec<&dyn ToSql> =
            params_converted.iter().map(|v| v as &dyn ToSql).collect();

        // Call execute_internal_typed (mit PK-Remapping!)
        Self::execute_internal_typed(tx, hlc_service, sql, &param_refs)
    }

    /// Führt SELECT aus (mit CRDT-Transformation) - OHNE Permission-Check
    pub fn select_internal(
        conn: &Connection,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
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

        // Hard Delete: Keine SELECT-Transformation mehr nötig
        let stmt_to_execute = ast_vec.pop().unwrap();
        let transformed_sql = stmt_to_execute.to_string();

        eprintln!("DEBUG: SELECT (no transformation): {}", transformed_sql);

        let mut prepared_stmt = conn.prepare(&transformed_sql)?;

        let num_columns = prepared_stmt.column_count();

        let mut rows = prepared_stmt
            .query(params_from_iter(&sql_params[..]))
            .map_err(|e| DatabaseError::QueryError {
                reason: e.to_string(),
            })?;

        let mut result_vec: Vec<Vec<JsonValue>> = Vec::new();

        while let Some(row) = rows.next().map_err(|e| DatabaseError::RowProcessingError {
            reason: format!("Row iteration error: {}", e),
        })? {
            let mut row_values: Vec<JsonValue> = Vec::with_capacity(num_columns);

            for i in 0..num_columns {
                let value_ref = row
                    .get_ref(i)
                    .map_err(|e| DatabaseError::RowProcessingError {
                        reason: format!("Failed to get column {}: {}", i, e),
                    })?;

                let json_val = convert_value_ref_to_json(value_ref)?;
                row_values.push(json_val);
            }
            result_vec.push(row_values);
        }

        Ok(result_vec)
    }

    /// Führt SQL mit CRDT-Transformation aus und gibt RETURNING-Ergebnisse zurück
    /// Speziell für INSERT/UPDATE/DELETE mit RETURNING (Drizzle-Integration)
    /// Nutzt PK-Remapping für INSERT-Operationen
    pub fn query_internal(
        tx: &Transaction,
        hlc_service: &HlcService,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
        // Parameter validation
        let total_placeholders = sql.matches('?').count();
        if total_placeholders != params.len() {
            return Err(DatabaseError::ParameterMismatchError {
                expected: total_placeholders,
                provided: params.len(),
                sql: sql.to_string(),
            });
        }

        // Parameter konvertieren
        let params_converted: Vec<SqliteValue> = params
            .iter()
            .map(ValueConverter::json_to_rusqlite_value)
            .collect::<Result<Vec<_>, _>>()?;

        // Convert to &dyn ToSql references
        let param_refs: Vec<&dyn ToSql> =
            params_converted.iter().map(|v| v as &dyn ToSql).collect();

        // Call query_internal_typed_with_context (mit PK-Remapping!)
        let mut context = PkRemappingContext::new();
        let (_tables, results) = Self::query_internal_typed_with_context(
            tx,
            hlc_service,
            sql,
            &param_refs,
            &mut context,
        )?;

        Ok(results)
    }
}

// =========================
// Helper-Funktionen für FK-Remapping
// =========================

/// Strukturiert FK-Informationen für einfache Lookups
#[derive(Debug)]
struct FkInfo {
    /// column_name -> (referenced_table, referenced_column)
    mappings: HashMap<String, (String, String)>,
}

/// Hole Foreign Key Informationen für eine Tabelle
fn get_fk_info(tx: &Transaction, table_name: &str) -> Result<FkInfo, DatabaseError> {
    // Nutze PRAGMA foreign_key_list um FK-Beziehungen zu holen
    let sql = format!("PRAGMA foreign_key_list('{}');", table_name);
    let mut stmt = tx
        .prepare(&sql)
        .map_err(|e| DatabaseError::ExecutionError {
            sql: sql.clone(),
            reason: e.to_string(),
            table: Some(table_name.to_string()),
        })?;

    let mut mappings = HashMap::new();
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("from")?,  // FK column in this table
                row.get::<_, String>("table")?, // referenced table
                row.get::<_, String>("to")?,    // referenced column
            ))
        })
        .map_err(|e| DatabaseError::ExecutionError {
            sql,
            reason: e.to_string(),
            table: Some(table_name.to_string()),
        })?;

    for row in rows {
        let (from_col, ref_table, ref_col) = row.map_err(|e| DatabaseError::ExecutionError {
            sql: format!("PRAGMA foreign_key_list('{}')", table_name),
            reason: e.to_string(),
            table: Some(table_name.to_string()),
        })?;
        mappings.insert(from_col, (ref_table, ref_col));
    }

    Ok(FkInfo { mappings })
}

/// Konvertiert &[&dyn ToSql] zu Vec<SqliteValue> für Manipulation
/// Nutzt einen Dummy-Query um die Parameter-Werte zu extrahieren
fn params_to_vec(
    params: &[&dyn ToSql],
    tx: &Transaction,
) -> Result<Vec<SqliteValue>, DatabaseError> {
    let mut values = Vec::new();

    // Erstelle eine Dummy-Query mit genau so vielen Platzhaltern wie wir Parameter haben
    // z.B. "SELECT ?, ?, ?"
    if params.is_empty() {
        return Ok(values);
    }

    let placeholders = vec!["?"; params.len()].join(", ");
    let dummy_sql = format!("SELECT {}", placeholders);

    let mut stmt = tx
        .prepare(&dummy_sql)
        .map_err(|e| DatabaseError::ExecutionError {
            sql: dummy_sql.clone(),
            reason: format!("Failed to prepare dummy query: {}", e),
            table: None,
        })?;

    // Führe die Query aus und extrahiere die Werte aus der Row
    let mut rows = stmt
        .query(params)
        .map_err(|e| DatabaseError::ExecutionError {
            sql: dummy_sql.clone(),
            reason: format!("Failed to execute dummy query: {}", e),
            table: None,
        })?;

    if let Some(row) = rows.next().map_err(|e| DatabaseError::ExecutionError {
        sql: dummy_sql,
        reason: format!("Failed to read dummy query result: {}", e),
        table: None,
    })? {
        // Extrahiere alle Spalten-Werte
        for i in 0..params.len() {
            let value: SqliteValue = row.get(i).map_err(|e| DatabaseError::ExecutionError {
                sql: format!("SELECT ..."),
                reason: format!("Failed to extract value at index {}: {}", i, e),
                table: None,
            })?;
            values.push(value);
        }
    }

    Ok(values)
}

/// Extrahiert PK-Werte aus den INSERT-Parametern
fn extract_pk_values_from_params(
    insert_stmt: &Insert,
    params: &[SqliteValue],
    pk_columns: &[String],
) -> Result<PkValues, DatabaseError> {
    let mut pk_values = PkValues::new();

    // Finde die Positionen der PK-Spalten in der INSERT column list
    for pk_col in pk_columns {
        if let Some(pos) = insert_stmt.columns.iter().position(|c| &c.value == pk_col) {
            // Hole den Parameter-Wert an dieser Position
            if pos < params.len() {
                // Konvertiere SqliteValue zu String
                let value_str = value_to_string(&params[pos]);
                pk_values.insert(pk_col.clone(), value_str);
            }
        }
    }

    Ok(pk_values)
}

/// Remapped FK-Werte in den Parametern basierend auf dem PK-Remapping Context
fn remap_fk_params(
    insert_stmt: &Insert,
    params: &mut Vec<SqliteValue>,
    fk_info: &FkInfo,
    pk_context: &PkRemappingContext,
) -> Result<(), DatabaseError> {
    // Für jede FK-Spalte: prüfe ob Remapping nötig ist
    for (col_name, (ref_table, ref_col)) in &fk_info.mappings {
        // Finde Position der FK-Spalte in der INSERT column list
        if let Some(pos) = insert_stmt
            .columns
            .iter()
            .position(|c| &c.value == col_name)
        {
            if pos < params.len() {
                // Hole aktuellen FK-Wert (als String)
                let current_value = value_to_string(&params[pos]);

                // Versuche zu remappen
                let new_value = pk_context.remap_fk_value(ref_table, ref_col, &current_value);

                if new_value != current_value {
                    // Ersetze den Parameter-Wert
                    params[pos] = SqliteValue::Text(new_value);
                    eprintln!(
                        "DEBUG: Remapped FK {}={} to {:?}",
                        col_name, current_value, params[pos]
                    );
                }
            }
        }
    }

    Ok(())
}

/// Hilfsfunktion: Konvertiert SqliteValue zu String für Vergleiche
fn value_to_string(value: &SqliteValue) -> String {
    match value {
        SqliteValue::Null => "NULL".to_string(),
        SqliteValue::Integer(i) => i.to_string(),
        SqliteValue::Real(r) => r.to_string(),
        SqliteValue::Text(s) => s.clone(),
        SqliteValue::Blob(b) => format!("BLOB({} bytes)", b.len()),
    }
}

/// Extrahiert PK-Werte aus einer RETURNING Row
fn extract_pk_values_from_row(
    row: &rusqlite::Row,
    pk_columns: &[String],
) -> Result<PkValues, DatabaseError> {
    let mut pk_values = PkValues::new();

    for pk_col in pk_columns.iter() {
        let value: String =
            row.get(pk_col.as_str())
                .map_err(|e| DatabaseError::ExecutionError {
                    sql: "RETURNING clause".to_string(),
                    reason: format!("Failed to extract PK column '{}': {}", pk_col, e),
                    table: None,
                })?;
        pk_values.insert(pk_col.clone(), value);
    }

    Ok(pk_values)
}
