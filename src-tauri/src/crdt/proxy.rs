// In src-tauri/src/crdt/proxy.rs
use crate::crdt::hlc::HlcService;
use crate::crdt::trigger::{HLC_TIMESTAMP_COLUMN, TOMBSTONE_COLUMN};
use crate::table_names::{TABLE_CRDT_CONFIGS, TABLE_CRDT_LOGS};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlparser::ast::{
    Assignment, AssignmentTarget, BinaryOperator, ColumnDef, DataType, Expr, Ident, Insert,
    ObjectName, ObjectNamePart, SelectItem, SetExpr, Statement, TableFactor, TableObject,
    TableWithJoins, UpdateTableFromKind, Value, ValueWithSpan,
};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use ts_rs::TS;
use uhlc::Timestamp;
pub struct DbConnection(pub Arc<Mutex<Option<Connection>>>);

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "details")]
pub enum ProxyError {
    /// Der SQL-Code konnte nicht geparst werden.
    ParseError {
        reason: String,
    },
    /// Ein Fehler ist während der Ausführung in der Datenbank aufgetreten.
    ExecutionError {
        sql: String,
        reason: String,
    },
    /// Ein Fehler ist beim Verwalten der Transaktion aufgetreten.
    TransactionError {
        reason: String,
    },
    /// Ein SQL-Statement wird vom Proxy nicht unterstützt (z.B. DELETE von einer Subquery).
    UnsupportedStatement {
        description: String,
    },
    HlcError {
        reason: String,
    },
}

// Tabellen, die von der Proxy-Logik ausgeschlossen sind.
const EXCLUDED_TABLES: &[&str] = &[TABLE_CRDT_CONFIGS, TABLE_CRDT_LOGS];

pub struct SqlProxy;

impl SqlProxy {
    pub fn new() -> Self {
        Self {}
    }

    /// Führt SQL-Anweisungen aus, nachdem sie für CRDT-Konformität transformiert wurden.
    pub fn execute(
        &self,
        sql: &str,
        params: Vec<JsonValue>,
        state: State<'_, DbConnection>,
        hlc_service: &HlcService,
    ) -> Result<Vec<String>, ProxyError> {
        let dialect = SQLiteDialect {};
        let mut ast_vec = Parser::parse_sql(&dialect, sql).map_err(|e| ProxyError::ParseError {
            reason: e.to_string(),
        })?;

        let mut modified_schema_tables = HashSet::new();

        let db_lock = state
            .0
            .lock()
            .map_err(|e| format!("Mutex Lock Fehler: {}", e))?;
        let conn = db_lock.as_ref().ok_or("Keine Datenbankverbindung")?;

        let tx = conn
            .transaction()
            .map_err(|e| ProxyError::TransactionError {
                reason: e.to_string(),
            })?;

        /* let hlc_timestamp =
        hlc_service
            .new_timestamp_and_persist(&tx)
            .map_err(|e| ProxyError::HlcError {
                reason: e.to_string(),
            })?; */

        for statement in &mut ast_vec {
            if let Some(table_name) = self.transform_statement(statement)? {
                modified_schema_tables.insert(table_name);
            }
        }

        for statement in ast_vec {
            let final_sql = statement.to_string();
            tx.execute(&final_sql, [])
                .map_err(|e| ProxyError::ExecutionError {
                    sql: final_sql,
                    reason: e.to_string(),
                })?;
        }
        tx.commit().map_err(|e| ProxyError::TransactionError {
            reason: e.to_string(),
        })?;

        Ok(modified_schema_tables.into_iter().collect())
    }

    /// Wendet die Transformation auf ein einzelnes Statement an.
    fn transform_statement(&self, stmt: &mut Statement) -> Result<Option<String>, ProxyError> {
        match stmt {
            Statement::Query(query) => {
                if let SetExpr::Select(select) = &mut *query.body {
                    let mut tombstone_filters = Vec::new();

                    for twj in &select.from {
                        if let TableFactor::Table { name, alias, .. } = &twj.relation {
                            if self.is_audited_table(name) {
                                let table_idents = if let Some(a) = alias {
                                    vec![a.name.clone()]
                                } else {
                                    name.0
                                        .iter()
                                        .filter_map(|part| match part {
                                            ObjectNamePart::Identifier(id) => Some(id.clone()),
                                            _ => None,
                                        })
                                        .collect::<Vec<_>>()
                                };
                                let column_ident = Ident::new(TOMBSTONE_COLUMN);
                                let full_ident = [table_idents, vec![column_ident]].concat();
                                let filter = Expr::BinaryOp {
                                    left: Box::new(Expr::CompoundIdentifier(full_ident)),
                                    op: BinaryOperator::Eq,
                                    right: Box::new(Expr::Value(
                                        sqlparser::ast::Value::Number("1".to_string(), false)
                                            .into(),
                                    )),
                                };
                                tombstone_filters.push(filter);
                            }
                        }
                    }
                    if !tombstone_filters.is_empty() {
                        let combined_filter = tombstone_filters
                            .into_iter()
                            .reduce(|acc, expr| Expr::BinaryOp {
                                left: Box::new(acc),
                                op: BinaryOperator::And,
                                right: Box::new(expr),
                            })
                            .unwrap();
                        match &mut select.selection {
                            Some(existing) => {
                                *existing = Expr::BinaryOp {
                                    left: Box::new(existing.clone()),
                                    op: BinaryOperator::And,
                                    right: Box::new(combined_filter),
                                };
                            }
                            None => {
                                select.selection = Some(combined_filter);
                            }
                        }
                    }
                }

                // TODO: UNION, EXCEPT etc. werden hier nicht behandelt
            }

            Statement::CreateTable(create_table) => {
                if self.is_audited_table(&create_table.name) {
                    self.add_crdt_columns(&mut create_table.columns);
                    return Ok(Some(
                        create_table
                            .name
                            .to_string()
                            .trim_matches('`')
                            .trim_matches('"')
                            .to_string(),
                    ));
                }
            }

            Statement::Insert(insert_stmt) => {
                if let TableObject::TableName(name) = &insert_stmt.table {
                    if self.is_audited_table(name) {
                        self.add_hlc_to_insert(insert_stmt);
                    }
                }
            }

            Statement::Update {
                table,
                assignments,
                from,
                selection,
                returning,
                or,
            } => {
                if let TableFactor::Table { name, .. } = &table.relation {
                    if self.is_audited_table(&name) {
                        if let Some(ts) = hlc_timestamp {
                            assignments.push(self.create_hlc_assignment(ts));
                        }
                    }
                }
                *stmt = Statement::Update {
                    table: table.clone(),
                    assignments: assignments.clone(),
                    from: from.clone(),
                    selection: selection.clone(),
                    returning: returning.clone(),
                    or: *or,
                };
            }

            Statement::Delete(del_stmt) => {
                let table_name = self.extract_table_name_from_from(&del_stmt.from);
                if let Some(name) = table_name {
                    if self.is_audited_table(&name) {
                        // GEÄNDERT: Übergibt den Zeitstempel an die Transformationsfunktion

                        self.transform_delete_to_update(stmt);
                    }
                } else {
                    return Err(ProxyError::UnsupportedStatement {
                        description: "DELETE from non-table source or multiple tables".to_string(),
                    });
                }
            }

            Statement::AlterTable { name, .. } => {
                if self.is_audited_table(name) {
                    return Ok(Some(
                        name.to_string()
                            .trim_matches('`')
                            .trim_matches('"')
                            .to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Fügt die Tombstone-Spalte zu einer Liste von Spaltendefinitionen hinzu.
    fn add_tombstone_column(&self, columns: &mut Vec<ColumnDef>) {
        if !columns
            .iter()
            .any(|c| c.name.value.to_lowercase() == TOMBSTONE_COLUMN)
        {
            columns.push(ColumnDef {
                name: Ident::new(TOMBSTONE_COLUMN),
                data_type: DataType::Integer(None),
                options: vec![],
            });
        }
    }

    /// Prüft, ob eine Tabelle von der Proxy-Logik betroffen sein soll.
    fn is_audited_table(&self, name: &ObjectName) -> bool {
        let table_name = name.to_string().to_lowercase();
        let table_name = table_name.trim_matches('`').trim_matches('"');
        !EXCLUDED_TABLES.contains(&table_name)
    }

    fn extract_table_name_from_from(&self, from: &sqlparser::ast::FromTable) -> Option<ObjectName> {
        let tables = match from {
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

    fn extract_table_name(&self, from: &[TableWithJoins]) -> Option<ObjectName> {
        if from.len() == 1 {
            if let TableFactor::Table { name, .. } = &from[0].relation {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn create_tombstone_assignment(&self) -> Assignment {
        Assignment {
            target: AssignmentTarget::ColumnName(ObjectName(vec![ObjectNamePart::Identifier(
                Ident::new(TOMBSTONE_COLUMN),
            )])),
            value: Expr::Value(sqlparser::ast::Value::Number("1".to_string(), false).into()),
        }
    }

    fn add_tombstone_filter(&self, selection: &mut Option<Expr>) {
        let tombstone_expr = Expr::BinaryOp {
            left: Box::new(Expr::Identifier(Ident::new(TOMBSTONE_COLUMN))),
            op: BinaryOperator::Eq,
            // HIER IST DIE FINALE KORREKTUR:
            right: Box::new(Expr::Value(Value::Number("0".to_string(), false).into())),
        };

        match selection {
            Some(existing) => {
                // Kombiniere mit AND, wenn eine WHERE-Klausel existiert
                *selection = Some(Expr::BinaryOp {
                    left: Box::new(existing.clone()),
                    op: BinaryOperator::And,
                    right: Box::new(tombstone_expr),
                });
            }
            None => {
                // Setze neue WHERE-Klausel, wenn keine existiert
                *selection = Some(tombstone_expr);
            }
        }
    }

    fn add_crdt_columns(&self, columns: &mut Vec<ColumnDef>) {
        if !columns.iter().any(|c| c.name.value == TOMBSTONE_COLUMN) {
            columns.push(ColumnDef {
                name: Ident::new(TOMBSTONE_COLUMN),
                data_type: DataType::Integer(None),
                options: vec![],
            });
        }
        if !columns.iter().any(|c| c.name.value == HLC_TIMESTAMP_COLUMN) {
            columns.push(ColumnDef {
                name: Ident::new(HLC_TIMESTAMP_COLUMN),
                data_type: DataType::String(None),
                options: vec![],
            });
        }
    }

    fn transform_delete_to_update(&self, stmt: &mut Statement) {
        if let Statement::Delete(del_stmt) = stmt {
            let table_to_update = match &del_stmt.from {
                sqlparser::ast::FromTable::WithFromKeyword(from)
                | sqlparser::ast::FromTable::WithoutKeyword(from) => from[0].clone(),
            };

            let assignments = vec![self.create_tombstone_assignment()];

            *stmt = Statement::Update {
                table: table_to_update,
                assignments,
                from: None,
                selection: del_stmt.selection.clone(),
                returning: None,
                or: None,
            };
        }
    }

    fn add_hlc_to_insert(
        &self,
        insert_stmt: &mut sqlparser::ast::Insert,
        ts: &Timestamp,
    ) -> Result<(), ProxyError> {
        insert_stmt.columns.push(Ident::new(HLC_TIMESTAMP_COLUMN));

        match insert_stmt.source.as_mut() {
            Some(query) => match &mut *query.body {
                // Dereferenziere die Box mit *
                SetExpr::Values(values) => {
                    for row in &mut values.rows {
                        row.push(Expr::Value(
                            Value::SingleQuotedString(ts.to_string()).into(),
                        ));
                    }
                }
                SetExpr::Select(select) => {
                    let hlc_expr = Expr::Value(Value::SingleQuotedString(ts.to_string()).into());
                    select.projection.push(SelectItem::UnnamedExpr(hlc_expr));
                }
                _ => {
                    return Err(ProxyError::UnsupportedStatement {
                        description: "INSERT with unsupported source".to_string(),
                    });
                }
            },
            None => {
                return Err(ProxyError::UnsupportedStatement {
                    description: "INSERT statement has no source".to_string(),
                });
            }
        }
        Ok(())
    }
    /// Erstellt eine Zuweisung `haex_modified_hlc = '...'`
    // NEU: Hilfsfunktion
    fn create_hlc_assignment(&self, ts: &Timestamp) -> Assignment {
        Assignment {
            target: AssignmentTarget::ColumnName(ObjectName(vec![ObjectNamePart::Identifier(
                Ident::new(HLC_TIMESTAMP_COLUMN),
            )])),
            value: Expr::Value(Value::SingleQuotedString(ts.to_string()).into()),
        }
    }
}
