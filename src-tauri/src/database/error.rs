// src-tauri/src/database/error.rs

use crate::crdt::trigger::CrdtSetupError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "details")]
pub enum DatabaseError {
    /// Der SQL-Code konnte nicht geparst werden.
    #[error("Failed to parse SQL: {reason} - SQL: {sql}")]
    ParseError { reason: String, sql: String },

    /// Parameter-Fehler (falsche Anzahl, ungültiger Typ, etc.)
    #[error("Parameter count mismatch: SQL has {expected} placeholders but {provided} provided. SQL Statement: {sql}")]
    ParameterMismatchError {
        expected: usize,
        provided: usize,
        sql: String,
    },

    #[error("No table provided in SQL Statement: {sql}")]
    NoTableError { sql: String },

    #[error("Statement Error: {reason}")]
    StatementError { reason: String },

    #[error("Failed to prepare statement: {reason}")]
    PrepareError { reason: String },

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    /// Ein Fehler ist während der Ausführung in der Datenbank aufgetreten.
    #[error("Execution error on table {table:?}: {reason} - SQL: {sql}")]
    ExecutionError {
        sql: String,
        reason: String,
        table: Option<String>,
    },
    /// Ein Fehler ist beim Verwalten der Transaktion aufgetreten.
    #[error("Transaction error: {reason}")]
    TransactionError { reason: String },

    /// Ein SQL-Statement wird vom Proxy nicht unterstützt.
    #[error("Unsupported statement. '{reason}'. - SQL: {sql}")]
    UnsupportedStatement { reason: String, sql: String },

    /// Fehler im HLC-Service
    #[error("HLC error: {reason}")]
    HlcError { reason: String },

    /// Fehler beim Sperren der Datenbankverbindung
    #[error("Lock error: {reason}")]
    LockError { reason: String },

    /// Fehler bei der Datenbankverbindung
    #[error("Connection error: {reason}")]
    ConnectionError { reason: String },

    /// Fehler bei der JSON-Serialisierung
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    /// Permission-bezogener Fehler für Extensions
    #[error("Permission error for extension '{extension_id}': {reason} (operation: {operation:?}, resource: {resource:?})")]
    PermissionError {
        extension_id: String,
        operation: Option<String>,
        resource: Option<String>,
        reason: String,
    },

    #[error("Query error: {reason}")]
    QueryError { reason: String },

    #[error("Row processing error: {reason}")]
    RowProcessingError { reason: String },

    #[error("Mutex Poisoned error: {reason}")]
    MutexPoisoned { reason: String },

    #[error("Datenbankverbindung fehlgeschlagen für Pfad '{path}': {reason}")]
    ConnectionFailed { path: String, reason: String },

    #[error("PRAGMA-Befehl '{pragma}' konnte nicht gesetzt werden: {reason}")]
    PragmaError { pragma: String, reason: String },

    #[error("Fehler beim Auflösen des Dateipfads: {reason}")]
    PathResolutionError { reason: String },

    #[error("Datei-I/O-Fehler für Pfad '{path}': {reason}")]
    IoError { path: String, reason: String },

    #[error("CRDT setup failed: {0}")]
    CrdtSetup(String),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::DatabaseError {
            reason: err.to_string(),
        }
    }
}

impl From<String> for DatabaseError {
    fn from(reason: String) -> Self {
        DatabaseError::DatabaseError { reason }
    }
}

impl From<CrdtSetupError> for DatabaseError {
    fn from(err: CrdtSetupError) -> Self {
        // Wir konvertieren den Fehler in einen String, um ihn einfach zu halten.
        DatabaseError::CrdtSetup(err.to_string())
    }
}

impl DatabaseError {
    /// Extract extension ID if this error is related to an extension
    pub fn extension_id(&self) -> Option<&str> {
        match self {
            DatabaseError::PermissionError { extension_id, .. } => Some(extension_id.as_str()),
            _ => None,
        }
    }

    /// Check if this is a permission-related error
    pub fn is_permission_error(&self) -> bool {
        matches!(self, DatabaseError::PermissionError { .. })
    }

    /// Get operation if available
    pub fn operation(&self) -> Option<&str> {
        match self {
            DatabaseError::PermissionError {
                operation: Some(op),
                ..
            } => Some(op.as_str()),
            _ => None,
        }
    }

    /// Get resource if available
    pub fn resource(&self) -> Option<&str> {
        match self {
            DatabaseError::PermissionError {
                resource: Some(res),
                ..
            } => Some(res.as_str()),
            _ => None,
        }
    }
}
/* impl From<crate::extension::database::ExtensionDatabaseError> for DatabaseError {
    fn from(err: crate::extension::database::ExtensionDatabaseError) -> Self {
        match err {
            crate::extension::database::ExtensionDatabaseError::Permission { source } => {
                // Konvertiere PermissionError zu DatabaseError
                match source {
                    crate::extension::database::permissions::PermissionError::AccessDenied {
                        extension_id,
                        operation,
                        resource,
                        reason,
                    } => DatabaseError::PermissionError {
                        extension_id,
                        operation: Some(operation),
                        resource: Some(resource),
                        reason,
                    },
                    crate::extension::database::permissions::PermissionError::Database {
                        source,
                    } => source,
                    other => DatabaseError::PermissionError {
                        extension_id: "unknown".to_string(),
                        operation: None,
                        resource: None,
                        reason: other.to_string(),
                    },
                }
            }
            crate::extension::database::ExtensionDatabaseError::Database { source } => source,
            crate::extension::database::ExtensionDatabaseError::ParameterValidation { reason } => {
                DatabaseError::ParamError {
                    reason: reason.clone(),
                    expected: 0, // Kann nicht aus dem Grund extrahiert werden
                    provided: 0, // Kann nicht aus dem Grund extrahiert werden
                }
            }
            crate::extension::database::ExtensionDatabaseError::StatementExecution { reason } => {
                DatabaseError::ExecutionError {
                    sql: "unknown".to_string(),
                    reason,
                    table: None,
                }
            }
        }
    }
} */
