// src-tauri/src/crdt/insert_transformer.rs
// INSERT-spezifische CRDT-Transformationen (ON CONFLICT, RETURNING)

use crate::crdt::trigger::{HLC_TIMESTAMP_COLUMN, TOMBSTONE_COLUMN};
use crate::database::error::DatabaseError;
use sqlparser::ast::{
    Assignment, AssignmentTarget, BinaryOperator, Expr, Ident, Insert, ObjectNamePart,
    OnConflict, OnConflictAction, OnInsert, SelectItem, SetExpr, Value,
};
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

    /// Transformiert INSERT-Statements (fügt HLC-Timestamp hinzu und behandelt Tombstone-Konflikte)
    /// Fügt automatisch RETURNING für Primary Keys hinzu, damit der Executor die tatsächlichen PKs kennt
    pub fn transform_insert(
        &self,
        insert_stmt: &mut Insert,
        timestamp: &Timestamp,
        primary_keys: &[String],
        foreign_keys: &[String],
    ) -> Result<(), DatabaseError> {
        // Add both haex_timestamp and haex_tombstone columns
        insert_stmt
            .columns
            .push(Ident::new(self.hlc_timestamp_column));
        insert_stmt.columns.push(Ident::new(self.tombstone_column));

        // Füge RETURNING für alle Primary Keys hinzu (falls noch nicht vorhanden)
        // Dies erlaubt uns, die tatsächlichen PK-Werte nach ON CONFLICT zu kennen
        if insert_stmt.returning.is_none() && !primary_keys.is_empty() {
            insert_stmt.returning = Some(
                primary_keys
                    .iter()
                    .map(|pk| SelectItem::UnnamedExpr(Expr::Identifier(Ident::new(pk))))
                    .collect(),
            );
        }

        // Setze ON CONFLICT für UPSERT-Verhalten bei Tombstone-Einträgen
        // Dies ermöglicht das Wiederverwenden von gelöschten Einträgen
        if insert_stmt.on.is_none() {
            // ON CONFLICT DO UPDATE SET ...
            // Aktualisiere alle Spalten außer CRDT-Spalten, wenn ein Konflikt auftritt

            // Erstelle UPDATE-Assignments für alle Spalten außer CRDT-Spalten, Primary Keys und Foreign Keys
            let mut assignments = Vec::new();
            for column in insert_stmt.columns.iter() {
                let col_name = &column.value;

                // Überspringe CRDT-Spalten
                if col_name == self.hlc_timestamp_column || col_name == self.tombstone_column {
                    continue;
                }

                // Überspringe Primary Key Spalten um FOREIGN KEY Konflikte zu vermeiden
                if primary_keys.contains(col_name) {
                    continue;
                }

                // Überspringe Foreign Key Spalten um FOREIGN KEY Konflikte zu vermeiden
                // Wenn eine FK auf eine neue ID verweist, die noch nicht existiert, schlägt der Constraint fehl
                if foreign_keys.contains(col_name) {
                    continue;
                }

                // excluded.column_name referenziert die neuen Werte aus dem INSERT
                assignments.push(Assignment {
                    target: AssignmentTarget::ColumnName(sqlparser::ast::ObjectName(vec![
                        ObjectNamePart::Identifier(column.clone()),
                    ])),
                    value: Expr::CompoundIdentifier(vec![Ident::new("excluded"), column.clone()]),
                });
            }

            // Füge HLC-Timestamp Update hinzu (mit dem übergebenen timestamp)
            assignments.push(Assignment {
                target: AssignmentTarget::ColumnName(sqlparser::ast::ObjectName(vec![ObjectNamePart::Identifier(
                    Ident::new(self.hlc_timestamp_column),
                )])),
                value: Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into()),
            });

            // Setze Tombstone auf 0 (reaktiviere den Eintrag)
            assignments.push(Assignment {
                target: AssignmentTarget::ColumnName(sqlparser::ast::ObjectName(vec![ObjectNamePart::Identifier(
                    Ident::new(self.tombstone_column),
                )])),
                value: Expr::Value(Value::Number("0".to_string(), false).into()),
            });

            // ON CONFLICT nur wenn Tombstone = 1 (Eintrag wurde gelöscht)
            // Ansonsten soll der INSERT fehlschlagen (UNIQUE constraint error)
            let tombstone_condition = Expr::BinaryOp {
                left: Box::new(Expr::Identifier(Ident::new(self.tombstone_column))),
                op: BinaryOperator::Eq,
                right: Box::new(Expr::Value(Value::Number("1".to_string(), false).into())),
            };

            insert_stmt.on = Some(OnInsert::OnConflict(OnConflict {
                conflict_target: None, // Wird auf alle UNIQUE Constraints angewendet
                action: OnConflictAction::DoUpdate(sqlparser::ast::DoUpdate {
                    assignments,
                    selection: Some(tombstone_condition),
                }),
            }));
        }

        match insert_stmt.source.as_mut() {
            Some(query) => match &mut *query.body {
                SetExpr::Values(values) => {
                    for row in &mut values.rows {
                        // Add haex_timestamp value
                        row.push(Expr::Value(
                            Value::SingleQuotedString(timestamp.to_string()).into(),
                        ));
                        // Add haex_tombstone value (0 = not deleted)
                        row.push(Expr::Value(Value::Number("0".to_string(), false).into()));
                    }
                }
                SetExpr::Select(select) => {
                    let hlc_expr =
                        Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into());
                    select.projection.push(SelectItem::UnnamedExpr(hlc_expr));
                    // Add haex_tombstone value (0 = not deleted)
                    let tombstone_expr = Expr::Value(Value::Number("0".to_string(), false).into());
                    select
                        .projection
                        .push(SelectItem::UnnamedExpr(tombstone_expr));
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
