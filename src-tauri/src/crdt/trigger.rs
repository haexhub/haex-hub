use crate::crdt::hlc;
use rusqlite::{Connection, Result, Row};
use serde::Serialize;
use std::fmt::Write;
use ts_rs::TS;

// the z_ prefix should make sure that these triggers are executed lasts
const INSERT_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_insert";
const UPDATE_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_update";

pub const LOG_TABLE_NAME: &str = "haex_crdt_logs";
pub const TOMBSTONE_COLUMN: &str = "haex_tombstone";
pub const HLC_TIMESTAMP_COLUMN: &str = "haex_hlc_timestamp";
#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(tag = "status", content = "details")]
pub enum TriggerSetupResult {
    Success,
    TableNotFound,
    TombstoneColumnMissing { column_name: String },
    PrimaryKeyMissing,
}

struct ColumnInfo {
    name: String,
    is_pk: bool,
}

impl ColumnInfo {
    fn from_row(row: &Row) -> Result<Self> {
        Ok(ColumnInfo {
            name: row.get("name")?,
            is_pk: row.get::<_, i64>("pk")? > 0,
        })
    }
}

pub struct TriggerManager;

impl TriggerManager {
    pub fn new() -> Self {
        TriggerManager {}
    }

    pub fn setup_triggers_for_table(
        &self,
        conn: &mut Connection,
        table_name: &str,
    ) -> Result<TriggerSetupResult, rusqlite::Error> {
        let columns = self.get_table_schema(conn, table_name)?;

        if columns.is_empty() {
            return Ok(TriggerSetupResult::TableNotFound);
        }

        if !columns.iter().any(|c| c.name == TOMBSTONE_COLUMN) {
            return Ok(TriggerSetupResult::TombstoneColumnMissing {
                column_name: TOMBSTONE_COLUMN.to_string(),
            });
        }

        let pks: Vec<String> = columns
            .iter()
            .filter(|c| c.is_pk)
            .map(|c| c.name.clone())
            .collect();
        if pks.is_empty() {
            return Ok(TriggerSetupResult::PrimaryKeyMissing);
        }

        let cols_to_track: Vec<String> = columns
            .iter()
            .filter(|c| !c.is_pk && c.name != TOMBSTONE_COLUMN)
            .map(|c| c.name.clone())
            .collect();

        let insert_trigger_sql = self.generate_insert_trigger_sql(table_name, &pks, &cols_to_track);
        let update_trigger_sql = self.generate_update_trigger_sql(table_name, &pks, &cols_to_track);

        let tx = conn.transaction()?;
        tx.execute_batch(&format!("{}\n{}", insert_trigger_sql, update_trigger_sql))?;
        tx.commit()?;

        Ok(TriggerSetupResult::Success)
    }

    fn get_table_schema(&self, conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let sql = format!("PRAGMA table_info('{}');", table_name);
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], ColumnInfo::from_row)?;
        rows.collect()
    }

    fn generate_insert_trigger_sql(
        &self,
        table_name: &str,
        pks: &[String],
        cols: &[String],
    ) -> String {
        let pk_json_payload = pks
            .iter()
            .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
            .collect::<Vec<_>>()
            .join(", ");

        let column_inserts = cols.iter().fold(String::new(), |mut acc, col| {
            writeln!(&mut acc, "INSERT INTO {log_table} (hlc_timestamp, op_type, table_name, row_pk, column_name, value) VALUES (NEW.\"{hlc_col}\", 'INSERT', '{table}', json_object({pk_payload}), '{column}', json_object('value', NEW.\"{column}\"));",
                log_table = LOG_TABLE_NAME,
                hlc_col = HLC_TIMESTAMP_COLUMN,
                table = table_name,
                pk_payload = pk_json_payload,
                column = col
            ).unwrap();
            acc
        });

        // Verwende die neue Konstante für den Trigger-Namen
        let trigger_name = INSERT_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);

        format!(
            "CREATE TRIGGER IF NOT EXISTS {trigger_name}
            AFTER INSERT ON \"{table_name}\"
            FOR EACH ROW
            BEGIN
                {column_inserts}
            END;"
        )
    }

    fn generate_update_trigger_sql(
        &self,
        table_name: &str,
        pks: &[String],
        cols: &[String],
    ) -> String {
        let pk_json_payload = pks
            .iter()
            .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
            .collect::<Vec<_>>()
            .join(", ");

        let column_updates = cols.iter().fold(String::new(), |mut acc, col| {
            writeln!(&mut acc, "IF NEW.\"{column}\" IS NOT OLD.\"{column}\" THEN INSERT INTO {log_table} (hlc_timestamp, op_type, table_name, row_pk, column_name, value, old_value) VALUES (NEW.\"{hlc_col}\", 'UPDATE', '{table}', json_object({pk_payload}), '{column}', json_object('value', NEW.\"{column}\"), json_object('value', OLD.\"{column}\")); END IF;",
                log_table = LOG_TABLE_NAME,
                hlc_col = HLC_TIMESTAMP_COLUMN,
                table = table_name,
                pk_payload = pk_json_payload,
                column = col).unwrap();
            acc
        });

        let soft_delete_logic = format!(
            "IF NEW.{tombstone_col} = 1 AND OLD.{tombstone_col} = 0 THEN INSERT INTO {log_table} (hlc_timestamp, op_type, table_name, row_pk) VALUES (NEW.\"{hlc_col}\", 'DELETE', '{table}', json_object({pk_payload})); END IF;",
            log_table = LOG_TABLE_NAME,
            hlc_col = HLC_TIMESTAMP_COLUMN,
            tombstone_col = TOMBSTONE_COLUMN,
            table = table_name,
            pk_payload = pk_json_payload
        );

        // Verwende die neue Konstante für den Trigger-Namen
        let trigger_name = UPDATE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);

        format!(
            "CREATE TRIGGER IF NOT EXISTS {trigger_name}
            AFTER UPDATE ON \"{table_name}\"
            FOR EACH ROW
            BEGIN
                {column_updates}
                {soft_delete_logic}
            END;"
        )
    }
}
