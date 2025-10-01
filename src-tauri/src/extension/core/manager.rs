// src-tauri/src/extension/core/manager.rs

use crate::extension::core::manifest::{EditablePermissions, ExtensionManifest, ExtensionPreview};
use crate::extension::core::types::{copy_directory, Extension, ExtensionSource};
use crate::extension::crypto::ExtensionCrypto;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::{ExtensionPermission, PermissionStatus};
use crate::AppState;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager, State};
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct CachedPermission {
    pub permissions: Vec<ExtensionPermission>,
    pub cached_at: SystemTime,
    pub ttl: Duration,
}

#[derive(Default)]
pub struct ExtensionManager {
    pub production_extensions: Mutex<HashMap<String, Extension>>,
    pub dev_extensions: Mutex<HashMap<String, Extension>>,
    pub permission_cache: Mutex<HashMap<String, CachedPermission>>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_base_extension_dir(
        &self,
        app_handle: &AppHandle,
    ) -> Result<PathBuf, ExtensionError> {
        let path = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|e| ExtensionError::Filesystem {
                source: std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()),
            })?
            .join("extensions");
        Ok(path)
    }

    pub fn get_extension_dir(
        &self,
        app_handle: &AppHandle,
        extension_id: &str,
        extension_version: &str,
    ) -> Result<PathBuf, ExtensionError> {
        let specific_extension_dir = self
            .get_base_extension_dir(app_handle)?
            .join(extension_id)
            .join(extension_version);

        Ok(specific_extension_dir)
    }

    pub fn add_production_extension(&self, extension: Extension) -> Result<(), ExtensionError> {
        if extension.id.is_empty() {
            return Err(ExtensionError::ValidationError {
                reason: "Extension ID cannot be empty".to_string(),
            });
        }

        match &extension.source {
            ExtensionSource::Production { .. } => {
                let mut extensions = self.production_extensions.lock().unwrap();
                extensions.insert(extension.id.clone(), extension);
                Ok(())
            }
            _ => Err(ExtensionError::ValidationError {
                reason: "Expected Production source".to_string(),
            }),
        }
    }

    pub fn add_dev_extension(&self, extension: Extension) -> Result<(), ExtensionError> {
        if extension.id.is_empty() {
            return Err(ExtensionError::ValidationError {
                reason: "Extension ID cannot be empty".to_string(),
            });
        }

        match &extension.source {
            ExtensionSource::Development { .. } => {
                let mut extensions = self.dev_extensions.lock().unwrap();
                extensions.insert(extension.id.clone(), extension);
                Ok(())
            }
            _ => Err(ExtensionError::ValidationError {
                reason: "Expected Development source".to_string(),
            }),
        }
    }

    pub fn get_extension(&self, extension_id: &str) -> Option<Extension> {
        let dev_extensions = self.dev_extensions.lock().unwrap();
        if let Some(extension) = dev_extensions.get(extension_id) {
            return Some(extension.clone());
        }

        let prod_extensions = self.production_extensions.lock().unwrap();
        prod_extensions.get(extension_id).cloned()
    }

    pub fn remove_extension(&self, extension_id: &str) -> Result<(), ExtensionError> {
        {
            let mut dev_extensions = self.dev_extensions.lock().unwrap();
            if dev_extensions.remove(extension_id).is_some() {
                return Ok(());
            }
        }

        {
            let mut prod_extensions = self.production_extensions.lock().unwrap();
            if prod_extensions.remove(extension_id).is_some() {
                return Ok(());
            }
        }

        Err(ExtensionError::NotFound {
            id: extension_id.to_string(),
        })
    }

    pub async fn remove_extension_internal(
        &self,
        app_handle: &AppHandle,
        extension_id: String,
        extension_version: String,
        state: &State<'_, AppState>,
    ) -> Result<(), ExtensionError> {
        PermissionManager::delete_permissions(state, &extension_id).await?;
        self.remove_extension(&extension_id)?;

        let extension_dir =
            self.get_extension_dir(app_handle, &extension_id, &extension_version)?;

        if extension_dir.exists() {
            std::fs::remove_dir_all(&extension_dir)
                .map_err(|e| ExtensionError::Filesystem { source: e })?;
        }

        Ok(())
    }

    pub async fn preview_extension_internal(
        &self,
        source_path: String,
    ) -> Result<ExtensionPreview, ExtensionError> {
        let source = PathBuf::from(&source_path);

        let temp = std::env::temp_dir().join(format!("haexhub_preview_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).map_err(|e| ExtensionError::Filesystem { source: e })?;

        let file = File::open(&source).map_err(|e| ExtensionError::Filesystem { source: e })?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Invalid ZIP: {}", e),
            })?;

        archive
            .extract(&temp)
            .map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Cannot extract ZIP: {}", e),
            })?;

        let manifest_path = temp.join("manifest.json");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        let content_hash = ExtensionCrypto::hash_directory(&temp)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        let is_valid_signature = ExtensionCrypto::verify_signature(
            &manifest.public_key,
            &content_hash,
            &manifest.signature,
        )
        .is_ok();

        let key_hash = manifest.calculate_key_hash()?;
        let editable_permissions = manifest.to_editable_permissions();

        std::fs::remove_dir_all(&temp).ok();

        Ok(ExtensionPreview {
            manifest,
            is_valid_signature,
            key_hash,
            editable_permissions,
        })
    }

    pub async fn install_extension_with_permissions_internal(
        &self,
        app_handle: AppHandle,
        source_path: String,
        custom_permissions: EditablePermissions,
        state: &State<'_, AppState>,
    ) -> Result<String, ExtensionError> {
        let source = PathBuf::from(&source_path);

        let temp = std::env::temp_dir().join(format!("haexhub_ext_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).map_err(|e| ExtensionError::Filesystem { source: e })?;

        let file = File::open(&source).map_err(|e| ExtensionError::Filesystem { source: e })?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Invalid ZIP: {}", e),
            })?;

        archive
            .extract(&temp)
            .map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Cannot extract ZIP: {}", e),
            })?;

        let manifest_path = temp.join("manifest.json");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        let content_hash = ExtensionCrypto::hash_directory(&temp)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        ExtensionCrypto::verify_signature(&manifest.public_key, &content_hash, &manifest.signature)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        let key_hash = manifest.calculate_key_hash()?;
        let full_extension_id = format!("{}-{}", key_hash, manifest.id);

        let extensions_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| ExtensionError::Filesystem {
                source: std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()),
            })?
            .join("extensions")
            .join(&full_extension_id)
            .join(&manifest.version);

        std::fs::create_dir_all(&extensions_dir)
            .map_err(|e| ExtensionError::Filesystem { source: e })?;

        copy_directory(
            temp.to_string_lossy().to_string(),
            extensions_dir.to_string_lossy().to_string(),
        )?;

        std::fs::remove_dir_all(&temp).ok();

        let permissions = custom_permissions.to_internal_permissions(&full_extension_id);

        let granted_permissions: Vec<_> = permissions
            .into_iter()
            .filter(|p| p.status == PermissionStatus::Granted)
            .collect();

        PermissionManager::save_permissions(state, &full_extension_id, &granted_permissions)
            .await?;

        let extension = Extension {
            id: full_extension_id.clone(),
            name: manifest.name.clone(),
            source: ExtensionSource::Production {
                path: extensions_dir.clone(),
                version: manifest.version.clone(),
            },
            manifest: manifest.clone(),
            enabled: true,
            last_accessed: SystemTime::now(),
        };

        self.add_production_extension(extension)?;

        Ok(full_extension_id)
    }
}

// Backward compatibility
#[derive(Default)]
pub struct ExtensionState {
    pub extensions: Mutex<HashMap<String, ExtensionManifest>>,
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
