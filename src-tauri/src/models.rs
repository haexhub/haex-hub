// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/* #[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub entry: String,
    pub icon: Option<String>,
    pub permissions: ExtensionPermissions,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionPermissions {
    pub database: Option<DatabasePermissions>,
    pub http: Option<Vec<String>>,
    pub filesystem: Option<String>,
}

/// Enum to represent the source of an extension
#[derive(Debug, Clone)]
pub enum ExtensionSource {
    /// Production extension installed in app data
    Production { path: PathBuf, version: String },
    /// Development mode extension with live reloading
    Development {
        dev_server_url: String,
        manifest_path: PathBuf,
        auto_reload: bool,
    },
} */

/*
#[derive(Default)]
pub struct ExtensionState {
    pub extensions: Mutex<std::collections::HashMap<String, ExtensionManifest>>,
}

impl ExtensionState {
    pub fn add_extension(&self, path: String, manifest: ExtensionManifest) {
        let mut extensions = self.extensions.lock().unwrap();
        extensions.insert(path, manifest);
    }

    pub fn get_extension(&self, addon_id: &str) -> Option<ExtensionManifest> {
        let extensions = self.extensions.lock().unwrap();
        extensions.values().find(|p| p.name == addon_id).cloned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbExtensionPermission {
    pub id: String,
    pub extension_id: String,
    pub resource: String,
    pub operation: String,
    pub path: String,
}

/// Comprehensive error type for all extension-related operations
#[derive(Error, Debug)]
pub enum ExtensionError {
    /// Security violation detected
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    /// Extension not found
    #[error("Extension not found: {id}")]
    NotFound { id: String },

    /// Permission denied
    #[error("Permission denied: {extension_id} cannot {operation} on {resource}")]
    PermissionDenied {
        extension_id: String,
        operation: String,
        resource: String,
    },

    /// IO error during extension operations
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Error during extension manifest parsing
    #[error("Manifest error: {reason}")]
    ManifestError { reason: String },

    /// Input validation error
    #[error("Validation error: {reason}")]
    ValidationError { reason: String },

    /// Development server error
    #[error("Dev server error: {reason}")]
    DevServerError { reason: String },
}
 */
