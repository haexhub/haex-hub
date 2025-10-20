// src-tauri/src/crdt/query_transformer.rs
// SELECT-spezifische CRDT-Transformationen (Tombstone-Filterung)

use crate::crdt::trigger::{TOMBSTONE_COLUMN};
use crate::database::error::DatabaseError;
use sqlparser::ast::{
    BinaryOperator, Expr, Ident, ObjectName, SelectItem, SetExpr, TableFactor, Value,
};
use std::collections::HashSet;

/// Helper-Struct für SELECT-Transformationen
pub struct QueryTransformer {
    tombstone_column: &'static str,
}

impl QueryTransformer {
    pub fn new() -> Self {
        Self {
            tombstone_column: TOMBSTONE_COLUMN,
        }
    }

    /// Transformiert Query-Statements (fügt Tombstone-Filter hinzu)
    pub fn transform_query_recursive(
        &self,
        query: &mut sqlparser::ast::Query,
        excluded_tables: &std::collections::HashSet<&str>,
    ) -> Result<(), DatabaseError> {
        self.add_tombstone_filters_recursive(&mut query.body, excluded_tables)
    }

    /// Rekursive Behandlung aller SetExpr-Typen mit vollständiger Subquery-Unterstützung
    fn add_tombstone_filters_recursive(
        &self,
        set_expr: &mut SetExpr,
        excluded_tables: &std::collections::HashSet<&str>,
    ) -> Result<(), DatabaseError> {
        match set_expr {
            SetExpr::Select(select) => {
                self.add_tombstone_filters_to_select(select, excluded_tables)?;

                // Transformiere auch Subqueries in Projektionen
                for projection in &mut select.projection {
                    match projection {
                        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                            self.transform_expression_subqueries(expr, excluded_tables)?;
                        }
                        _ => {} // Wildcard projections ignorieren
                    }
                }

                // Transformiere Subqueries in WHERE
                if let Some(where_clause) = &mut select.selection {
                    self.transform_expression_subqueries(where_clause, excluded_tables)?;
                }

                // Transformiere Subqueries in GROUP BY
                match &mut select.group_by {
                    sqlparser::ast::GroupByExpr::All(_) => {
                        // GROUP BY ALL - keine Expressions zu transformieren
                    }
                    sqlparser::ast::GroupByExpr::Expressions(exprs, _) => {
                        for group_expr in exprs {
                            self.transform_expression_subqueries(group_expr, excluded_tables)?;
                        }
                    }
                }

                // Transformiere Subqueries in HAVING
                if let Some(having) = &mut select.having {
                    self.transform_expression_subqueries(having, excluded_tables)?;
                }
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.add_tombstone_filters_recursive(left, excluded_tables)?;
                self.add_tombstone_filters_recursive(right, excluded_tables)?;
            }
            SetExpr::Query(query) => {
                self.add_tombstone_filters_recursive(&mut query.body, excluded_tables)?;
            }
            SetExpr::Values(values) => {
                // Transformiere auch Subqueries in Values-Listen
                for row in &mut values.rows {
                    for expr in row {
                        self.transform_expression_subqueries(expr, excluded_tables)?;
                    }
                }
            }
            _ => {} // Andere Fälle
        }
        Ok(())
    }

    /// Transformiert Subqueries innerhalb von Expressions
    fn transform_expression_subqueries(
        &self,
        expr: &mut Expr,
        excluded_tables: &std::collections::HashSet<&str>,
    ) -> Result<(), DatabaseError> {
        match expr {
            // Einfache Subqueries
            Expr::Subquery(query) => {
                self.add_tombstone_filters_recursive(&mut query.body, excluded_tables)?;
            }
            // EXISTS Subqueries
            Expr::Exists { subquery, .. } => {
                self.add_tombstone_filters_recursive(&mut subquery.body, excluded_tables)?;
            }
            // IN Subqueries
            Expr::InSubquery {
                expr: left_expr,
                subquery,
                ..
            } => {
                self.transform_expression_subqueries(left_expr, excluded_tables)?;
                self.add_tombstone_filters_recursive(&mut subquery.body, excluded_tables)?;
            }
            // ANY/ALL Subqueries
            Expr::AnyOp { left, right, .. } | Expr::AllOp { left, right, .. } => {
                self.transform_expression_subqueries(left, excluded_tables)?;
                self.transform_expression_subqueries(right, excluded_tables)?;
            }
            // Binäre Operationen
            Expr::BinaryOp { left, right, .. } => {
                self.transform_expression_subqueries(left, excluded_tables)?;
                self.transform_expression_subqueries(right, excluded_tables)?;
            }
            // Unäre Operationen
            Expr::UnaryOp {
                expr: inner_expr, ..
            } => {
                self.transform_expression_subqueries(inner_expr, excluded_tables)?;
            }
            // Verschachtelte Ausdrücke
            Expr::Nested(nested) => {
                self.transform_expression_subqueries(nested, excluded_tables)?;
            }
            // CASE-Ausdrücke
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                if let Some(op) = operand {
                    self.transform_expression_subqueries(op, excluded_tables)?;
                }
                for case_when in conditions {
                    self.transform_expression_subqueries(&mut case_when.condition, excluded_tables)?;
                    self.transform_expression_subqueries(&mut case_when.result, excluded_tables)?;
                }
                if let Some(else_res) = else_result {
                    self.transform_expression_subqueries(else_res, excluded_tables)?;
                }
            }
            // Funktionsaufrufe
            Expr::Function(func) => match &mut func.args {
                sqlparser::ast::FunctionArguments::List(sqlparser::ast::FunctionArgumentList {
                    args,
                    ..
                }) => {
                    for arg in args {
                        if let sqlparser::ast::FunctionArg::Unnamed(
                            sqlparser::ast::FunctionArgExpr::Expr(expr),
                        ) = arg
                        {
                            self.transform_expression_subqueries(expr, excluded_tables)?;
                        }
                    }
                }
                _ => {}
            },
            // BETWEEN
            Expr::Between {
                expr: main_expr,
                low,
                high,
                ..
            } => {
                self.transform_expression_subqueries(main_expr, excluded_tables)?;
                self.transform_expression_subqueries(low, excluded_tables)?;
                self.transform_expression_subqueries(high, excluded_tables)?;
            }
            // IN Liste
            Expr::InList {
                expr: main_expr,
                list,
                ..
            } => {
                self.transform_expression_subqueries(main_expr, excluded_tables)?;
                for list_expr in list {
                    self.transform_expression_subqueries(list_expr, excluded_tables)?;
                }
            }
            // IS NULL/IS NOT NULL
            Expr::IsNull(inner) | Expr::IsNotNull(inner) => {
                self.transform_expression_subqueries(inner, excluded_tables)?;
            }
            // Andere Expression-Typen benötigen keine Transformation
            _ => {}
        }
        Ok(())
    }

    /// Erstellt einen Tombstone-Filter für eine Tabelle
    pub fn create_tombstone_filter(&self, table_alias: Option<&str>) -> Expr {
        let column_expr = match table_alias {
            Some(alias) => {
                Expr::CompoundIdentifier(vec![Ident::new(alias), Ident::new(self.tombstone_column)])
            }
            None => {
                Expr::Identifier(Ident::new(self.tombstone_column))
            }
        };

        Expr::BinaryOp {
            left: Box::new(column_expr),
            op: BinaryOperator::NotEq,
            right: Box::new(Expr::Value(Value::Number("1".to_string(), false).into())),
        }
    }

    /// Normalisiert Tabellennamen (entfernt Anführungszeichen)
    pub fn normalize_table_name(&self, name: &ObjectName) -> String {
        let name_str = name.to_string().to_lowercase();
        name_str.trim_matches('`').trim_matches('"').to_string()
    }

    /// Fügt Tombstone-Filter zu SELECT-Statements hinzu
    pub fn add_tombstone_filters_to_select(
        &self,
        select: &mut sqlparser::ast::Select,
        excluded_tables: &HashSet<&str>,
    ) -> Result<(), DatabaseError> {
        // Sammle alle CRDT-Tabellen mit ihren Aliasen
        let mut crdt_tables = Vec::new();
        for twj in &select.from {
            if let TableFactor::Table { name, alias, .. } = &twj.relation {
                let table_name_str = self.normalize_table_name(name);
                if !excluded_tables.contains(table_name_str.as_str()) {
                    let table_alias = alias.as_ref().map(|a| a.name.value.as_str());
                    crdt_tables.push((name.clone(), table_alias));
                }
            }
        }

        if crdt_tables.is_empty() {
            return Ok(());
        }

        // Prüfe, welche Tombstone-Spalten bereits in der WHERE-Klausel referenziert werden
        let explicitly_filtered_tables = if let Some(where_clause) = &select.selection {
            self.find_explicitly_filtered_tombstone_tables(where_clause, &crdt_tables)
        } else {
            HashSet::new()
        };

        // Erstelle Filter nur für Tabellen, die noch nicht explizit gefiltert werden
        let mut tombstone_filters = Vec::new();
        for (table_name, table_alias) in crdt_tables {
            let table_name_string = table_name.to_string();
            let table_key = table_alias.unwrap_or(&table_name_string);
            if !explicitly_filtered_tables.contains(table_key) {
                tombstone_filters.push(self.create_tombstone_filter(table_alias));
            }
        }

        // Füge die automatischen Filter hinzu
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

        Ok(())
    }

    /// Findet alle Tabellen, die bereits explizit Tombstone-Filter in der WHERE-Klausel haben
    fn find_explicitly_filtered_tombstone_tables(
        &self,
        where_expr: &Expr,
        crdt_tables: &[(ObjectName, Option<&str>)],
    ) -> HashSet<String> {
        let mut filtered_tables = HashSet::new();
        self.scan_expression_for_tombstone_references(
            where_expr,
            crdt_tables,
            &mut filtered_tables,
        );
        filtered_tables
    }

    /// Rekursiv durchsucht einen Expression-Baum nach Tombstone-Spalten-Referenzen
    fn scan_expression_for_tombstone_references(
        &self,
        expr: &Expr,
        crdt_tables: &[(ObjectName, Option<&str>)],
        filtered_tables: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Identifier(ident) => {
                if ident.value == self.tombstone_column && crdt_tables.len() == 1 {
                    let table_name_str = crdt_tables[0].0.to_string();
                    let table_key = crdt_tables[0].1.unwrap_or(&table_name_str);
                    filtered_tables.insert(table_key.to_string());
                }
            }
            Expr::CompoundIdentifier(idents) => {
                if idents.len() == 2 && idents[1].value == self.tombstone_column {
                    let table_ref = &idents[0].value;
                    for (table_name, alias) in crdt_tables {
                        let table_name_str = table_name.to_string();
                        if table_ref == &table_name_str || alias.map_or(false, |a| a == table_ref) {
                            filtered_tables.insert(table_ref.clone());
                            break;
                        }
                    }
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.scan_expression_for_tombstone_references(left, crdt_tables, filtered_tables);
                self.scan_expression_for_tombstone_references(right, crdt_tables, filtered_tables);
            }
            Expr::UnaryOp { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            Expr::Nested(nested) => {
                self.scan_expression_for_tombstone_references(nested, crdt_tables, filtered_tables);
            }
            Expr::InList { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            Expr::Between { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            Expr::Function(func) => {
                if let sqlparser::ast::FunctionArguments::List(
                    sqlparser::ast::FunctionArgumentList { args, .. },
                ) = &func.args
                {
                    for arg in args {
                        if let sqlparser::ast::FunctionArg::Unnamed(
                            sqlparser::ast::FunctionArgExpr::Expr(expr),
                        ) = arg
                        {
                            self.scan_expression_for_tombstone_references(
                                expr,
                                crdt_tables,
                                filtered_tables,
                            );
                        }
                    }
                }
            }
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                if let Some(op) = operand {
                    self.scan_expression_for_tombstone_references(op, crdt_tables, filtered_tables);
                }
                for case_when in conditions {
                    self.scan_expression_for_tombstone_references(
                        &case_when.condition,
                        crdt_tables,
                        filtered_tables,
                    );
                    self.scan_expression_for_tombstone_references(
                        &case_when.result,
                        crdt_tables,
                        filtered_tables,
                    );
                }
                if let Some(else_res) = else_result {
                    self.scan_expression_for_tombstone_references(
                        else_res,
                        crdt_tables,
                        filtered_tables,
                    );
                }
            }
            Expr::Subquery(query) => {
                self.analyze_query_for_tombstone_references(query, crdt_tables, filtered_tables)
                    .ok();
            }
            Expr::Exists { subquery, .. } => {
                self.analyze_query_for_tombstone_references(subquery, crdt_tables, filtered_tables)
                    .ok();
            }
            Expr::InSubquery { expr, subquery, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
                self.analyze_query_for_tombstone_references(subquery, crdt_tables, filtered_tables)
                    .ok();
            }
            Expr::AnyOp { left, right, .. } | Expr::AllOp { left, right, .. } => {
                self.scan_expression_for_tombstone_references(left, crdt_tables, filtered_tables);
                self.scan_expression_for_tombstone_references(right, crdt_tables, filtered_tables);
            }
            _ => {}
        }
    }

    fn analyze_query_for_tombstone_references(
        &self,
        query: &sqlparser::ast::Query,
        crdt_tables: &[(ObjectName, Option<&str>)],
        filtered_tables: &mut HashSet<String>,
    ) -> Result<(), DatabaseError> {
        self.analyze_set_expr_for_tombstone_references(&query.body, crdt_tables, filtered_tables)
    }

    fn analyze_set_expr_for_tombstone_references(
        &self,
        set_expr: &SetExpr,
        crdt_tables: &[(ObjectName, Option<&str>)],
        filtered_tables: &mut HashSet<String>,
    ) -> Result<(), DatabaseError> {
        match set_expr {
            SetExpr::Select(select) => {
                if let Some(where_clause) = &select.selection {
                    self.scan_expression_for_tombstone_references(
                        where_clause,
                        crdt_tables,
                        filtered_tables,
                    );
                }

                for projection in &select.projection {
                    match projection {
                        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                            self.scan_expression_for_tombstone_references(
                                expr,
                                crdt_tables,
                                filtered_tables,
                            );
                        }
                        _ => {}
                    }
                }

                match &select.group_by {
                    sqlparser::ast::GroupByExpr::All(_) => {}
                    sqlparser::ast::GroupByExpr::Expressions(exprs, _) => {
                        for group_expr in exprs {
                            self.scan_expression_for_tombstone_references(
                                group_expr,
                                crdt_tables,
                                filtered_tables,
                            );
                        }
                    }
                }

                if let Some(having) = &select.having {
                    self.scan_expression_for_tombstone_references(
                        having,
                        crdt_tables,
                        filtered_tables,
                    );
                }
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.analyze_set_expr_for_tombstone_references(left, crdt_tables, filtered_tables)?;
                self.analyze_set_expr_for_tombstone_references(
                    right,
                    crdt_tables,
                    filtered_tables,
                )?;
            }
            SetExpr::Query(query) => {
                self.analyze_set_expr_for_tombstone_references(
                    &query.body,
                    crdt_tables,
                    filtered_tables,
                )?;
            }
            SetExpr::Values(values) => {
                for row in &values.rows {
                    for expr in row {
                        self.scan_expression_for_tombstone_references(
                            expr,
                            crdt_tables,
                            filtered_tables,
                        );
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
