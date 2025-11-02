// src-tauri/src/crdt/hlc.rs

use crate::table_names::TABLE_CRDT_CONFIGS;
use rusqlite::{params, Connection, Transaction};
use serde_json::json;
use std::{
    fmt::Debug,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use thiserror::Error;
use uhlc::{HLCBuilder, Timestamp, HLC, ID};
use uuid::Uuid;

const HLC_NODE_ID_TYPE: &str = "hlc_node_id";
const HLC_TIMESTAMP_TYPE: &str = "hlc_timestamp";

#[derive(Error, Debug)]
pub enum HlcError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Failed to parse persisted HLC timestamp: {0}")]
    ParseTimestamp(String),
    #[error("Failed to parse persisted HLC state: {0}")]
    Parse(String),
    #[error("Failed to parse HLC Node ID: {0}")]
    ParseNodeId(String),
    #[error("HLC mutex was poisoned")]
    MutexPoisoned,
    #[error("Failed to create node ID: {0}")]
    CreateNodeId(#[from] uhlc::SizeError),
    #[error("No database connection available")]
    NoConnection,
    #[error("HLC service not initialized")]
    NotInitialized,
    #[error("Hex decode error: {0}")]
    HexDecode(String),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(String),
    #[error("Failed to access device store: {0}")]
    DeviceStore(String),
}

impl From<tauri_plugin_store::Error> for HlcError {
    fn from(error: tauri_plugin_store::Error) -> Self {
        HlcError::DeviceStore(error.to_string())
    }
}

/// A thread-safe, persistent HLC service.
#[derive(Clone)]
pub struct HlcService {
    hlc: Arc<Mutex<Option<HLC>>>,
}

impl HlcService {
    /// Creates a new HLC service. The HLC will be initialized on first database access.
    pub fn new() -> Self {
        HlcService {
            hlc: Arc::new(Mutex::new(None)),
        }
    }

    /// Factory-Funktion: Erstellt und initialisiert einen neuen HLC-Service aus einer bestehenden DB-Verbindung.
    /// Dies ist die bevorzugte Methode zur Instanziierung.
    pub fn try_initialize(conn: &Connection, app_handle: &AppHandle) -> Result<Self, HlcError> {
        // 1. Hole oder erstelle eine persistente Node-ID
        let node_id_str = Self::get_or_create_device_id(app_handle)?;

        // Parse den String in ein Uuid-Objekt.
        let uuid = Uuid::parse_str(&node_id_str).map_err(|e| {
            HlcError::ParseNodeId(format!(
                "Stored device ID is not a valid UUID: {node_id_str}. Error: {e}"
            ))
        })?;

        // Hol dir die rohen 16 Bytes und erstelle daraus die uhlc::ID.
        // Das `*` dereferenziert den `&[u8; 16]` zu `[u8; 16]`, was `try_from` erwartet.
        let node_id = ID::try_from(*uuid.as_bytes()).map_err(|e| {
            HlcError::ParseNodeId(format!("Invalid node ID format from device store: {e:?}"))
        })?;

        // 2. Erstelle eine HLC-Instanz mit stabiler Identität
        let hlc = HLCBuilder::new()
            .with_id(node_id)
            .with_max_delta(Duration::from_secs(1))
            .build();

        // 3. Lade und wende den letzten persistenten Zeitstempel an
        if let Some(last_timestamp) = Self::load_last_timestamp(conn)? {
            hlc.update_with_timestamp(&last_timestamp).map_err(|e| {
                HlcError::Parse(format!(
                    "Failed to update HLC with persisted timestamp: {e:?}"
                ))
            })?;
        }

        Ok(HlcService {
            hlc: Arc::new(Mutex::new(Some(hlc))),
        })
    }

    /// Holt die Geräte-ID aus dem Tauri Store oder erstellt eine neue, wenn keine existiert.
    fn get_or_create_device_id(app_handle: &AppHandle) -> Result<String, HlcError> {
        let store_path = PathBuf::from("instance.json");
        let store = app_handle
            .store(store_path)
            .map_err(|e| HlcError::DeviceStore(e.to_string()))?;

        let id_exists = match store.get("id") {
            // Fall 1: Der Schlüssel "id" existiert UND sein Wert ist ein String.
            Some(value) => {
                if let Some(s) = value.as_str() {
                    // Das ist unser Erfolgsfall. Wir haben einen &str und können
                    // eine Kopie davon zurückgeben.
                    println!("Gefundene und validierte Geräte-ID: {s}");
                    if Uuid::parse_str(s).is_ok() {
                        // Erfolgsfall: Der Wert ist ein String UND eine gültige UUID.
                        // Wir können die Funktion direkt mit dem Wert verlassen.
                        return Ok(s.to_string());
                    }
                }
                // Der Wert existiert, ist aber kein String (z.B. eine Zahl).
                // Wir behandeln das, als gäbe es keine ID.
                false
            }
            // Fall 2: Der Schlüssel "id" existiert nicht.
            None => false,
        };

        // Wenn wir hier ankommen, bedeutet das, `id_exists` ist `false`.
        // Entweder weil der Schlüssel fehlte oder weil der Wert kein String war.
        // Also erstellen wir eine neue ID.
        if !id_exists {
            let new_id = Uuid::new_v4().to_string();

            store.set("id".to_string(), json!(new_id.clone()));

            store.save()?;

            return Ok(new_id);
        }

        // Dieser Teil des Codes sollte nie erreicht werden, aber der Compiler
        // braucht einen finalen return-Wert. Wir können hier einen Fehler werfen.
        Err(HlcError::DeviceStore(
            "Unreachable code: Failed to determine device ID".to_string(),
        ))
    }

    /// Generiert einen neuen Zeitstempel und persistiert den neuen Zustand des HLC sofort.
    /// Muss innerhalb einer bestehenden Datenbanktransaktion aufgerufen werden.
    pub fn new_timestamp_and_persist<'tx>(
        &self,
        tx: &Transaction<'tx>,
    ) -> Result<Timestamp, HlcError> {
        let mut hlc_guard = self.hlc.lock().map_err(|_| HlcError::MutexPoisoned)?;
        let hlc = hlc_guard.as_mut().ok_or(HlcError::NotInitialized)?;

        let new_timestamp = hlc.new_timestamp();
        Self::persist_timestamp(tx, &new_timestamp)?;

        Ok(new_timestamp)
    }

    /// Erstellt einen neuen Zeitstempel, ohne ihn zu persistieren (z.B. für Leseoperationen).
    pub fn new_timestamp(&self) -> Result<Timestamp, HlcError> {
        let mut hlc_guard = self.hlc.lock().map_err(|_| HlcError::MutexPoisoned)?;
        let hlc = hlc_guard.as_mut().ok_or(HlcError::NotInitialized)?;

        Ok(hlc.new_timestamp())
    }

    /// Aktualisiert den HLC mit einem externen Zeitstempel (für die Synchronisation).
    pub fn update_with_timestamp(&self, timestamp: &Timestamp) -> Result<(), HlcError> {
        let mut hlc_guard = self.hlc.lock().map_err(|_| HlcError::MutexPoisoned)?;
        let hlc = hlc_guard.as_mut().ok_or(HlcError::NotInitialized)?;

        hlc.update_with_timestamp(timestamp)
            .map_err(|e| HlcError::Parse(format!("Failed to update HLC: {e:?}")))
    }

    /// Lädt den letzten persistierten Zeitstempel aus der Datenbank.
    fn load_last_timestamp(conn: &Connection) -> Result<Option<Timestamp>, HlcError> {
        let query = format!("SELECT value FROM {TABLE_CRDT_CONFIGS} WHERE key = ?1");

        match conn.query_row(&query, params![HLC_TIMESTAMP_TYPE], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(state_str) => {
                let timestamp = Timestamp::from_str(&state_str).map_err(|e| {
                    HlcError::ParseTimestamp(format!("Invalid timestamp format: {e:?}"))
                })?;
                Ok(Some(timestamp))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(HlcError::Database(e)),
        }
    }

    /// Persistiert einen Zeitstempel in der Datenbank innerhalb einer Transaktion.
    fn persist_timestamp(tx: &Transaction, timestamp: &Timestamp) -> Result<(), HlcError> {
        let timestamp_str = timestamp.to_string();
        tx.execute(
            &format!(
                "INSERT INTO {TABLE_CRDT_CONFIGS} (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value"
            ),
            params![HLC_TIMESTAMP_TYPE, timestamp_str],
        )?;
        Ok(())
    }
}

impl Default for HlcService {
    fn default() -> Self {
        Self::new()
    }
}
