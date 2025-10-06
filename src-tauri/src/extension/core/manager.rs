use crate::database::core::with_connection;
use crate::database::error::DatabaseError;
use crate::extension::core::manifest::{EditablePermissions, ExtensionManifest, ExtensionPreview};
use crate::extension::core::types::{copy_directory, Extension, ExtensionSource};
use crate::extension::core::ExtensionPermissions;
use crate::extension::crypto::ExtensionCrypto;
use crate::extension::database::executor::SqlExecutor;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::ExtensionPermission;
use crate::table_names::TABLE_EXTENSIONS;
use crate::AppState;
use std::collections::HashMap;
use std::fs::{self, File};
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

#[derive(Debug, Clone)]
pub struct MissingExtension {
    pub full_extension_id: String,
    pub name: String,
    pub version: String,
}

struct ExtensionDataFromDb {
    manifest: ExtensionManifest,
    enabled: bool,
}

#[derive(Default)]
pub struct ExtensionManager {
    pub production_extensions: Mutex<HashMap<String, Extension>>,
    pub dev_extensions: Mutex<HashMap<String, Extension>>,
    pub permission_cache: Mutex<HashMap<String, CachedPermission>>,
    pub missing_extensions: Mutex<Vec<MissingExtension>>,
}

struct ExtractedExtension {
    temp_dir: PathBuf,
    manifest: ExtensionManifest,
    content_hash: String,
}

impl Drop for ExtractedExtension {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.temp_dir).ok();
    }
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extrahiert eine Extension-ZIP-Datei und validiert das Manifest
    fn extract_and_validate_extension(
        source_path: &str,
        temp_prefix: &str,
    ) -> Result<ExtractedExtension, ExtensionError> {
        let source = PathBuf::from(source_path);
        let temp = std::env::temp_dir().join(format!("{}_{}", temp_prefix, uuid::Uuid::new_v4()));

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

        let content_hash = ExtensionCrypto::hash_directory(&temp).map_err(|e| {
            ExtensionError::SignatureVerificationFailed {
                reason: e.to_string(),
            }
        })?;

        Ok(ExtractedExtension {
            temp_dir: temp,
            manifest,
            content_hash,
        })
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

        // Sicherstellen, dass das Basisverzeichnis existiert
        if !path.exists() {
            fs::create_dir_all(&path).map_err(|e| ExtensionError::Filesystem { source: e })?;
        }
        Ok(path)
    }

    pub fn get_extension_dir(
        &self,
        app_handle: &AppHandle,
        key_hash: &str,
        extension_name: &str,
        extension_version: &str,
    ) -> Result<PathBuf, ExtensionError> {
        let specific_extension_dir = self
            .get_base_extension_dir(app_handle)?
            .join(key_hash)
            .join(extension_name)
            .join(extension_version);

        Ok(specific_extension_dir)
    }

    pub fn get_extension_path_by_full_extension_id(
        &self,
        app_handle: &AppHandle,
        full_extension_id: &str,
    ) -> Result<PathBuf, ExtensionError> {
        let specific_extension_dir = self
            .get_base_extension_dir(app_handle)?
            .join(full_extension_id);

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
        key_hash: &str,
        extension_id: &str,
        extension_version: &str,
        state: &State<'_, AppState>,
    ) -> Result<(), ExtensionError> {
        // Lösche Permissions und Extension-Eintrag in einer Transaktion
        with_connection(&state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
                reason: "Failed to lock HLC service".to_string(),
            })?;

            // Lösche alle Permissions
            PermissionManager::delete_permissions_in_transaction(&tx, &hlc_service, extension_id)?;

            // Lösche Extension-Eintrag
            let sql = format!("DELETE FROM {} WHERE id = ?", TABLE_EXTENSIONS);
            SqlExecutor::execute_internal_typed(
                &tx,
                &hlc_service,
                &sql,
                rusqlite::params![extension_id],
            )?;

            tx.commit().map_err(DatabaseError::from)
        })?;

        // Entferne aus dem In-Memory-Manager
        self.remove_extension(&extension_id)?;

        // Lösche Dateien vom Dateisystem
        let extension_dir =
            self.get_extension_dir(app_handle, key_hash, extension_id, extension_version)?;

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
        let extracted = Self::extract_and_validate_extension(&source_path, "haexhub_preview")?;

        let is_valid_signature = ExtensionCrypto::verify_signature(
            &extracted.manifest.public_key,
            &extracted.content_hash,
            &extracted.manifest.signature,
        )
        .is_ok();

        let key_hash = extracted.manifest.calculate_key_hash()?;
        let editable_permissions = extracted.manifest.to_editable_permissions();

        Ok(ExtensionPreview {
            manifest: extracted.manifest.clone(),
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
        let extracted = Self::extract_and_validate_extension(&source_path, "haexhub_ext")?;

        // Signatur verifizieren (bei Installation wird ein Fehler geworfen, nicht nur geprüft)
        ExtensionCrypto::verify_signature(
            &extracted.manifest.public_key,
            &extracted.content_hash,
            &extracted.manifest.signature,
        )
        .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        let full_extension_id = extracted.manifest.full_extension_id()?;

        let extensions_dir = self.get_extension_dir(
            &app_handle,
            &extracted.manifest.calculate_key_hash()?,
            &extracted.manifest.name,
            &extracted.manifest.version,
        )?;

        std::fs::create_dir_all(&extensions_dir)
            .map_err(|e| ExtensionError::Filesystem { source: e })?;

        copy_directory(
            extracted.temp_dir.to_string_lossy().to_string(),
            extensions_dir.to_string_lossy().to_string(),
        )?;

        let permissions = custom_permissions.to_internal_permissions(&full_extension_id);

        PermissionManager::save_permissions(state, &permissions).await?;

        let extension = Extension {
            id: full_extension_id.clone(),
            name: extracted.manifest.name.clone(),
            source: ExtensionSource::Production {
                path: extensions_dir.clone(),
                version: extracted.manifest.version.clone(),
            },
            manifest: extracted.manifest.clone(),
            enabled: true,
            last_accessed: SystemTime::now(),
        };

        self.add_production_extension(extension)?;

        Ok(full_extension_id)
    }

    /// Scannt das Dateisystem beim Start und lädt alle installierten Erweiterungen.
    pub async fn load_installed_extensions(
        &self,
        app_handle: &AppHandle,
        state: &State<'_, AppState>,
    ) -> Result<Vec<String>, ExtensionError> {
        self.production_extensions
            .lock()
            .map_err(|e| ExtensionError::MutexPoisoned {
                reason: e.to_string(),
            })?
            .clear();
        self.permission_cache
            .lock()
            .map_err(|e| ExtensionError::MutexPoisoned {
                reason: e.to_string(),
            })?
            .clear();
        self.missing_extensions
            .lock()
            .map_err(|e| ExtensionError::MutexPoisoned {
                reason: e.to_string(),
            })?
            .clear();

        // Schritt 1: Alle Daten aus der Datenbank in einem Rutsch laden.
        let extensions = with_connection(&state.db, |conn| {
            let sql = "SELECT id, name, version, author, entry, icon, public_key, signature, homepage, description, enabled FROM haexExtensions";
            let results = SqlExecutor::select_internal(conn, sql, &[])?;

            let mut data = Vec::new();
            for result in results {
                let manifest = ExtensionManifest {
                    id: result["id"]
                        .as_str()
                        .ok_or_else(|| DatabaseError::SerializationError {
                            reason: "Missing id field".to_string(),
                        })?
                        .to_string(),
                    name: result["name"]
                        .as_str()
                        .ok_or_else(|| DatabaseError::SerializationError {
                            reason: "Missing name field".to_string(),
                        })?
                        .to_string(),
                    version: result["version"]
                        .as_str()
                        .ok_or_else(|| DatabaseError::SerializationError {
                            reason: "Missing version field".to_string(),
                        })?
                        .to_string(),
                    author: result["author"].as_str().map(String::from),
                    entry: result["entry"].as_str().unwrap_or("index.html").to_string(),
                    icon: result["icon"].as_str().map(String::from),
                    public_key: result["public_key"].as_str().unwrap_or("").to_string(),
                    signature: result["signature"].as_str().unwrap_or("").to_string(),
                    permissions: ExtensionPermissions::default(),
                    homepage: result["homepage"].as_str().map(String::from),
                    description: result["description"].as_str().map(String::from),
                };

                let enabled = result["enabled"]
                    .as_bool()
                    .or_else(|| result["enabled"].as_i64().map(|v| v != 0))
                    .unwrap_or(false);

                data.push(ExtensionDataFromDb { manifest, enabled });
            }
            Ok(data)
        })?;

        // Schritt 2: Die gesammelten Daten verarbeiten (Dateisystem, State-Mutationen).
        let mut loaded_extension_ids = Vec::new();

        for extension in extensions {
            let full_extension_id = extension.manifest.full_extension_id()?;
            let extension_path =
                self.get_extension_path_by_full_extension_id(app_handle, &full_extension_id)?;

            if !extension_path.exists() || !extension_path.join("manifest.json").exists() {
                self.missing_extensions
                    .lock()
                    .map_err(|e| ExtensionError::MutexPoisoned {
                        reason: e.to_string(),
                    })?
                    .push(MissingExtension {
                        full_extension_id: full_extension_id.clone(),
                        name: extension.manifest.name.clone(),
                        version: extension.manifest.version.clone(),
                    });
                continue;
            }

            let extension = Extension {
                id: full_extension_id.clone(),
                name: extension.manifest.name.clone(),
                source: ExtensionSource::Production {
                    path: extension_path,
                    version: extension.manifest.version.clone(),
                },
                manifest: extension.manifest,
                enabled: extension.enabled,
                last_accessed: SystemTime::now(),
            };

            self.add_production_extension(extension)?;
            loaded_extension_ids.push(full_extension_id);
        }

        Ok(loaded_extension_ids)
    }
}
