// src-tauri/src/crdt/insert_transformer.rs
// INSERT-spezifische CRDT-Transformationen (ON CONFLICT, RETURNING)

use crate::crdt::trigger::{HLC_TIMESTAMP_COLUMN, TOMBSTONE_COLUMN};
use crate::database::error::DatabaseError;
use sqlparser::ast::{Expr, Ident, Insert, SelectItem, SetExpr, Value};
use uhlc::Timestamp;

/// Helper-Struct für INSERT-Transformationen
pub struct InsertTransformer {
    tombstone_column: &'static str,
    hlc_timestamp_column: &'static str,
}

impl InsertTransformer {
    pub fn new() -> Self {
        Self {
            tombstone_column: TOMBSTONE_COLUMN,
            hlc_timestamp_column: HLC_TIMESTAMP_COLUMN,
        }
    }

    fn find_or_add_column(columns: &mut Vec<Ident>, col_name: &'static str) -> usize {
        match columns.iter().position(|c| c.value == col_name) {
            Some(index) => index, // Gefunden! Gib Index zurück.
            None => {
                // Nicht gefunden! Hinzufügen.
                columns.push(Ident::new(col_name));
                columns.len() - 1 // Der Index des gerade hinzugefügten Elements
            }
        }
    }

    /// Wenn der Index == der Länge ist, wird der Wert stattdessen gepusht.
    fn set_or_push_value(row: &mut Vec<Expr>, index: usize, value: Expr) {
        if index < row.len() {
            // Spalte war vorhanden, Wert (wahrscheinlich `?` oder NULL) ersetzen
            row[index] = value;
        } else {
            // Spalte war nicht vorhanden, Wert hinzufügen
            row.push(value);
        }
    }

    fn set_or_push_projection(projection: &mut Vec<SelectItem>, index: usize, value: Expr) {
        let item = SelectItem::UnnamedExpr(value);
        if index < projection.len() {
            projection[index] = item;
        } else {
            projection.push(item);
        }
    }

    /// Transformiert INSERT-Statements (fügt HLC-Timestamp hinzu)
    /// Hard Delete: Kein ON CONFLICT mehr nötig - gelöschte Einträge sind wirklich weg
    pub fn transform_insert(
        &self,
        insert_stmt: &mut Insert,
        timestamp: &Timestamp,
    ) -> Result<(), DatabaseError> {
        // Add both haex_timestamp and haex_tombstone columns if not exists
        let hlc_col_index =
            Self::find_or_add_column(&mut insert_stmt.columns, self.hlc_timestamp_column);
        let tombstone_col_index =
            Self::find_or_add_column(&mut insert_stmt.columns, self.tombstone_column);

        // ON CONFLICT Logik komplett entfernt!
        // Bei Hard Deletes gibt es keine Tombstone-Einträge mehr zu reaktivieren
        // UNIQUE Constraint Violations sind echte Fehler

        match insert_stmt.source.as_mut() {
            Some(query) => match &mut *query.body {
                SetExpr::Values(values) => {
                    for row in &mut values.rows {
                        let hlc_value =
                            Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into());
                        let tombstone_value =
                            Expr::Value(Value::Number("0".to_string(), false).into());

                        Self::set_or_push_value(row, hlc_col_index, hlc_value);
                        Self::set_or_push_value(row, tombstone_col_index, tombstone_value);
                    }
                }
                SetExpr::Select(select) => {
                    let hlc_value =
                        Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into());
                    let tombstone_value = Expr::Value(Value::Number("0".to_string(), false).into());

                    Self::set_or_push_projection(&mut select.projection, hlc_col_index, hlc_value);
                    Self::set_or_push_projection(
                        &mut select.projection,
                        tombstone_col_index,
                        tombstone_value,
                    );
                }
                _ => {
                    return Err(DatabaseError::UnsupportedStatement {
                        sql: insert_stmt.to_string(),
                        reason: "INSERT with unsupported source type".to_string(),
                    });
                }
            },
            None => {
                return Err(DatabaseError::UnsupportedStatement {
                    reason: "INSERT statement has no source".to_string(),
                    sql: insert_stmt.to_string(),
                });
            }
        }
        Ok(())
    }
}
