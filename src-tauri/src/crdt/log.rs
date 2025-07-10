// src/entities/crdt_log.rs
use sea_orm::entity::prelude::*;

#[sea_orm(table_name = "crdt_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub hlc_timestamp: String,
    pub op_type: String,
    pub table_name: String,
    pub row_pk: String, // Wird als JSON-String gespeichert
    #[sea_orm(nullable)]
    pub column_name: Option<String>,
    #[sea_orm(nullable)]
    pub value: Option<String>,
    #[sea_orm(nullable)]
    pub old_value: Option<String>,
}

pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
