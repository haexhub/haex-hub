// src-tauri/src/crdt/hlc.rs

use crate::table_names::TABLE_CRDT_CONFIGS;
use rusqlite::{params, Connection, Transaction};
use std::{
    fmt::Debug,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
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
    pub fn new_from_connection(conn: &mut Connection) -> Result<Self, HlcError> {
        // 1. Hole oder erstelle eine persistente Node-ID
        let node_id = Self::get_or_create_node_id(conn)?;

        // 2. Erstelle eine HLC-Instanz mit stabiler Identität
        let hlc = HLCBuilder::new()
            .with_id(node_id)
            .with_max_delta(Duration::from_secs(1))
            .build();

        // 3. Lade und wende den letzten persistenten Zeitstempel an
        if let Some(last_timestamp) = Self::load_last_timestamp(conn)? {
            hlc.update_with_timestamp(&last_timestamp).map_err(|e| {
                HlcError::Parse(format!(
                    "Failed to update HLC with persisted timestamp: {:?}",
                    e
                ))
            })?;
        }

        Ok(HlcService {
            hlc: Arc::new(Mutex::new(Some(hlc))),
        })
    }

    /* /// Initializes the HLC service with data from the database.
    /// This should be called once after the database connection is available.
    pub fn initialize(&self, conn: &mut Connection) -> Result<(), HlcError> {
        let mut initialized = self
            .initialized
            .lock()
            .map_err(|_| HlcError::MutexPoisoned)?;

        if *initialized {
            return Ok(()); // Already initialized
        }

        let mut hlc_guard = self.hlc.lock().map_err(|_| HlcError::MutexPoisoned)?;

        // 1. Get or create persistent node ID
        let node_id = Self::get_or_create_node_id(conn)?;

        // 2. Create HLC instance with stable identity
        let hlc = HLCBuilder::new()
            .with_id(node_id)
            .with_max_delta(Duration::from_secs(1))
            .build();

        // 3. Load and apply last persisted timestamp
        if let Some(last_timestamp) = Self::load_last_timestamp(conn)? {
            hlc.update_with_timestamp(&last_timestamp).map_err(|e| {
                HlcError::Parse(format!(
                    "Failed to update HLC with persisted timestamp: {:?}",
                    e
                ))
            })?;
        }

        *hlc_guard = Some(hlc);
        *initialized = true;

        Ok(())
    } */

    /* /// Ensures the HLC service is initialized, calling initialize if needed.
    pub fn ensure_initialized(&self, conn: &mut Connection) -> Result<(), HlcError> {
        let initialized = self
            .initialized
            .lock()
            .map_err(|_| HlcError::MutexPoisoned)?;
        if !*initialized {
            drop(initialized); // Release lock before calling initialize
            self.initialize(conn)?;
        }
        Ok(())
    } */

    /* /// Checks if the service is initialized without requiring a database connection.
    pub fn is_initialized(&self) -> Result<bool, HlcError> {
        let initialized = self
            .initialized
            .lock()
            .map_err(|_| HlcError::MutexPoisoned)?;
        Ok(*initialized)
    } */

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
            .map_err(|e| HlcError::Parse(format!("Failed to update HLC: {:?}", e)))
    }

    /// Lädt den letzten persistierten Zeitstempel aus der Datenbank.
    fn load_last_timestamp(conn: &Connection) -> Result<Option<Timestamp>, HlcError> {
        let query = format!("SELECT value FROM {} WHERE key = ?1", TABLE_CRDT_CONFIGS);

        match conn.query_row(&query, params![HLC_TIMESTAMP_TYPE], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(state_str) => {
                let timestamp = Timestamp::from_str(&state_str).map_err(|e| {
                    HlcError::ParseTimestamp(format!("Invalid timestamp format: {:?}", e))
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
                "INSERT INTO {} (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                TABLE_CRDT_CONFIGS
            ),
            params![HLC_TIMESTAMP_TYPE, timestamp_str],
        )?;
        Ok(())
    }

    /// Holt oder erstellt und persistiert eine stabile Node-ID für den HLC.
    fn get_or_create_node_id(conn: &mut Connection) -> Result<ID, HlcError> {
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let query = format!("SELECT value FROM {} WHERE key = ?1", TABLE_CRDT_CONFIGS);

        let id = match tx.query_row(&query, params![HLC_NODE_ID_TYPE], |row| {
            row.get::<_, Vec<u8>>(0)
        }) {
            Ok(id_bytes) => ID::try_from(id_bytes.as_slice())
                .map_err(|e| HlcError::Parse(format!("Invalid node ID format: {:?}", e)))?,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let new_id_bytes = Uuid::new_v4().as_bytes().to_vec();
                let new_id = ID::try_from(new_id_bytes.as_slice())?;

                tx.execute(
                    &format!(
                        "INSERT INTO {} (key, value) VALUES (?1, ?2)",
                        TABLE_CRDT_CONFIGS
                    ),
                    params![HLC_NODE_ID_TYPE, new_id_bytes.as_slice()],
                )?;
                new_id
            }
            Err(e) => return Err(HlcError::Database(e)),
        };

        tx.commit()?;
        Ok(id)
    }
}

impl Default for HlcService {
    fn default() -> Self {
        Self::new()
    }
}
