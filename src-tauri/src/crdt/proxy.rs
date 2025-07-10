// In src-tauri/src/sql_proxy.rs

use rusqlite::Connection;
use sqlparser::ast::Statement;
use sqlparser::ast::{ColumnDef, DataType, Expr, Ident, Query, Statement, TableWithJoins, Value};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;
use sqlparser::visit_mut::{self, VisitorMut};
use std::ops::ControlFlow;

// Der Name der Tombstone-Spalte als Konstante, um "Magic Strings" zu vermeiden.
pub const TOMBSTONE_COLUMN_NAME: &str = "tombstone";
const EXCLUDED_TABLES: &[&str] = &["crdt_log"];

// Die Hauptstruktur unseres Proxys.
// Sie ist zustandslos, da wir uns gegen einen Schema-Cache entschieden haben.
pub struct SqlProxy;

impl SqlProxy {
    pub fn new() -> Self {
        Self {}
    }

    // Die zentrale Ausführungsfunktion
    pub fn execute(&self, sql: &str, conn: &Connection) -> Result<(), String> {
        // 1. Parsen des SQL-Strings in einen oder mehrere ASTs.
        // Ein String kann mehrere, durch Semikolon getrennte Anweisungen enthalten.
        let dialect = SQLiteDialect {};
        let mut ast_vec =
            Parser::parse_sql(&dialect, sql).map_err(|e| format!("SQL-Parse-Fehler: {}", e))?;

        // 2. Wir durchlaufen und transformieren jedes einzelne Statement im AST-Vektor.
        for statement in &mut ast_vec {
            self.transform_statement(statement)?;
        }

        // 3. Ausführen der (möglicherweise modifizierten) Anweisungen in einer einzigen Transaktion.
        // Dies stellt sicher, dass alle Operationen atomar sind.
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for statement in ast_vec {
            let final_sql = statement.to_string();
            tx.execute(&final_sql)
                .map_err(|e| format!("DB-Ausführungsfehler bei '{}': {}", final_sql, e))?;

            // Wenn es ein CREATE/ALTER TABLE war, müssen die Trigger neu erstellt werden.
            // Dies geschieht innerhalb derselben Transaktion.
            if let Statement::CreateTable { name, .. } | Statement::AlterTable { name, .. } =
                statement
            {
                let table_name = name.0.last().unwrap().value.clone();
                let trigger_manager = crate::trigger_manager::TriggerManager::new(&tx);
                trigger_manager
                    .setup_triggers_for_table(&table_name)
                    .map_err(|e| {
                        format!("Trigger-Setup-Fehler für Tabelle '{}': {}", table_name, e)
                    })?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }

    // Diese Methode wendet die Transformation auf ein einzelnes Statement an.
    fn transform_statement(&self, statement: &mut Statement) -> Result<(), String> {
        let mut visitor = TombstoneVisitor;
        // `visit` durchläuft den AST und ruft die entsprechenden `visit_*_mut` Methoden auf.
        statement.visit(&mut visitor);
        Ok(())
    }
}

struct TombstoneVisitor;

impl TombstoneVisitor {
    fn is_audited_table(&self, table_name: &str) -> bool {
        !EXCLUDED_TABLES.contains(&table_name.to_lowercase().as_str())
    }
}

impl VisitorMut for TombstoneVisitor {
    type Break = ();

    // Diese Methode wird für jedes Statement im AST aufgerufen
    fn visit_statement_mut(&mut self, stmt: &mut Statement) -> ControlFlow<Self::Break> {
        match stmt {
            // Fall 1: CREATE TABLE
            Statement::CreateTable { name, columns, .. } => {
                let table_name = name.0.last().unwrap().value.as_str();
                if self.is_audited_table(table_name) {
                    // Füge die 'tombstone'-Spalte hinzu, wenn sie nicht existiert
                    if !columns
                        .iter()
                        .any(|c| c.name.value.to_lowercase() == TOMBSTONE_COLUMN_NAME)
                    {
                        columns.push(ColumnDef {
                            name: Ident::new(TOMBSTONE_COLUMN_NAME),
                            data_type: DataType::Integer,
                            collation: None,
                            options: vec![], // Default ist 0
                        });
                    }
                }
            }

            // Fall 2: DELETE
            Statement::Delete(del_stmt) => {
                // Wandle das DELETE-Statement in ein UPDATE-Statement um
                let new_update = Statement::Update {
                    table: del_stmt.from.clone(),
                    assignments: vec![],
                    value: Box::new(Expr::Value(Value::Number("1".to_string(), false))),
                    from: None,
                    selection: del_stmt.selection.clone(),
                    returning: None,
                };
                // Ersetze das aktuelle Statement im AST
                *stmt = new_update;
            }
            _ => {}
        }

        // Setze die Traversierung für untergeordnete Knoten fort (z.B. SELECTs)
        visit_mut::walk_statement_mut(self, stmt)
    }

    // Diese Methode wird für jede Query (auch Subqueries) aufgerufen
    fn visit_query_mut(&mut self, query: &mut Query) -> ControlFlow<Self::Break> {
        // Zuerst rekursiv in die Tiefe gehen, um innere Queries zuerst zu bearbeiten
        visit_mut::walk_query_mut(self, query);

        // Dann die WHERE-Klausel der aktuellen Query anpassen
        if let Some(from_clause) = query.body.as_select_mut().map(|s| &mut s.from) {
            // (Hier würde eine komplexere Logik zur Analyse der Joins und Tabellen stehen)
            // Vereinfacht nehmen wir an, wir fügen es für die erste Tabelle hinzu.
            let table_name = if let Some(relation) = from_clause.get_mut(0) {
                // Diese Logik muss verfeinert werden, um Aliase etc. zu behandeln
                relation.relation.to_string()
            } else {
                "".to_string()
            };

            if self.is_audited_table(&table_name) {
                let tombstone_check = Expr::BinaryOp {
                    left: Box::new(Expr::Identifier(Ident::new(TOMBSTONE_COLUMN_NAME))),
                    op: sqlparser::ast::BinaryOperator::Eq,
                    right: Box::new(Expr::Value(Value::Number("0".to_string(), false))),
                };

                let existing_selection = query.selection.take();
                let new_selection = match existing_selection {
                    Some(expr) => Expr::BinaryOp {
                        left: Box::new(expr),
                        op: sqlparser::ast::BinaryOperator::And,
                        right: Box::new(tombstone_check),
                    },
                    None => tombstone_check,
                };
                query.selection = Some(Box::new(new_selection));
            }
        }

        ControlFlow::Continue(())
    }
}
