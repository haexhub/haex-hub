use crate::crdt::insert_transformer::InsertTransformer;
use crate::crdt::query_transformer::QueryTransformer;
use crate::crdt::trigger::{HLC_TIMESTAMP_COLUMN, TOMBSTONE_COLUMN};
use crate::database::error::DatabaseError;
use crate::table_names::{TABLE_CRDT_CONFIGS, TABLE_CRDT_LOGS};
use sqlparser::ast::{
    Assignment, AssignmentTarget, BinaryOperator, ColumnDef, DataType, Expr, Ident,
    ObjectName, ObjectNamePart, Statement, TableFactor, TableObject,
    Value,
};
use std::borrow::Cow;
use std::collections::HashSet;
use uhlc::Timestamp;

/// Konfiguration für CRDT-Spalten
#[derive(Clone)]
struct CrdtColumns {
    tombstone: &'static str,
    hlc_timestamp: &'static str,
}

impl CrdtColumns {
    const DEFAULT: Self = Self {
        tombstone: TOMBSTONE_COLUMN,
        hlc_timestamp: HLC_TIMESTAMP_COLUMN,
    };

    /// Erstellt einen Tombstone-Filter für eine Tabelle
    fn create_tombstone_filter(&self, table_alias: Option<&str>) -> Expr {
        let column_expr = match table_alias {
            Some(alias) => {
                // Qualifizierte Referenz: alias.tombstone
                Expr::CompoundIdentifier(vec![Ident::new(alias), Ident::new(self.tombstone)])
            }
            None => {
                // Einfache Referenz: tombstone
                Expr::Identifier(Ident::new(self.tombstone))
            }
        };

        Expr::BinaryOp {
            left: Box::new(column_expr),
            op: BinaryOperator::NotEq,
            right: Box::new(Expr::Value(Value::Number("1".to_string(), false).into())),
        }
    }

    /// Erstellt eine Tombstone-Zuweisung für UPDATE/DELETE
    fn create_tombstone_assignment(&self) -> Assignment {
        Assignment {
            target: AssignmentTarget::ColumnName(ObjectName(vec![ObjectNamePart::Identifier(
                Ident::new(self.tombstone),
            )])),
            value: Expr::Value(Value::Number("1".to_string(), false).into()),
        }
    }

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
        if !columns.iter().any(|c| c.name.value == self.tombstone) {
            columns.push(ColumnDef {
                name: Ident::new(self.tombstone),
                data_type: DataType::Integer(None),
                options: vec![],
            });
        }
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

    pub fn transform_select_statement(&self, stmt: &mut Statement) -> Result<(), DatabaseError> {
        match stmt {
            Statement::Query(query) => {
                let query_transformer = QueryTransformer::new();
                query_transformer.transform_query_recursive(query, &self.excluded_tables)
            }
            // Fange alle anderen Fälle ab und gib einen Fehler zurück
            _ => Err(DatabaseError::UnsupportedStatement {
                sql: stmt.to_string(),
                reason: "This operation only accepts SELECT statements.".to_string(),
            }),
        }
    }

    /// Transformiert Statements MIT Zugriff auf Tabelleninformationen (empfohlen)
    pub fn transform_execute_statement_with_table_info(
        &self,
        stmt: &mut Statement,
        hlc_timestamp: &Timestamp,
        tx: &rusqlite::Transaction,
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
                        // Hole die Tabelleninformationen um PKs und FKs zu identifizieren
                        let table_name_str = self.normalize_table_name(name);

                        let columns = crate::crdt::trigger::get_table_schema(tx, &table_name_str)
                            .map_err(|e| DatabaseError::ExecutionError {
                            sql: format!("PRAGMA table_info('{}')", table_name_str),
                            reason: e.to_string(),
                            table: Some(table_name_str.to_string()),
                        })?;

                        let primary_keys: Vec<String> = columns
                            .iter()
                            .filter(|c| c.is_pk)
                            .map(|c| c.name.clone())
                            .collect();

                        let foreign_keys = crate::crdt::trigger::get_foreign_key_columns(tx, &table_name_str)
                            .map_err(|e| DatabaseError::ExecutionError {
                            sql: format!("PRAGMA foreign_key_list('{}')", table_name_str),
                            reason: e.to_string(),
                            table: Some(table_name_str.to_string()),
                        })?;

                        let insert_transformer = InsertTransformer::new();
                        insert_transformer.transform_insert(insert_stmt, hlc_timestamp, &primary_keys, &foreign_keys)?;
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
            Statement::Delete(del_stmt) => {
                if let Some(table_name) = self.extract_table_name_from_delete(del_stmt) {
                    let table_name_str = self.normalize_table_name(&table_name);
                    let is_crdt = self.is_crdt_sync_table(&table_name);
                    eprintln!("DEBUG DELETE (with_table_info): table='{}', is_crdt_sync={}, normalized='{}'",
                              table_name, is_crdt, table_name_str);
                    if is_crdt {
                        eprintln!("DEBUG: Transforming DELETE to UPDATE for table '{}'", table_name_str);
                        self.transform_delete_to_update(stmt, hlc_timestamp)?;
                    }
                    Ok(None)
                } else {
                    Err(DatabaseError::UnsupportedStatement {
                        sql: del_stmt.to_string(),
                        reason: "DELETE from non-table source or multiple tables".to_string(),
                    })
                }
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
        // Für INSERT-Statements ohne Connection nutzen wir eine leere PK-Liste
        // Das bedeutet ALLE Spalten werden im ON CONFLICT UPDATE gesetzt
        // Dies ist ein Fallback für den Fall, dass keine Connection verfügbar ist
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
                        // Ohne Connection: leere PK- und FK-Listen (alle Spalten werden upgedatet)
                        let insert_transformer = InsertTransformer::new();
                        insert_transformer.transform_insert(insert_stmt, hlc_timestamp, &[], &[])?;
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
            Statement::Delete(del_stmt) => {
                if let Some(table_name) = self.extract_table_name_from_delete(del_stmt) {
                    if self.is_crdt_sync_table(&table_name) {
                        self.transform_delete_to_update(stmt, hlc_timestamp)?;
                    }
                    Ok(None)
                } else {
                    Err(DatabaseError::UnsupportedStatement {
                        sql: del_stmt.to_string(),
                        reason: "DELETE from non-table source or multiple tables".to_string(),
                    })
                }
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


    /// Transformiert DELETE zu UPDATE (soft delete)
    fn transform_delete_to_update(
        &self,
        stmt: &mut Statement,
        timestamp: &Timestamp,
    ) -> Result<(), DatabaseError> {
        if let Statement::Delete(del_stmt) = stmt {
            let table_to_update = match &del_stmt.from {
                sqlparser::ast::FromTable::WithFromKeyword(from)
                | sqlparser::ast::FromTable::WithoutKeyword(from) => {
                    if from.len() == 1 {
                        from[0].clone()
                    } else {
                        return Err(DatabaseError::UnsupportedStatement {
                            reason: "DELETE with multiple tables not supported".to_string(),
                            sql: stmt.to_string(),
                        });
                    }
                }
            };

            let assignments = vec![
                self.columns.create_tombstone_assignment(),
                self.columns.create_hlc_assignment(timestamp),
            ];

            *stmt = Statement::Update {
                table: table_to_update,
                assignments,
                from: None,
                selection: del_stmt.selection.clone(),
                returning: None,
                or: None,
                limit: None,
            };
        }
        Ok(())
    }

    /// Extrahiert Tabellennamen aus DELETE-Statement
    fn extract_table_name_from_delete(
        &self,
        del_stmt: &sqlparser::ast::Delete,
    ) -> Option<ObjectName> {
        let tables = match &del_stmt.from {
            sqlparser::ast::FromTable::WithFromKeyword(from)
            | sqlparser::ast::FromTable::WithoutKeyword(from) => from,
        };

        if tables.len() == 1 {
            if let TableFactor::Table { name, .. } = &tables[0].relation {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
