use crate::crdt::trigger::{HLC_TIMESTAMP_COLUMN, TOMBSTONE_COLUMN};
use crate::database::error::DatabaseError;
use crate::table_names::{TABLE_CRDT_CONFIGS, TABLE_CRDT_LOGS};
use sqlparser::ast::{
    Assignment, AssignmentTarget, BinaryOperator, ColumnDef, DataType, Expr, Ident, Insert,
    ObjectName, ObjectNamePart, SelectItem, SetExpr, Statement, TableFactor, TableObject, Value,
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
            Statement::Query(query) => self.transform_query_recursive(query),
            // Fange alle anderen Fälle ab und gib einen Fehler zurück
            _ => Err(DatabaseError::UnsupportedStatement {
                sql: stmt.to_string(),
                reason: "This operation only accepts SELECT statements.".to_string(),
            }),
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
                        self.transform_insert(insert_stmt, hlc_timestamp)?;
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

    /// Transformiert Query-Statements (fügt Tombstone-Filter hinzu)
    fn transform_query_recursive(
        &self,
        query: &mut sqlparser::ast::Query,
    ) -> Result<(), DatabaseError> {
        self.add_tombstone_filters_recursive(&mut query.body)
    }

    /// Rekursive Behandlung aller SetExpr-Typen mit vollständiger Subquery-Unterstützung
    fn add_tombstone_filters_recursive(&self, set_expr: &mut SetExpr) -> Result<(), DatabaseError> {
        match set_expr {
            SetExpr::Select(select) => {
                self.add_tombstone_filters_to_select(select)?;

                // Transformiere auch Subqueries in Projektionen
                for projection in &mut select.projection {
                    match projection {
                        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                            self.transform_expression_subqueries(expr)?;
                        }
                        _ => {} // Wildcard projections ignorieren
                    }
                }

                // Transformiere Subqueries in WHERE
                if let Some(where_clause) = &mut select.selection {
                    self.transform_expression_subqueries(where_clause)?;
                }

                // Transformiere Subqueries in GROUP BY
                match &mut select.group_by {
                    sqlparser::ast::GroupByExpr::All(_) => {
                        // GROUP BY ALL - keine Expressions zu transformieren
                    }
                    sqlparser::ast::GroupByExpr::Expressions(exprs, _) => {
                        for group_expr in exprs {
                            self.transform_expression_subqueries(group_expr)?;
                        }
                    }
                }

                // Transformiere Subqueries in HAVING
                if let Some(having) = &mut select.having {
                    self.transform_expression_subqueries(having)?;
                }
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.add_tombstone_filters_recursive(left)?;
                self.add_tombstone_filters_recursive(right)?;
            }
            SetExpr::Query(query) => {
                self.add_tombstone_filters_recursive(&mut query.body)?;
            }
            SetExpr::Values(values) => {
                // Transformiere auch Subqueries in Values-Listen
                for row in &mut values.rows {
                    for expr in row {
                        self.transform_expression_subqueries(expr)?;
                    }
                }
            }
            _ => {} // Andere Fälle
        }
        Ok(())
    }

    /// Transformiert Subqueries innerhalb von Expressions
    fn transform_expression_subqueries(&self, expr: &mut Expr) -> Result<(), DatabaseError> {
        match expr {
            // Einfache Subqueries
            Expr::Subquery(query) => {
                self.add_tombstone_filters_recursive(&mut query.body)?;
            }
            // EXISTS Subqueries
            Expr::Exists { subquery, .. } => {
                self.add_tombstone_filters_recursive(&mut subquery.body)?;
            }
            // IN Subqueries
            Expr::InSubquery {
                expr: left_expr,
                subquery,
                ..
            } => {
                self.transform_expression_subqueries(left_expr)?;
                self.add_tombstone_filters_recursive(&mut subquery.body)?;
            }
            // ANY/ALL Subqueries
            Expr::AnyOp { left, right, .. } | Expr::AllOp { left, right, .. } => {
                self.transform_expression_subqueries(left)?;
                self.transform_expression_subqueries(right)?;
            }
            // Binäre Operationen
            Expr::BinaryOp { left, right, .. } => {
                self.transform_expression_subqueries(left)?;
                self.transform_expression_subqueries(right)?;
            }
            // Unäre Operationen
            Expr::UnaryOp {
                expr: inner_expr, ..
            } => {
                self.transform_expression_subqueries(inner_expr)?;
            }
            // Verschachtelte Ausdrücke
            Expr::Nested(nested) => {
                self.transform_expression_subqueries(nested)?;
            }
            // CASE-Ausdrücke
            Expr::Case {
                operand,
                conditions,
                else_result,
                ..
            } => {
                if let Some(op) = operand {
                    self.transform_expression_subqueries(op)?;
                }
                for case_when in conditions {
                    self.transform_expression_subqueries(&mut case_when.condition)?;
                    self.transform_expression_subqueries(&mut case_when.result)?;
                }
                if let Some(else_res) = else_result {
                    self.transform_expression_subqueries(else_res)?;
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
                            self.transform_expression_subqueries(expr)?;
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
                self.transform_expression_subqueries(main_expr)?;
                self.transform_expression_subqueries(low)?;
                self.transform_expression_subqueries(high)?;
            }
            // IN Liste
            Expr::InList {
                expr: main_expr,
                list,
                ..
            } => {
                self.transform_expression_subqueries(main_expr)?;
                for list_expr in list {
                    self.transform_expression_subqueries(list_expr)?;
                }
            }
            // IS NULL/IS NOT NULL
            Expr::IsNull(inner) | Expr::IsNotNull(inner) => {
                self.transform_expression_subqueries(inner)?;
            }
            // Andere Expression-Typen benötigen keine Transformation
            _ => {}
        }
        Ok(())
    }

    /// Fügt Tombstone-Filter zu SELECT-Statements hinzu (nur wenn nicht explizit in WHERE gesetzt)
    fn add_tombstone_filters_to_select(
        &self,
        select: &mut sqlparser::ast::Select,
    ) -> Result<(), DatabaseError> {
        // Sammle alle CRDT-Tabellen mit ihren Aliasen
        let mut crdt_tables = Vec::new();
        for twj in &select.from {
            if let TableFactor::Table { name, alias, .. } = &twj.relation {
                if self.is_crdt_sync_table(name) {
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
                tombstone_filters.push(self.columns.create_tombstone_filter(table_alias));
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
            // Einfache Spaltenreferenz: tombstone = ?
            Expr::Identifier(ident) => {
                if ident.value == self.columns.tombstone {
                    // Wenn keine Tabelle spezifiziert ist und es nur eine CRDT-Tabelle gibt
                    if crdt_tables.len() == 1 {
                        let table_name_str = crdt_tables[0].0.to_string();
                        let table_key = crdt_tables[0].1.unwrap_or(&table_name_str);
                        filtered_tables.insert(table_key.to_string());
                    }
                }
            }
            // Qualifizierte Spaltenreferenz: table.tombstone = ? oder alias.tombstone = ?
            Expr::CompoundIdentifier(idents) => {
                if idents.len() == 2 && idents[1].value == self.columns.tombstone {
                    let table_ref = &idents[0].value;

                    // Prüfe, ob es eine unserer CRDT-Tabellen ist (nach Name oder Alias)
                    for (table_name, alias) in crdt_tables {
                        let table_name_str = table_name.to_string();
                        if table_ref == &table_name_str || alias.map_or(false, |a| a == table_ref) {
                            filtered_tables.insert(table_ref.clone());
                            break;
                        }
                    }
                }
            }
            // Binäre Operationen: AND, OR, etc.
            Expr::BinaryOp { left, right, .. } => {
                self.scan_expression_for_tombstone_references(left, crdt_tables, filtered_tables);
                self.scan_expression_for_tombstone_references(right, crdt_tables, filtered_tables);
            }
            // Unäre Operationen: NOT, etc.
            Expr::UnaryOp { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            // Verschachtelte Ausdrücke
            Expr::Nested(nested) => {
                self.scan_expression_for_tombstone_references(nested, crdt_tables, filtered_tables);
            }
            // IN-Klauseln
            Expr::InList { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            // BETWEEN-Klauseln
            Expr::Between { expr, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            // IS NULL/IS NOT NULL
            Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
            }
            // Funktionsaufrufe - KORRIGIERT
            Expr::Function(func) => {
                match &func.args {
                    sqlparser::ast::FunctionArguments::List(
                        sqlparser::ast::FunctionArgumentList { args, .. },
                    ) => {
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
                    _ => {} // Andere FunctionArguments-Varianten ignorieren
                }
            }
            // CASE-Ausdrücke - KORRIGIERT
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
            // Subqueries mit vollständiger Unterstützung
            Expr::Subquery(query) => {
                self.transform_query_recursive_for_tombstone_analysis(
                    query,
                    crdt_tables,
                    filtered_tables,
                )
                .ok();
            }
            // EXISTS/NOT EXISTS Subqueries
            Expr::Exists { subquery, .. } => {
                self.transform_query_recursive_for_tombstone_analysis(
                    subquery,
                    crdt_tables,
                    filtered_tables,
                )
                .ok();
            }
            // IN/NOT IN Subqueries
            Expr::InSubquery { expr, subquery, .. } => {
                self.scan_expression_for_tombstone_references(expr, crdt_tables, filtered_tables);
                self.transform_query_recursive_for_tombstone_analysis(
                    subquery,
                    crdt_tables,
                    filtered_tables,
                )
                .ok();
            }
            // ANY/ALL Subqueries
            Expr::AnyOp { left, right, .. } | Expr::AllOp { left, right, .. } => {
                self.scan_expression_for_tombstone_references(left, crdt_tables, filtered_tables);
                self.scan_expression_for_tombstone_references(right, crdt_tables, filtered_tables);
            }
            // Andere Expression-Typen ignorieren wir für jetzt
            _ => {}
        }
    }

    /// Analysiert eine Subquery und sammelt Tombstone-Referenzen
    fn transform_query_recursive_for_tombstone_analysis(
        &self,
        query: &sqlparser::ast::Query,
        crdt_tables: &[(ObjectName, Option<&str>)],
        filtered_tables: &mut HashSet<String>,
    ) -> Result<(), DatabaseError> {
        self.analyze_set_expr_for_tombstone_references(&query.body, crdt_tables, filtered_tables)
    }

    /// Rekursiv analysiert SetExpr für Tombstone-Referenzen
    fn analyze_set_expr_for_tombstone_references(
        &self,
        set_expr: &SetExpr,
        crdt_tables: &[(ObjectName, Option<&str>)],
        filtered_tables: &mut HashSet<String>,
    ) -> Result<(), DatabaseError> {
        match set_expr {
            SetExpr::Select(select) => {
                // Analysiere WHERE-Klausel
                if let Some(where_clause) = &select.selection {
                    self.scan_expression_for_tombstone_references(
                        where_clause,
                        crdt_tables,
                        filtered_tables,
                    );
                }

                // Analysiere alle Projektionen (können auch Subqueries enthalten)
                for projection in &select.projection {
                    match projection {
                        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                            self.scan_expression_for_tombstone_references(
                                expr,
                                crdt_tables,
                                filtered_tables,
                            );
                        }
                        _ => {} // Wildcard projections ignorieren
                    }
                }

                // Analysiere GROUP BY
                match &select.group_by {
                    sqlparser::ast::GroupByExpr::All(_) => {
                        // GROUP BY ALL - keine Expressions zu analysieren
                    }
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

                // Analysiere HAVING
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
                // Analysiere Values-Listen
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
            _ => {} // Andere Varianten
        }
        Ok(())
    }

    /// Transformiert INSERT-Statements (fügt HLC-Timestamp hinzu)
    fn transform_insert(
        &self,
        insert_stmt: &mut Insert,
        timestamp: &Timestamp,
    ) -> Result<(), DatabaseError> {
        insert_stmt
            .columns
            .push(Ident::new(self.columns.hlc_timestamp));

        match insert_stmt.source.as_mut() {
            Some(query) => match &mut *query.body {
                SetExpr::Values(values) => {
                    for row in &mut values.rows {
                        row.push(Expr::Value(
                            Value::SingleQuotedString(timestamp.to_string()).into(),
                        ));
                    }
                }
                SetExpr::Select(select) => {
                    let hlc_expr =
                        Expr::Value(Value::SingleQuotedString(timestamp.to_string()).into());
                    select.projection.push(SelectItem::UnnamedExpr(hlc_expr));
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
