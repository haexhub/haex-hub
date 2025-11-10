// src-tauri/src/extension/error.rs
use thiserror::Error;
use ts_rs::TS;

use crate::database::error::DatabaseError;

/// Error codes for frontend handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
pub enum ExtensionErrorCode {
    SecurityViolation = 1000,
    NotFound = 1001,
    PermissionDenied = 1002,
    MutexPoisoned = 1003,
    Database = 2000,
    Filesystem = 2001,
    FilesystemWithPath = 2004,
    Http = 2002,
    Shell = 2003,
    Manifest = 3000,
    Validation = 3001,
    InvalidPublicKey = 4000,
    InvalidSignature = 4001,
    InvalidActionString = 4004,
    SignatureVerificationFailed = 4002,
    CalculateHash = 4003,
    Installation = 5000,
}

/// Serialized representation of ExtensionError for TypeScript
#[derive(Debug, Clone, serde::Serialize, TS)]
#[ts(export)]
pub struct SerializedExtensionError {
    pub code: u16,
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub extension_id: Option<String>,
}

impl serde::Serialize for ExtensionErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(*self as u16)
    }
}

#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Extension not found: {name} (public_key: {public_key})")]
    NotFound { public_key: String, name: String },

    #[error("Permission denied: {extension_id} cannot {operation} on {resource}")]
    PermissionDenied {
        extension_id: String,
        operation: String,
        resource: String,
    },

    #[error("Database operation failed: {source}")]
    Database {
        #[from]
        source: DatabaseError,
    },

    #[error("Filesystem operation failed: {source}")]
    Filesystem {
        #[from]
        source: std::io::Error,
    },

    #[error("Filesystem operation failed at '{path}': {source}")]
    FilesystemWithPath {
        path: String,
        source: std::io::Error,
    },

    #[error("HTTP request failed: {reason}")]
    Http { reason: String },

    #[error("Shell command failed: {reason}")]
    Shell {
        reason: String,
        exit_code: Option<i32>,
    },

    #[error("Manifest error: {reason}")]
    ManifestError { reason: String },

    #[error("Validation error: {reason}")]
    ValidationError { reason: String },

    #[error("Invalid Public Key: {reason}")]
    InvalidPublicKey { reason: String },

    #[error("Invalid Action: {input} for resource {resource_type}")]
    InvalidActionString {
        input: String,
        resource_type: String,
    },

    #[error("Invalid Signature: {reason}")]
    InvalidSignature { reason: String },

    #[error("Error during hash calculation: {reason}")]
    CalculateHashError { reason: String },

    #[error("Signature verification failed: {reason}")]
    SignatureVerificationFailed { reason: String },

    #[error("Extension installation failed: {reason}")]
    InstallationFailed { reason: String },

    #[error("A mutex was poisoned: {reason}")]
    MutexPoisoned { reason: String },
}

impl ExtensionError {
    /// Get error code for this error
    pub fn code(&self) -> ExtensionErrorCode {
        match self {
            ExtensionError::SecurityViolation { .. } => ExtensionErrorCode::SecurityViolation,
            ExtensionError::NotFound { .. } => ExtensionErrorCode::NotFound,
            ExtensionError::PermissionDenied { .. } => ExtensionErrorCode::PermissionDenied,
            ExtensionError::Database { .. } => ExtensionErrorCode::Database,
            ExtensionError::Filesystem { .. } => ExtensionErrorCode::Filesystem,
            ExtensionError::FilesystemWithPath { .. } => ExtensionErrorCode::FilesystemWithPath,
            ExtensionError::Http { .. } => ExtensionErrorCode::Http,
            ExtensionError::Shell { .. } => ExtensionErrorCode::Shell,
            ExtensionError::ManifestError { .. } => ExtensionErrorCode::Manifest,
            ExtensionError::ValidationError { .. } => ExtensionErrorCode::Validation,
            ExtensionError::InvalidPublicKey { .. } => ExtensionErrorCode::InvalidPublicKey,
            ExtensionError::InvalidSignature { .. } => ExtensionErrorCode::InvalidSignature,
            ExtensionError::SignatureVerificationFailed { .. } => {
                ExtensionErrorCode::SignatureVerificationFailed
            }
            ExtensionError::InstallationFailed { .. } => ExtensionErrorCode::Installation,
            ExtensionError::CalculateHashError { .. } => ExtensionErrorCode::CalculateHash,
            ExtensionError::MutexPoisoned { .. } => ExtensionErrorCode::MutexPoisoned,
            ExtensionError::InvalidActionString { .. } => ExtensionErrorCode::InvalidActionString,
        }
    }

    pub fn permission_denied(extension_id: &str, operation: &str, resource: &str) -> Self {
        Self::PermissionDenied {
            extension_id: extension_id.to_string(),
            operation: operation.to_string(),
            resource: resource.to_string(),
        }
    }

    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            ExtensionError::PermissionDenied { .. } | ExtensionError::SecurityViolation { .. }
        )
    }

    pub fn extension_id(&self) -> Option<&str> {
        match self {
            ExtensionError::PermissionDenied { extension_id, .. } => Some(extension_id),
            _ => None,
        }
    }

    /// Helper to create a filesystem error with path context
    pub fn filesystem_with_path<P: Into<String>>(path: P, source: std::io::Error) -> Self {
        Self::FilesystemWithPath {
            path: path.into(),
            source,
        }
    }
}

impl serde::Serialize for ExtensionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ExtensionError", 4)?;

        state.serialize_field("code", &self.code())?;
        state.serialize_field("type", &format!("{self:?}"))?;
        state.serialize_field("message", &self.to_string())?;

        if let Some(ext_id) = self.extension_id() {
            state.serialize_field("extension_id", ext_id)?;
        } else {
            state.serialize_field("extension_id", &Option::<String>::None)?;
        }

        state.end()
    }
}

impl From<ExtensionError> for String {
    fn from(error: ExtensionError) -> Self {
        serde_json::to_string(&error).unwrap_or_else(|_| error.to_string())
    }
}

impl From<serde_json::Error> for ExtensionError {
    fn from(err: serde_json::Error) -> Self {
        ExtensionError::ManifestError {
            reason: err.to_string(),
        }
    }
}
