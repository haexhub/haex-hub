// src/hlc_service.rs

use rusqlite::{params, Connection, Result as RusqliteResult, Transaction};
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

pub const CRDT_SETTINGS_TABLE: &str = "haex_crdt_settings";

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
}

/// A thread-safe, persistent HLC service.
#[derive(Clone)]
pub struct HlcService(Arc<Mutex<HLC>>);

impl HlcService {
    /// Creates a new HLC service, initializing it from the database or creating a new
    /// persistent identity if one does not exist.
    pub fn new(conn: &mut Connection) -> Result<Self, HlcError> {
        // 1. Manage persistent node identity.
        let node_id = Self::get_or_create_node_id(conn)?;

        // 2. Create HLC instance with stable identity using the HLCBuilder.
        let hlc = HLCBuilder::new()
            .with_id(node_id)
            .with_max_delta(Duration::from_secs(1)) // Example of custom configuration
            .build();

        // 3. Load the last persisted timestamp and update the clock.
        let last_state_str: RusqliteResult<String> = conn.query_row(
            &format!("SELECT value FROM {} WHERE type = ?1", CRDT_SETTINGS_TABLE),
            params![HLC_TIMESTAMP_TYPE],
            |row| row.get(0),
        );

        if let Ok(state_str) = last_state_str {
            let timestamp =
                Timestamp::from_str(&state_str).map_err(|e| HlcError::ParseTimestamp(e.cause))?;

            // Update the clock with the persisted state.
            // we might want to handle the error case where the clock drifts too far.
            hlc.update_with_timestamp(&timestamp)
                .map_err(|e| HlcError::Parse(e.to_string()))?;
        }

        let hlc_arc = Arc::new(Mutex::new(hlc));
        Ok(HlcService(hlc_arc))
    }

    /// Generates a new timestamp and immediately persists the HLC's new state.
    /// This method MUST be called within an existing database transaction (`tx`)
    /// along with the actual data operation that this timestamp is for.
    /// This design ensures atomicity: the data is saved with its timestamp,
    /// and the clock state is updated, or none of it is.
    pub fn new_timestamp_and_persist<'tx>(
        &self,
        tx: &Transaction<'tx>,
    ) -> Result<Timestamp, HlcError> {
        let hlc = self.0.lock().map_err(|_| HlcError::MutexPoisoned)?;
        let new_timestamp = hlc.new_timestamp();
        let timestamp_str = new_timestamp.to_string();

        tx.execute(
            &format!(
                "INSERT INTO {} (type, value) VALUES (?1,?2)
                 ON CONFLICT(type) DO UPDATE SET value = excluded.value",
                CRDT_SETTINGS_TABLE
            ),
            params![HLC_TIMESTAMP_TYPE, timestamp_str],
        )?;

        Ok(new_timestamp)
    }

    /// Retrieves or creates and persists a stable node ID for the HLC.
    fn get_or_create_node_id(conn: &mut Connection) -> Result<ID, HlcError> {
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;

        let query = format!("SELECT value FROM {} WHERE type =?1", CRDT_SETTINGS_TABLE);

        match tx.query_row(&query, params![HLC_NODE_ID_TYPE], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(id_str) => {
                // ID exists, parse and return it.
                let id_bytes = hex::decode(id_str).map_err(|e| HlcError::Parse(e.to_string()))?;
                let id = ID::try_from(id_bytes.as_slice())?;
                tx.commit()?;
                Ok(id)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No ID found, create, persist, and return a new one.
                let new_id_bytes = Uuid::new_v4().as_bytes().to_vec();
                let new_id = ID::try_from(new_id_bytes.as_slice())?;
                let new_id_str = hex::encode(new_id.to_le_bytes());

                tx.execute(
                    &format!(
                        "INSERT INTO {} (type, value) VALUES (?1, ?2)",
                        CRDT_SETTINGS_TABLE
                    ),
                    params![HLC_NODE_ID_TYPE, new_id_str],
                )?;
                tx.commit()?;
                Ok(new_id)
            }
            Err(e) => Err(HlcError::from(e)),
        }
    }
}
