// src-tauri/src/crdt/transformer.rs

use crate::crdt::insert_transformer::InsertTransformer;
use crate::crdt::trigger::HLC_TIMESTAMP_COLUMN;
use crate::database::error::DatabaseError;
use crate::table_names::{TABLE_CRDT_CONFIGS, TABLE_CRDT_LOGS};
use sqlparser::ast::{
    Assignment, AssignmentTarget, ColumnDef, DataType, Expr, Ident, ObjectName, ObjectNamePart,
    Statement, TableFactor, TableObject, Value,
};
use std::borrow::Cow;
use std::collections::HashSet;
use uhlc::Timestamp;

/// Konfiguration für CRDT-Spalten
#[derive(Clone)]
struct CrdtColumns {
    hlc_timestamp: &'static str,
}

impl CrdtColumns {
    const DEFAULT: Self = Self {
        hlc_timestamp: HLC_TIMESTAMP_COLUMN,
    };

    /// Erstellt eine HLC-Zuweisung für UPDATE/DELETE
    fn create_hlc_assignment(&self, timestamp: &Timestamp) -> Assignment {
        Assignment {
            target: AssignmentTarget::ColumnName(ObjectName(vec![ObjectNamePart::Identifier(
                Ident::new(self.hlc_timestamp),
            )])),
            value: Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into()),
        }
    }

    /// Fügt CRDT-Spalten zu einer Tabellendefinition hinzu
    fn add_to_table_definition(&self, columns: &mut Vec<ColumnDef>) {
        if !columns.iter().any(|c| c.name.value == self.hlc_timestamp) {
            columns.push(ColumnDef {
                name: Ident::new(self.hlc_timestamp),
                data_type: DataType::String(None),
                options: vec![],
            });
        }
    }
}

pub struct CrdtTransformer {
    columns: CrdtColumns,
    excluded_tables: HashSet<&'static str>,
}

impl CrdtTransformer {
    pub fn new() -> Self {
        let mut excluded_tables = HashSet::new();
        excluded_tables.insert(TABLE_CRDT_CONFIGS);
        excluded_tables.insert(TABLE_CRDT_LOGS);

        Self {
            columns: CrdtColumns::DEFAULT,
            excluded_tables,
        }
    }

    /// Prüft, ob eine Tabelle CRDT-Synchronisation unterstützen soll
    fn is_crdt_sync_table(&self, name: &ObjectName) -> bool {
        let table_name = self.normalize_table_name(name);
        !self.excluded_tables.contains(table_name.as_ref())
    }

    /// Normalisiert Tabellennamen (entfernt Anführungszeichen)
    fn normalize_table_name(&self, name: &ObjectName) -> Cow<str> {
        let name_str = name.to_string().to_lowercase();
        Cow::Owned(name_str.trim_matches('`').trim_matches('"').to_string())
    }

    // =================================================================
    // ÖFFENTLICHE API-METHODEN
    // =================================================================

    pub fn transform_execute_statement_with_table_info(
        &self,
        stmt: &mut Statement,
        hlc_timestamp: &Timestamp,
    ) -> Result<Option<String>, DatabaseError> {
        match stmt {
            Statement::CreateTable(create_table) => {
                if self.is_crdt_sync_table(&create_table.name) {
                    self.columns
                        .add_to_table_definition(&mut create_table.columns);
                    Ok(Some(
                        self.normalize_table_name(&create_table.name).into_owned(),
                    ))
                } else {
                    Ok(None)
                }
            }
            Statement::Insert(insert_stmt) => {
                if let TableObject::TableName(name) = &insert_stmt.table {
                    if self.is_crdt_sync_table(name) {
                        // Hard Delete: Kein Schema-Lookup mehr nötig (kein ON CONFLICT)
                        let insert_transformer = InsertTransformer::new();
                        insert_transformer.transform_insert(insert_stmt, hlc_timestamp)?;
                    }
                }
                Ok(None)
            }
            Statement::Update {
                table, assignments, ..
            } => {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if self.is_crdt_sync_table(name) {
                        assignments.push(self.columns.create_hlc_assignment(hlc_timestamp));
                    }
                }
                Ok(None)
            }
            Statement::Delete(_del_stmt) => {
                // Hard Delete - keine Transformation!
                // DELETE bleibt DELETE
                // BEFORE DELETE Trigger schreiben die Logs
                Ok(None)
            }
            Statement::AlterTable { name, .. } => {
                if self.is_crdt_sync_table(name) {
                    Ok(Some(self.normalize_table_name(name).into_owned()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub fn transform_execute_statement(
        &self,
        stmt: &mut Statement,
        hlc_timestamp: &Timestamp,
    ) -> Result<Option<String>, DatabaseError> {
        match stmt {
            Statement::CreateTable(create_table) => {
                if self.is_crdt_sync_table(&create_table.name) {
                    self.columns
                        .add_to_table_definition(&mut create_table.columns);
                    Ok(Some(
                        self.normalize_table_name(&create_table.name).into_owned(),
                    ))
                } else {
                    Ok(None)
                }
            }
            Statement::Insert(insert_stmt) => {
                if let TableObject::TableName(name) = &insert_stmt.table {
                    if self.is_crdt_sync_table(name) {
                        // Hard Delete: Keine ON CONFLICT Logik mehr nötig
                        let insert_transformer = InsertTransformer::new();
                        insert_transformer.transform_insert(insert_stmt, hlc_timestamp)?;
                    }
                }
                Ok(None)
            }
            Statement::Update {
                table, assignments, ..
            } => {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if self.is_crdt_sync_table(name) {
                        assignments.push(self.columns.create_hlc_assignment(hlc_timestamp));
                    }
                }
                Ok(None)
            }
            Statement::Delete(_del_stmt) => {
                // Hard Delete - keine Transformation!
                // DELETE bleibt DELETE
                Ok(None)
            }
            Statement::AlterTable { name, .. } => {
                if self.is_crdt_sync_table(name) {
                    Ok(Some(self.normalize_table_name(name).into_owned()))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
}
