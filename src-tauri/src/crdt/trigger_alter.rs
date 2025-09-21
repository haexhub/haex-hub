// Wir binden die Konstanten aus unserem generierten Modul ein.
// `crate` bezieht sich auf das Wurzelverzeichnis unseres Crates (src-tauri/src).
use crate::tableNames::*;

use rusqlite::{Connection, Result as RusqliteResult, Row};
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use std::panic::{self, AssertUnwindSafe};
use ts_rs::TS;

// Harte Konstanten, die nicht aus der JSON-Datei kommen, da sie Teil der internen Logik sind.
const SYNC_ACTIVE_KEY: &str = "sync_active";
const TOMBSTONE_COLUMN: &str = "haex_tombstone";
const HLC_TIMESTAMP_COLUMN: &str = "haex_hlc_timestamp";
const INSERT_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_insert";
const UPDATE_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_update";

// --- Eigener Error-Typ für klares Fehler-Handling ---
#[derive(Debug)]
pub enum CrdtSetupError {
    DatabaseError(rusqlite::Error),
    TombstoneColumnMissing {
        table_name: String,
        column_name: String,
    },
    PrimaryKeyMissing {
        table_name: String,
    },
}

impl Display for CrdtSetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CrdtSetupError::DatabaseError(e) => write!(f, "Database error: {}", e),
            CrdtSetupError::TombstoneColumnMissing {
                table_name,
                column_name,
            } => write!(
                f,
                "Table '{}' is missing the required tombstone column '{}'",
                table_name, column_name
            ),
            CrdtSetupError::PrimaryKeyMissing { table_name } => {
                write!(f, "Table '{}' has no primary key", table_name)
            }
        }
    }
}
impl Error for CrdtSetupError {}
impl From<rusqlite::Error> for CrdtSetupError {
    fn from(err: rusqlite::Error) -> Self {
        CrdtSetupError::DatabaseError(err)
    }
}

// --- Öffentliche Structs und Enums ---
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub enum TriggerSetupResult {
    Success,
    TableNotFound,
}

#[derive(Debug)]
struct ColumnInfo {
    name: String,
    is_pk: bool,
}
impl ColumnInfo {
    fn from_row(row: &Row) -> RusqliteResult<Self> {
        Ok(ColumnInfo {
            name: row.get("name")?,
            is_pk: row.get::<_, i64>("pk")? > 0,
        })
    }
}

// --- Öffentliche Funktionen für die Anwendungslogik ---

/// Erstellt die benötigten CRDT-Systemtabellen (z.B. die Config-Tabelle), falls sie nicht existieren.
/// Sollte beim Anwendungsstart einmalig aufgerufen werden.
pub fn setup_crdt_tables(conn: &mut Connection) -> RusqliteResult<()> {
    let config_sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{config_table}\" (key TEXT PRIMARY KEY, value TEXT) WITHOUT ROWID;",
        config_table = TABLE_CRDT_CONFIGS
    );
    conn.execute(&config_sql, [])?;
    Ok(())
}

/// Führt eine Aktion aus, während die Trigger temporär deaktiviert sind.
/// Stellt sicher, dass die Trigger auch bei einem Absturz (Panic) wieder aktiviert werden.
pub fn with_triggers_paused<F, R>(conn: &mut Connection, action: F) -> RusqliteResult<R>
where
    F: FnOnce(&mut Connection) -> RusqliteResult<R>,
{
    set_sync_active(conn)?;
    // `catch_unwind` fängt einen möglichen Panic in `action` ab.
    let result = panic::catch_unwind(AssertUnwindSafe(|| action(conn)));
    // Diese Aufräumaktion wird immer ausgeführt.
    clear_sync_active(conn)?;
    match result {
        Ok(res) => res,                    // Alles gut, gib das Ergebnis von `action` zurück.
        Err(e) => panic::resume_unwind(e), // Ein Panic ist aufgetreten, wir geben ihn weiter, nachdem wir aufgeräumt haben.
    }
}

/// Analysiert alle `haex_`-Tabellen in der Datenbank und erstellt die notwendigen CRDT-Trigger.
pub fn generate_haex_triggers(conn: &mut Connection) -> RusqliteResult<()> {
    println!("🔄 Setup CRDT triggers...");
    let table_names: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'haex_%' AND name NOT LIKE 'haex_crdt_%';")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<RusqliteResult<Vec<String>>>()?
    };

    for table_name in table_names {
        // Überspringe die Config-Tabelle selbst, sie braucht keine Trigger.
        if table_name == TABLE_CRDT_CONFIGS {
            continue;
        }
        println!("➡️  Processing table: {}", table_name);
        match setup_triggers_for_table(conn, &table_name) {
            Ok(TriggerSetupResult::Success) => {
                println!("   ✅ Triggers created for {}", table_name)
            }
            Ok(TriggerSetupResult::TableNotFound) => {
                println!("   ℹ️ Table {} not found, skipping.", table_name)
            }
            Err(e) => println!("   ❌ Could not set up triggers for {}: {}", table_name, e),
        }
    }
    println!("✨ Done setting up CRDT triggers.");
    Ok(())
}

// --- Private Hilfsfunktionen ---

fn set_sync_active(conn: &mut Connection) -> RusqliteResult<()> {
    let sql = format!(
        "INSERT OR REPLACE INTO \"{config_table}\" (key, value) VALUES (?, '1');",
        config_table = TABLE_CRDT_CONFIGS
    );
    conn.execute(&sql, [SYNC_ACTIVE_KEY])?;
    Ok(())
}

fn clear_sync_active(conn: &mut Connection) -> RusqliteResult<()> {
    let sql = format!(
        "DELETE FROM \"{config_table}\" WHERE key = ?;",
        config_table = TABLE_CRDT_CONFIGS
    );
    conn.execute(&sql, [SYNC_ACTIVE_KEY])?;
    Ok(())
}

fn is_safe_identifier(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn setup_triggers_for_table(
    conn: &mut Connection,
    table_name: &str,
) -> Result<TriggerSetupResult, CrdtSetupError> {
    if !is_safe_identifier(table_name) {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid table name: {}",
            table_name
        ))
        .into());
    }
    let columns = get_table_schema(conn, table_name)?;
    if columns.is_empty() {
        return Ok(TriggerSetupResult::TableNotFound);
    }
    if !columns.iter().any(|c| c.name == TOMBSTONE_COLUMN) {
        return Err(CrdtSetupError::TombstoneColumnMissing {
            table_name: table_name.to_string(),
            column_name: TOMBSTONE_COLUMN.to_string(),
        });
    }
    let pks: Vec<String> = columns
        .iter()
        .filter(|c| c.is_pk)
        .map(|c| c.name.clone())
        .collect();
    if pks.is_empty() {
        return Err(CrdtSetupError::PrimaryKeyMissing {
            table_name: table_name.to_string(),
        });
    }
    let cols_to_track: Vec<String> = columns
        .iter()
        .filter(|c| !c.is_pk && c.name != TOMBSTONE_COLUMN && c.name != HLC_TIMESTAMP_COLUMN)
        .map(|c| c.name.clone())
        .collect();

    let insert_trigger_sql = generate_insert_trigger_sql(table_name, &pks, &cols_to_track);
    let update_trigger_sql = generate_update_trigger_sql(table_name, &pks, &cols_to_track);
    let drop_insert_trigger_sql =
        drop_trigger_sql(INSERT_TRIGGER_TPL.replace("{TABLE_NAME}", table_name));
    let drop_update_trigger_sql =
        drop_trigger_sql(UPDATE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name));

    let tx = conn.transaction()?;
    tx.execute_batch(&format!(
        "{}\n{}\n{}\n{}",
        drop_insert_trigger_sql, drop_update_trigger_sql, insert_trigger_sql, update_trigger_sql
    ))?;
    tx.commit()?;

    Ok(TriggerSetupResult::Success)
}

fn get_table_schema(conn: &Connection, table_name: &str) -> RusqliteResult<Vec<ColumnInfo>> {
    let sql = format!("PRAGMA table_info(\"{}\");", table_name);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], ColumnInfo::from_row)?;
    rows.collect()
}

fn drop_trigger_sql(trigger_name: String) -> String {
    format!("DROP TRIGGER IF EXISTS \"{}\";", trigger_name)
}

fn generate_insert_trigger_sql(table_name: &str, pks: &[String], cols: &[String]) -> String {
    let pk_json_payload = pks
        .iter()
        .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
        .collect::<Vec<_>>()
        .join(", ");
    let column_inserts = cols.iter().fold(String::new(), |mut acc, col| {
        writeln!(&mut acc, "    INSERT INTO \"{log_table}\" (hlc_timestamp, op_type, table_name, row_pk, column_name, value) VALUES (NEW.\"{hlc_col}\", 'INSERT', '{table}', json_object({pk_payload}), '{column}', json_object('value', NEW.\"{column}\"));", log_table = TABLE_CRDT_LOGS, hlc_col = HLC_TIMESTAMP_COLUMN, table = table_name, pk_payload = pk_json_payload, column = col).unwrap();
        acc
    });
    let trigger_name = INSERT_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);
    format!(
        "CREATE TRIGGER IF NOT EXISTS \"{trigger_name}\"\n"
      + "        AFTER INSERT ON \"{table_name}\"\n"
      + "        WHEN (SELECT value FROM \"{config_table}\" WHERE key = '{sync_key}') IS NOT '1'\n"
      + "        FOR EACH ROW\n"
      + "        BEGIN\n"
      + "        {column_inserts}\n"
      + "        END;",
        config_table = TABLE_CRDT_CONFIGS,
        sync_key = SYNC_ACTIVE_KEY
    )
}

fn generate_update_trigger_sql(table_name: &str, pks: &[String], cols: &[String]) -> String {
    let pk_json_payload = pks
        .iter()
        .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
        .collect::<Vec<_>>()
        .join(", ");
    let column_updates = cols.iter().fold(String::new(), |mut acc, col| {
        writeln!(&mut acc, "    IF NEW.\"{column}\" IS NOT OLD.\"{column}\" THEN INSERT INTO \"{log_table}\" (hlc_timestamp, op_type, table_name, row_pk, column_name, value, old_value) VALUES (NEW.\"{hlc_col}\", 'UPDATE', '{table}', json_object({pk_payload}), '{column}', json_object('value', NEW.\"{column}\"), json_object('value', OLD.\"{column}\")); END IF;", log_table = TABLE_CRDT_LOGS, hlc_col = HLC_TIMESTAMP_COLUMN, table = table_name, pk_payload = pk_json_payload, column = col).unwrap();
        acc
    });
    let soft_delete_logic = format!(
        "    IF NEW.\"{tombstone_col}\" = 1 AND OLD.\"{tombstone_col}\" = 0 THEN INSERT INTO \"{log_table}\" (hlc_timestamp, op_type, table_name, row_pk) VALUES (NEW.\"{hlc_col}\", 'DELETE', '{table}', json_object({pk_payload})); END IF;", log_table = TABLE_CRDT_LOGS, hlc_col = HLC_TIMESTAMP_COLUMN, tombstone_col = TOMBSTONE_COLUMN, table = table_name, pk_payload = pk_json_payload);
    let trigger_name = UPDATE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);
    format!(
        "CREATE TRIGGER IF NOT EXISTS \"{trigger_name}\"\n"
      + "        AFTER UPDATE ON \"{table_name}\"\n"
      + "        WHEN (SELECT value FROM \"{config_table}\" WHERE key = '{sync_key}') IS NOT '1'\n"
      + "        FOR EACH ROW\n"
      + "        BEGIN\n"
      + "        {column_updates}\n"
      + "        {soft_delete_logic}\n"
      + "        END;",
        config_table = TABLE_CRDT_CONFIGS,
        sync_key = SYNC_ACTIVE_KEY
    )
}
