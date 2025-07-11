use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use uhlc::{Timestamp, HLC};
use uuid::Uuid;

const HLC_SETTING_TYPE: &str = "hlc_timestamp";

pub const GET_HLC_FUNCTION: &str = "get_hlc_timestamp";
pub const CRDT_SETTINGS_TABLE: &str = "haex_crdt_settings";
pub struct HlcService(pub Arc<Mutex<HLC>>);

pub fn setup_hlc(conn: &mut Connection) -> Result<()> {
    // 1. Lade den letzten HLC-Zustand oder erstelle einen neuen.
    let hlc = conn
        .query_row(
            "SELECT value FROM {CRDT_SETTINGS_TABLE} meta WHERE type = ?1",
            params![HLC_SETTING_TYPE],
            |row| {
                let state_str: String = row.get(0)?;
                let timestamp = Timestamp::from_str(&state_str)
                    .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?; // Konvertiere den Fehler
                Ok(HLC::new(timestamp))
            },
        )
        .unwrap_or_else(|_| HLC::default()); // Bei Fehler (z.B. nicht gefunden) -> neuen HLC erstellen.

    let hlc_arc = Arc::new(Mutex::new(hlc));

    // 2. Erstelle eine Klon für die SQL-Funktion und speichere den Zustand bei jeder neuen Timestamp-Generierung.
    let hlc_clone = hlc_arc.clone();
    let db_conn_arc = Arc::new(Mutex::new(conn.try_clone()?));

    conn.create_scalar_function(
        GET_HLC_FUNCTION,
        0,
        rusqlite::functions::FunctionFlags::SQLITE_UTF8
            | rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC,
        move |_| {
            let mut hlc = hlc_clone.lock().unwrap();
            let new_timestamp = hlc.new_timestamp();
            let timestamp_str = new_timestamp.to_string();

            // 3. Speichere den neuen Zustand sofort zurück in die DB.
            //    UPSERT-Logik: Ersetze den Wert, falls der Schlüssel existiert, sonst füge ihn ein.
            let db_conn = db_conn_arc.lock().unwrap();
            db_conn
                .execute(
                    "INSERT INTO {CRDT_SETTINGS_TABLE} (id, type, value) VALUES (?1, ?2, ?3)
                 ON CONFLICT(type) DO UPDATE SET value = excluded.value",
                    params![
                        Uuid::new_v4().to_string(), // Generiere eine neue ID für den Fall eines INSERTs
                        HLC_SETTING_TYPE,
                        &timestamp_str
                    ],
                )
                .expect("HLC state could not be persisted."); // In Prod sollte hier ein besseres Error-Handling hin.

            Ok(timestamp_str)
        },
    )?;

    // Hinweis: Den HLC-Service im Tauri-State zu managen ist nicht mehr zwingend,
    // da die SQL-Funktion nun alles Notwendige über geklonte Arcs erhält.
    // Falls du ihn dennoch für andere Commands brauchst, kannst du ihn im State speichern.

    Ok(())
}
