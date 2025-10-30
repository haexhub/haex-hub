// src-tauri/src/crdt/trigger.rs
use crate::table_names::TABLE_CRDT_LOGS;
use rusqlite::{Connection, Result as RusqliteResult, Row, Transaction};
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write};
use ts_rs::TS;

// Der "z_"-Präfix soll sicherstellen, dass diese Trigger als Letzte ausgeführt werden
const INSERT_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_insert";
const UPDATE_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_update";
const DELETE_TRIGGER_TPL: &str = "z_crdt_{TABLE_NAME}_delete";

//const SYNC_ACTIVE_KEY: &str = "sync_active";

pub const HLC_TIMESTAMP_COLUMN: &str = "haex_timestamp";

/// Name der custom UUID-Generierungs-Funktion (registriert in database::core::open_and_init_db)
pub const UUID_FUNCTION_NAME: &str = "gen_uuid";

#[derive(Debug)]
pub enum CrdtSetupError {
    /// Kapselt einen Fehler, der von der rusqlite-Bibliothek kommt.
    DatabaseError(rusqlite::Error),
    HlcColumnMissing {
        table_name: String,
        column_name: String,
    },
    /// Die Tabelle hat keinen Primärschlüssel, was eine CRDT-Voraussetzung ist.
    PrimaryKeyMissing { table_name: String },
}

// Implementierung, damit unser Error-Typ schön formatiert werden kann.
impl Display for CrdtSetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CrdtSetupError::DatabaseError(e) => write!(f, "Database error: {}", e),
            CrdtSetupError::HlcColumnMissing {
                table_name,
                column_name,
            } => write!(
                f,
                "Table '{}' is missing the required hlc column '{}'",
                table_name, column_name
            ),
            CrdtSetupError::PrimaryKeyMissing { table_name } => {
                write!(f, "Table '{}' has no primary key", table_name)
            }
        }
    }
}

// Implementierung, damit unser Typ als "echter" Error erkannt wird.
impl Error for CrdtSetupError {}

// Wichtige Konvertierung: Erlaubt uns, den `?`-Operator auf Funktionen zu verwenden,
// die `rusqlite::Error` zurückgeben. Der Fehler wird automatisch in unseren
// `CrdtSetupError::DatabaseError` verpackt.
impl From<rusqlite::Error> for CrdtSetupError {
    fn from(err: rusqlite::Error) -> Self {
        CrdtSetupError::DatabaseError(err)
    }
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub enum TriggerSetupResult {
    Success,
    TableNotFound,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub is_pk: bool,
}

impl ColumnInfo {
    pub fn from_row(row: &Row) -> RusqliteResult<Self> {
        Ok(ColumnInfo {
            name: row.get("name")?,
            is_pk: row.get::<_, i64>("pk")? > 0,
        })
    }
}

fn is_safe_identifier(name: &str) -> bool {
    // Allow alphanumeric characters, underscores, and hyphens (for extension names like "nuxt-app")
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Richtet CRDT-Trigger für eine einzelne Tabelle ein.
pub fn setup_triggers_for_table(
    tx: &Transaction,
    table_name: &str,
    recreate: bool,
) -> Result<TriggerSetupResult, CrdtSetupError> {
    let columns = get_table_schema(tx, table_name)?;

    if columns.is_empty() {
        return Ok(TriggerSetupResult::TableNotFound);
    }

    if !columns.iter().any(|c| c.name == HLC_TIMESTAMP_COLUMN) {
        return Err(CrdtSetupError::HlcColumnMissing {
            table_name: table_name.to_string(),
            column_name: HLC_TIMESTAMP_COLUMN.to_string(),
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
        .filter(|c| !c.is_pk)
        .map(|c| c.name.clone())
        .collect();

    let insert_trigger_sql = generate_insert_trigger_sql(table_name, &pks, &cols_to_track);
    let update_trigger_sql = generate_update_trigger_sql(table_name, &pks, &cols_to_track);
    let delete_trigger_sql = generate_delete_trigger_sql(table_name, &pks, &cols_to_track);

    if recreate {
        drop_triggers_for_table(&tx, table_name)?;
    }

    tx.execute_batch(&insert_trigger_sql)?;
    tx.execute_batch(&update_trigger_sql)?;
    tx.execute_batch(&delete_trigger_sql)?;

    Ok(TriggerSetupResult::Success)
}

/// Holt das Schema für eine gegebene Tabelle.
pub fn get_table_schema(conn: &Connection, table_name: &str) -> RusqliteResult<Vec<ColumnInfo>> {
    if !is_safe_identifier(table_name) {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid or unsafe table name provided: {}",
            table_name
        ))
        .into());
    }

    let sql = format!("PRAGMA table_info(\"{}\");", table_name);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], ColumnInfo::from_row)?;
    rows.collect()
}

// get_foreign_key_columns() removed - not needed with hard deletes (no ON CONFLICT logic)

pub fn drop_triggers_for_table(
    tx: &Transaction, // Arbeitet direkt auf einer Transaktion
    table_name: &str,
) -> Result<(), CrdtSetupError> {
    if !is_safe_identifier(table_name) {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid or unsafe table name provided: {}",
            table_name
        ))
        .into());
    }

    let drop_insert_trigger_sql =
        drop_trigger_sql(INSERT_TRIGGER_TPL.replace("{TABLE_NAME}", table_name));
    let drop_update_trigger_sql =
        drop_trigger_sql(UPDATE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name));
    let drop_delete_trigger_sql =
        drop_trigger_sql(DELETE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name));

    let sql_batch = format!(
        "{}\n{}\n{}",
        drop_insert_trigger_sql, drop_update_trigger_sql, drop_delete_trigger_sql
    );

    tx.execute_batch(&sql_batch)?;
    Ok(())
}

/* pub fn recreate_triggers_for_table(
    conn: &mut Connection,
    table_name: &str,
) -> Result<TriggerSetupResult, CrdtSetupError> {
    // Starte eine einzige Transaktion für beide Operationen
    let tx = conn.transaction()?;

    // 1. Rufe die Drop-Funktion auf
    drop_triggers_for_table(&tx, table_name)?;

    // 2. Erstelle die Trigger neu (vereinfachte Logik ohne Drop)
    // Wir rufen die `setup_triggers_for_table` Logik hier manuell nach,
    // um die Transaktion weiterzuverwenden.
    let columns = get_table_schema(&tx, table_name)?;

    if columns.is_empty() {
        tx.commit()?; // Wichtig: Transaktion beenden
        return Ok(TriggerSetupResult::TableNotFound);
    }
    // ... (Validierungslogik wiederholen) ...
    if !columns.iter().any(|c| c.name == TOMBSTONE_COLUMN) {
        /* ... */
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
        /* ... */
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
    let sql_batch = format!("{}\n{}", insert_trigger_sql, update_trigger_sql);
    tx.execute_batch(&sql_batch)?;

    // Beende die Transaktion
    tx.commit()?;

    Ok(TriggerSetupResult::Success)
}
 */
/// Generiert das SQL für den INSERT-Trigger.
fn generate_insert_trigger_sql(table_name: &str, pks: &[String], cols: &[String]) -> String {
    let pk_json_payload = pks
        .iter()
        .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
        .collect::<Vec<_>>()
        .join(", ");

    let column_inserts = if cols.is_empty() {
        // Nur PKs -> einfacher Insert ins Log
        format!(
            "INSERT INTO {log_table} (id, haex_timestamp, op_type, table_name, row_pks)
            VALUES ({uuid_fn}(), NEW.\"{hlc_col}\", 'INSERT', '{table}', json_object({pk_payload}));",
            log_table = TABLE_CRDT_LOGS,
            uuid_fn = UUID_FUNCTION_NAME,
            hlc_col = HLC_TIMESTAMP_COLUMN,
            table = table_name,
            pk_payload = pk_json_payload
        )
    } else {
        cols.iter().fold(String::new(), |mut acc, col| {
            writeln!(
                &mut acc,
                "INSERT INTO {log_table} (id, haex_timestamp, op_type, table_name, row_pks, column_name, new_value)
                VALUES ({uuid_fn}(), NEW.\"{hlc_col}\", 'INSERT', '{table}', json_object({pk_payload}), '{column}', json_object('value', NEW.\"{column}\"));",
                log_table = TABLE_CRDT_LOGS,
                uuid_fn = UUID_FUNCTION_NAME,
                hlc_col = HLC_TIMESTAMP_COLUMN,
                table = table_name,
                pk_payload = pk_json_payload,
                column = col
            ).unwrap();
            acc
        })
    };

    let trigger_name = INSERT_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);

    format!(
        "CREATE TRIGGER IF NOT EXISTS \"{trigger_name}\"
            AFTER INSERT ON \"{table_name}\"
            FOR EACH ROW
            BEGIN
            {column_inserts}
            END;"
    )
}

/// Generiert das SQL zum Löschen eines Triggers.
fn drop_trigger_sql(trigger_name: String) -> String {
    format!("DROP TRIGGER IF EXISTS \"{}\";", trigger_name)
}

/// Generiert das SQL für den UPDATE-Trigger.
fn generate_update_trigger_sql(table_name: &str, pks: &[String], cols: &[String]) -> String {
    let pk_json_payload = pks
        .iter()
        .map(|pk| format!("'{}', NEW.\"{}\"", pk, pk))
        .collect::<Vec<_>>()
        .join(", ");

    let mut body = String::new();

    // Spaltenänderungen loggen
    if !cols.is_empty() {
        for col in cols {
            writeln!(
                &mut body,
                "INSERT INTO {log_table} (id, haex_timestamp, op_type, table_name, row_pks, column_name, new_value, old_value)
                    SELECT {uuid_fn}(), NEW.\"{hlc_col}\", 'UPDATE', '{table}', json_object({pk_payload}), '{column}',
                    json_object('value', NEW.\"{column}\"), json_object('value', OLD.\"{column}\")
                    WHERE NEW.\"{column}\" IS NOT OLD.\"{column}\";",
                log_table = TABLE_CRDT_LOGS,
                uuid_fn = UUID_FUNCTION_NAME,
                hlc_col = HLC_TIMESTAMP_COLUMN,
                table = table_name,
                pk_payload = pk_json_payload,
                column = col
            ).unwrap();
        }
    }

    // Soft-delete Logging entfernt - wir nutzen jetzt Hard Deletes mit eigenem BEFORE DELETE Trigger

    let trigger_name = UPDATE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);

    format!(
        "CREATE TRIGGER IF NOT EXISTS \"{trigger_name}\"
            AFTER UPDATE ON \"{table_name}\"
            FOR EACH ROW
            BEGIN
            {body}
            END;"
    )
}

/// Generiert das SQL für den BEFORE DELETE-Trigger.
/// WICHTIG: BEFORE DELETE damit die Daten noch verfügbar sind!
fn generate_delete_trigger_sql(table_name: &str, pks: &[String], cols: &[String]) -> String {
    let pk_json_payload = pks
        .iter()
        .map(|pk| format!("'{}', OLD.\"{}\"", pk, pk))
        .collect::<Vec<_>>()
        .join(", ");

    let mut body = String::new();

    // Alle Spaltenwerte speichern für mögliche Wiederherstellung
    if !cols.is_empty() {
        for col in cols {
            writeln!(
                &mut body,
                "INSERT INTO {log_table} (id, haex_timestamp, op_type, table_name, row_pks, column_name, old_value)
                    VALUES ({uuid_fn}(), OLD.\"{hlc_col}\", 'DELETE', '{table}', json_object({pk_payload}), '{column}',
                    json_object('value', OLD.\"{column}\"));",
                log_table = TABLE_CRDT_LOGS,
                uuid_fn = UUID_FUNCTION_NAME,
                hlc_col = HLC_TIMESTAMP_COLUMN,
                table = table_name,
                pk_payload = pk_json_payload,
                column = col
            ).unwrap();
        }
    } else {
        // Nur PKs -> minimales Delete Log
        writeln!(
            &mut body,
            "INSERT INTO {log_table} (id, haex_timestamp, op_type, table_name, row_pks)
                VALUES ({uuid_fn}(), OLD.\"{hlc_col}\", 'DELETE', '{table}', json_object({pk_payload}));",
            log_table = TABLE_CRDT_LOGS,
            uuid_fn = UUID_FUNCTION_NAME,
            hlc_col = HLC_TIMESTAMP_COLUMN,
            table = table_name,
            pk_payload = pk_json_payload
        )
        .unwrap();
    }

    let trigger_name = DELETE_TRIGGER_TPL.replace("{TABLE_NAME}", table_name);

    format!(
        "CREATE TRIGGER IF NOT EXISTS \"{trigger_name}\"
            BEFORE DELETE ON \"{table_name}\"
            FOR EACH ROW
            BEGIN
            {body}
            END;"
    )
}
