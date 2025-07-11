// In src-tauri/src/trigger_manager.rs -> impl<'a> TriggerManager<'a>

// In einem neuen Modul, z.B. src-tauri/src/trigger_manager.rs
use crate::crdt::proxy::ColumnInfo;
use rusqlite::{params, Connection, Result, Transaction};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

pub struct TriggerManager<'a> {
    tx: &'a Transaction<'a>,
}

impl<'a> TriggerManager<'a> {
    pub fn new(tx: &'a Transaction<'a>) -> Self {
        Self { tx }
    }

    // Die Hauptfunktion, die alles einrichtet
    pub fn setup_triggers_for_table(&self, table_name: &str) -> Result<()> {
        let columns = self.get_table_schema(table_name)?;
        let pk_cols: Vec<_> = columns
            .iter()
            .filter(|c| c.is_pk)
            .map(|c| c.name.as_str())
            .collect();
        let other_cols: Vec<_> = columns
            .iter()
            .filter(|c| !c.is_pk && c.name != "tombstone")
            .map(|c| c.name.as_str())
            .collect();

        let drop_sql = self.generate_drop_triggers_sql(table_name);
        let insert_sql = self.generate_insert_trigger_sql(table_name, &pk_cols, &other_cols);
        let update_sql = self.generate_update_trigger_sql(table_name, &pk_cols, &other_cols);

        self.tx.execute_batch(&drop_sql)?;
        self.tx.execute_batch(&insert_sql)?;
        self.tx.execute_batch(&update_sql)?;

        Ok(())
    }

    fn get_table_schema(&self, table_name: &str) -> Result<Vec<ColumnInfo>> {
        let sql = format!("PRAGMA table_info('{}')", table_name);
        let mut stmt = self.tx.prepare(&sql)?;
        let rows = stmt.query_map(|row| {
            let pk_val: i64 = row.get(5)?;
            Ok(ColumnInfo {
                name: row.get(1)?,
                is_pk: pk_val > 0,
            })
        })?;

        let mut columns = Vec::new();
        for row_result in rows {
            columns.push(row_result?);
        }
        Ok(columns)
    }

    //... Implementierung der SQL-Generierungsfunktionen...

    fn generate_update_trigger_sql(&self, table_name: &str, pks: &[&str], cols: &[&str]) -> String {
        // Erstellt dynamisch die Key-Value-Paare für das JSON-Objekt des Primärschlüssels.
        let pk_json_payload_new = pks
            .iter()
            .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
            .collect::<Vec<_>>()
            .join(", ");

        let pk_json_payload_old = pks
            .iter()
            .map(|pk| format!("'{}', OLD.\"{}\"", pk, pk))
            .collect::<Vec<_>>()
            .join(", ");

        // Erstellt die einzelnen INSERT-Anweisungen für jede Spalte
        let column_updates = cols.iter().map(|col| format!(
            r#"
            -- Protokolliere die Spaltenänderung, wenn sie stattgefunden hat und es kein Soft-Delete ist
            INSERT INTO crdt_log (hlc_timestamp, op_type, table_name, row_pk, column_name, value, old_value)
            SELECT
                'placeholder_hlc', -- TODO: HLC-Funktion hier aufrufen
                'UPDATE',
                '{table}',
                json_object({pk_payload_new}),
                '{column}',
                json_object('value', NEW."{column}"),
                json_object('value', OLD."{column}")
            WHERE
                NEW."{column}" IS NOT OLD."{column}" 
            "#,
            table = table_name,
            pk_payload_new = pk_json_payload_new,
            column = col
        )).collect::<Vec<_>>().join("\n");

        // Erstellt die Logik für den Soft-Delete
        let soft_delete_logic = format!(
            r#"
            -- Protokolliere den Soft-Delete
            INSERT INTO crdt_log (hlc_timestamp, op_type, table_name, row_pk)
            SELECT
                'placeholder_hlc', -- TODO: HLC-Funktion hier aufrufen
                'DELETE',
                '{table}',
                json_object({pk_payload_old})
            WHERE
                OLD.{tombstone_col} = 0
            "#,
            table = table_name,
            pk_payload_old = pk_json_payload_old
        );

        // Kombiniert alles zu einem einzigen Trigger
        format!(
            "CREATE TRIGGER IF NOT EXISTS {table_name}_crdt_update
            AFTER UPDATE ON {table_name}
            FOR EACH ROW
            BEGIN
                {column_updates}
                {soft_delete_logic}
            END;"
        )
    }
}
