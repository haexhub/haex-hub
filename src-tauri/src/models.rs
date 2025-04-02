// models.rs
use serde::{Deserialize, Serialize};
//use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtensionManifest {
    pub name: String,
    pub entry: String,
    pub permissions: ExtensionPermissions,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExtensionPermissions {
    pub database: Option<DatabasePermissions>,
    pub http: Option<Vec<String>>,
    pub filesystem: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabasePermissions {
    pub read: Option<Vec<String>>,
    pub write: Option<Vec<String>>,
    pub create: Option<Vec<String>>,
}

/* #[derive(Default)]
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
} */

#[derive(Debug, Serialize, Deserialize)]
pub struct DbExtensionPermission {
    pub id: String,
    pub extension_id: String,
    pub resource: String,
    pub operation: String,
    pub path: String,
}
