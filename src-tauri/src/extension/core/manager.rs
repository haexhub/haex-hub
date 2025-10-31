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
use crate::table_names::{TABLE_EXTENSIONS, TABLE_EXTENSION_PERMISSIONS};
use crate::AppState;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
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
    pub id: String,
    pub public_key: String,
    pub name: String,
    pub version: String,
}

struct ExtensionDataFromDb {
    id: String,
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

    /// Helper function to validate path and check for path traversal
    /// Returns the cleaned path if valid, or None if invalid/not found
    /// If require_exists is true, returns None if path doesn't exist
    pub fn validate_path_in_directory(
        base_dir: &PathBuf,
        relative_path: &str,
        require_exists: bool,
    ) -> Result<Option<PathBuf>, ExtensionError> {
        // Check for path traversal patterns
        if relative_path.contains("..") {
            return Err(ExtensionError::SecurityViolation {
                reason: format!("Path traversal attempt: {}", relative_path),
            });
        }

        // Clean the path (same logic as in protocol.rs)
        let clean_path = relative_path
            .replace('\\', "/")
            .trim_start_matches('/')
            .split('/')
            .filter(|&part| !part.is_empty() && part != "." && part != "..")
            .collect::<PathBuf>();

        let full_path = base_dir.join(&clean_path);

        // Check if file/directory exists (if required)
        if require_exists && !full_path.exists() {
            return Ok(None);
        }

        // Verify path is within base directory
        let canonical_base = base_dir
            .canonicalize()
            .map_err(|e| ExtensionError::Filesystem { source: e })?;

        if let Ok(canonical_path) = full_path.canonicalize() {
            if !canonical_path.starts_with(&canonical_base) {
                return Err(ExtensionError::SecurityViolation {
                    reason: format!("Path outside base directory: {}", relative_path),
                });
            }
            Ok(Some(canonical_path))
        } else {
            // Path doesn't exist yet - still validate it would be within base
            if full_path.starts_with(&canonical_base) {
                Ok(Some(full_path))
            } else {
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Path outside base directory: {}", relative_path),
                })
            }
        }
    }

    /// Validates icon path and falls back to favicon.ico if not specified
    fn validate_and_resolve_icon_path(
        extension_dir: &PathBuf,
        haextension_dir: &str,
        icon_path: Option<&str>,
    ) -> Result<Option<String>, ExtensionError> {
        // If icon is specified in manifest, validate it
        if let Some(icon) = icon_path {
            if let Some(clean_path) = Self::validate_path_in_directory(extension_dir, icon, true)? {
                return Ok(Some(clean_path.to_string_lossy().to_string()));
            } else {
                eprintln!("WARNING: Icon path specified in manifest not found: {}", icon);
                // Continue to fallback logic
            }
        }

        // Fallback 1: Check haextension/favicon.ico
        let haextension_favicon = format!("{}/favicon.ico", haextension_dir);
        if let Some(clean_path) = Self::validate_path_in_directory(extension_dir, &haextension_favicon, true)? {
            return Ok(Some(clean_path.to_string_lossy().to_string()));
        }

        // Fallback 2: Check public/favicon.ico
        if let Some(clean_path) = Self::validate_path_in_directory(extension_dir, "public/favicon.ico", true)? {
            return Ok(Some(clean_path.to_string_lossy().to_string()));
        }

        // No icon found
        Ok(None)
    }

    /// Extrahiert eine Extension-ZIP-Datei und validiert das Manifest
    fn extract_and_validate_extension(
        bytes: Vec<u8>,
        temp_prefix: &str,
        app_handle: &AppHandle,
    ) -> Result<ExtractedExtension, ExtensionError> {
        // Use app_cache_dir for better Android compatibility
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Cannot get app cache dir: {}", e),
            })?;

        let temp_id = uuid::Uuid::new_v4();
        let temp = cache_dir.join(format!("{}_{}", temp_prefix, temp_id));
        let zip_file_path = cache_dir.join(format!("{}_{}_{}.haextension", temp_prefix, temp_id, "temp"));

        // Write bytes to a temporary ZIP file first (important for Android file system)
        fs::write(&zip_file_path, &bytes).map_err(|e| {
            ExtensionError::filesystem_with_path(zip_file_path.display().to_string(), e)
        })?;

        // Create extraction directory
        fs::create_dir_all(&temp)
            .map_err(|e| ExtensionError::filesystem_with_path(temp.display().to_string(), e))?;

        // Open ZIP file from disk (more reliable on Android than from memory)
        let zip_file = fs::File::open(&zip_file_path).map_err(|e| {
            ExtensionError::filesystem_with_path(zip_file_path.display().to_string(), e)
        })?;

        let mut archive = ZipArchive::new(zip_file).map_err(|e| {
            ExtensionError::InstallationFailed {
                reason: format!("Invalid ZIP: {}", e),
            }
        })?;

        archive
            .extract(&temp)
            .map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Cannot extract ZIP: {}", e),
            })?;

        // Clean up temporary ZIP file
        let _ = fs::remove_file(&zip_file_path);

        // Read haextension_dir from config if it exists, otherwise use default
        let config_path = temp.join("haextension.config.json");
        let haextension_dir = if config_path.exists() {
            let config_content = std::fs::read_to_string(&config_path)
                .map_err(|e| ExtensionError::ManifestError {
                    reason: format!("Cannot read haextension.config.json: {}", e),
                })?;

            let config: serde_json::Value = serde_json::from_str(&config_content)
                .map_err(|e| ExtensionError::ManifestError {
                    reason: format!("Invalid haextension.config.json: {}", e),
                })?;

            let dir = config
                .get("dev")
                .and_then(|dev| dev.get("haextension_dir"))
                .and_then(|dir| dir.as_str())
                .unwrap_or("haextension")
                .to_string();

            dir
        } else {
            "haextension".to_string()
        };

        // Validate manifest path using helper function
        let manifest_relative_path = format!("{}/manifest.json", haextension_dir);
        let manifest_path = Self::validate_path_in_directory(&temp, &manifest_relative_path, true)?
            .ok_or_else(|| ExtensionError::ManifestError {
                reason: format!("manifest.json not found at {}/manifest.json", haextension_dir),
            })?;

        let actual_dir = temp.clone();
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let mut manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        // Validate and resolve icon path with fallback logic
        let validated_icon = Self::validate_and_resolve_icon_path(&actual_dir, &haextension_dir, manifest.icon.as_deref())?;
        manifest.icon = validated_icon;

        let content_hash = ExtensionCrypto::hash_directory(&actual_dir, &manifest_path).map_err(|e| {
            ExtensionError::SignatureVerificationFailed {
                reason: e.to_string(),
            }
        })?;

        Ok(ExtractedExtension {
            temp_dir: actual_dir,
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
            fs::create_dir_all(&path)
                .map_err(|e| ExtensionError::filesystem_with_path(path.display().to_string(), e))?;
        }
        Ok(path)
    }

    pub fn get_extension_dir(
        &self,
        app_handle: &AppHandle,
        public_key: &str,
        extension_name: &str,
        extension_version: &str,
    ) -> Result<PathBuf, ExtensionError> {
        let specific_extension_dir = self
            .get_base_extension_dir(app_handle)?
            .join(public_key)
            .join(extension_name)
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

    /// Find extension ID by public_key and name (checks dev extensions first, then production)
    fn find_extension_id_by_public_key_and_name(
        &self,
        public_key: &str,
        name: &str,
    ) -> Result<Option<(String, Extension)>, ExtensionError> {
        // 1. Check dev extensions first (higher priority)
        let dev_extensions =
            self.dev_extensions
                .lock()
                .map_err(|e| ExtensionError::MutexPoisoned {
                    reason: e.to_string(),
                })?;

        for (id, ext) in dev_extensions.iter() {
            if ext.manifest.public_key == public_key && ext.manifest.name == name {
                return Ok(Some((id.clone(), ext.clone())));
            }
        }

        // 2. Check production extensions
        let prod_extensions =
            self.production_extensions
                .lock()
                .map_err(|e| ExtensionError::MutexPoisoned {
                    reason: e.to_string(),
                })?;

        for (id, ext) in prod_extensions.iter() {
            if ext.manifest.public_key == public_key && ext.manifest.name == name {
                return Ok(Some((id.clone(), ext.clone())));
            }
        }

        Ok(None)
    }

    /// Get extension by public_key and name (used by frontend)
    pub fn get_extension_by_public_key_and_name(
        &self,
        public_key: &str,
        name: &str,
    ) -> Result<Option<Extension>, ExtensionError> {
        Ok(self
            .find_extension_id_by_public_key_and_name(public_key, name)?
            .map(|(_, ext)| ext))
    }

    pub fn remove_extension(&self, public_key: &str, name: &str) -> Result<(), ExtensionError> {
        let (id, _) = self
            .find_extension_id_by_public_key_and_name(public_key, name)?
            .ok_or_else(|| ExtensionError::NotFound {
                public_key: public_key.to_string(),
                name: name.to_string(),
            })?;

        // Remove from dev extensions first
        {
            let mut dev_extensions =
                self.dev_extensions
                    .lock()
                    .map_err(|e| ExtensionError::MutexPoisoned {
                        reason: e.to_string(),
                    })?;
            if dev_extensions.remove(&id).is_some() {
                return Ok(());
            }
        }

        // Remove from production extensions
        {
            let mut prod_extensions =
                self.production_extensions
                    .lock()
                    .map_err(|e| ExtensionError::MutexPoisoned {
                        reason: e.to_string(),
                    })?;
            prod_extensions.remove(&id);
        }

        Ok(())
    }

    pub async fn remove_extension_internal(
        &self,
        app_handle: &AppHandle,
        public_key: &str,
        extension_name: &str,
        extension_version: &str,
        state: &State<'_, AppState>,
    ) -> Result<(), ExtensionError> {
        // Get the extension from memory to get its ID
        let extension = self
            .get_extension_by_public_key_and_name(public_key, extension_name)?
            .ok_or_else(|| ExtensionError::NotFound {
                public_key: public_key.to_string(),
                name: extension_name.to_string(),
            })?;

        eprintln!("DEBUG: Removing extension with ID: {}", extension.id);
        eprintln!(
            "DEBUG: Extension name: {}, version: {}",
            extension_name, extension_version
        );

        // Lösche Permissions und Extension-Eintrag in einer Transaktion
        with_connection(&state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
                reason: "Failed to lock HLC service".to_string(),
            })?;

            // Lösche alle Permissions mit extension_id
            eprintln!(
                "DEBUG: Deleting permissions for extension_id: {}",
                extension.id
            );
            PermissionManager::delete_permissions_in_transaction(&tx, &hlc_service, &extension.id)?;

            // Lösche Extension-Eintrag mit extension_id
            let sql = format!("DELETE FROM {} WHERE id = ?", TABLE_EXTENSIONS);
            eprintln!("DEBUG: Executing SQL: {} with id = {}", sql, extension.id);
            SqlExecutor::execute_internal_typed(
                &tx,
                &hlc_service,
                &sql,
                rusqlite::params![&extension.id],
            )?;

            eprintln!("DEBUG: Committing transaction");
            tx.commit().map_err(DatabaseError::from)
        })?;

        eprintln!("DEBUG: Transaction committed successfully");

        // Entferne aus dem In-Memory-Manager
        self.remove_extension(public_key, extension_name)?;

        // Lösche nur den spezifischen Versions-Ordner: public_key/name/version
        let extension_dir =
            self.get_extension_dir(app_handle, public_key, extension_name, extension_version)?;

        if extension_dir.exists() {
            std::fs::remove_dir_all(&extension_dir).map_err(|e| {
                ExtensionError::filesystem_with_path(extension_dir.display().to_string(), e)
            })?;

            // Versuche, leere Parent-Ordner zu löschen
            // 1. Extension-Name-Ordner (key_hash/name)
            if let Some(name_dir) = extension_dir.parent() {
                if name_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(name_dir) {
                        if entries.count() == 0 {
                            let _ = std::fs::remove_dir(name_dir);

                            // 2. Key-Hash-Ordner (key_hash) - nur wenn auch leer
                            if let Some(key_hash_dir) = name_dir.parent() {
                                if key_hash_dir.exists() {
                                    if let Ok(entries) = std::fs::read_dir(key_hash_dir) {
                                        if entries.count() == 0 {
                                            let _ = std::fs::remove_dir(key_hash_dir);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn preview_extension_internal(
        &self,
        app_handle: &AppHandle,
        file_bytes: Vec<u8>,
    ) -> Result<ExtensionPreview, ExtensionError> {
        let extracted = Self::extract_and_validate_extension(file_bytes, "haexhub_preview", app_handle)?;

        let is_valid_signature = ExtensionCrypto::verify_signature(
            &extracted.manifest.public_key,
            &extracted.content_hash,
            &extracted.manifest.signature,
        )
        .is_ok();

        let editable_permissions = extracted.manifest.to_editable_permissions();

        Ok(ExtensionPreview {
            manifest: extracted.manifest.clone(),
            is_valid_signature,
            editable_permissions,
        })
    }

    pub async fn install_extension_with_permissions_internal(
        &self,
        app_handle: AppHandle,
        file_bytes: Vec<u8>,
        custom_permissions: EditablePermissions,
        state: &State<'_, AppState>,
    ) -> Result<String, ExtensionError> {
        let extracted = Self::extract_and_validate_extension(file_bytes, "haexhub_ext", &app_handle)?;

        // Signatur verifizieren (bei Installation wird ein Fehler geworfen, nicht nur geprüft)
        ExtensionCrypto::verify_signature(
            &extracted.manifest.public_key,
            &extracted.content_hash,
            &extracted.manifest.signature,
        )
        .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        let extensions_dir = self.get_extension_dir(
            &app_handle,
            &extracted.manifest.public_key,
            &extracted.manifest.name,
            &extracted.manifest.version,
        )?;

        // If extension version already exists, remove it completely before installing
        if extensions_dir.exists() {
            eprintln!(
                "Extension version already exists at {}, removing old version",
                extensions_dir.display()
            );
            std::fs::remove_dir_all(&extensions_dir).map_err(|e| {
                ExtensionError::filesystem_with_path(extensions_dir.display().to_string(), e)
            })?;
        }

        std::fs::create_dir_all(&extensions_dir).map_err(|e| {
            ExtensionError::filesystem_with_path(extensions_dir.display().to_string(), e)
        })?;

        // Copy contents of extracted.temp_dir to extensions_dir
        // Note: extracted.temp_dir already points to the correct directory with manifest.json
        for entry in fs::read_dir(&extracted.temp_dir).map_err(|e| {
            ExtensionError::filesystem_with_path(extracted.temp_dir.display().to_string(), e)
        })? {
            let entry = entry.map_err(|e| ExtensionError::Filesystem { source: e })?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = extensions_dir.join(&file_name);

            if path.is_dir() {
                copy_directory(
                    path.to_string_lossy().to_string(),
                    dest_path.to_string_lossy().to_string(),
                )?;
            } else {
                fs::copy(&path, &dest_path).map_err(|e| {
                    ExtensionError::filesystem_with_path(path.display().to_string(), e)
                })?;
            }
        }

        // Generate UUID for extension (Drizzle's $defaultFn only works from JS, not raw SQL)
        let extension_id = uuid::Uuid::new_v4().to_string();
        let permissions = custom_permissions.to_internal_permissions(&extension_id);

        // Extension-Eintrag und Permissions in einer Transaktion speichern
        let actual_extension_id = with_connection(&state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service_guard = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
                reason: "Failed to lock HLC service".to_string(),
            })?;
            // Klonen, um den MutexGuard freizugeben, bevor potenziell lange DB-Operationen stattfinden
            let hlc_service = hlc_service_guard.clone();
            drop(hlc_service_guard);

            // 1. Extension-Eintrag erstellen mit generierter UUID
            let insert_ext_sql = format!(
                "INSERT INTO {} (id, name, version, author, entry, icon, public_key, signature, homepage, description, enabled, single_instance) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                TABLE_EXTENSIONS
            );

            SqlExecutor::execute_internal_typed(
                    &tx,
                    &hlc_service,
                    &insert_ext_sql,
                    rusqlite::params![
                        extension_id,
                        extracted.manifest.name,
                        extracted.manifest.version,
                        extracted.manifest.author,
                        extracted.manifest.entry,
                        extracted.manifest.icon,
                        extracted.manifest.public_key,
                        extracted.manifest.signature,
                        extracted.manifest.homepage,
                        extracted.manifest.description,
                        true, // enabled
                        extracted.manifest.single_instance.unwrap_or(false),
                    ],
                )?;

            // 2. Permissions speichern
            let insert_perm_sql = format!(
                "INSERT INTO {} (id, extension_id, resource_type, action, target, constraints, status) VALUES (?, ?, ?, ?, ?, ?, ?)",
                TABLE_EXTENSION_PERMISSIONS
            );

            for perm in &permissions {
                use crate::database::generated::HaexExtensionPermissions;
                let db_perm: HaexExtensionPermissions = perm.into();

                SqlExecutor::execute_internal_typed(
                    &tx,
                    &hlc_service,
                    &insert_perm_sql,
                    rusqlite::params![
                        db_perm.id,
                        db_perm.extension_id,
                        db_perm.resource_type,
                        db_perm.action,
                        db_perm.target,
                        db_perm.constraints,
                        db_perm.status,
                    ],
                )?;
            }

            tx.commit().map_err(DatabaseError::from)?;
            Ok(extension_id.clone())
        })?;

        let extension = Extension {
            id: extension_id.clone(),
            source: ExtensionSource::Production {
                path: extensions_dir.clone(),
                version: extracted.manifest.version.clone(),
            },
            manifest: extracted.manifest.clone(),
            enabled: true,
            last_accessed: SystemTime::now(),
        };

        self.add_production_extension(extension)?;

        Ok(actual_extension_id) // Gebe die actual_extension_id an den Caller zurück
    }

    /// Scannt das Dateisystem beim Start und lädt alle installierten Erweiterungen.
    pub async fn load_installed_extensions(
        &self,
        app_handle: &AppHandle,
        state: &State<'_, AppState>,
    ) -> Result<Vec<String>, ExtensionError> {
        // Clear existing data
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

        // Lade alle Daten aus der Datenbank
        let extensions = with_connection(&state.db, |conn| {
            let sql = format!(
            "SELECT id, name, version, author, entry, icon, public_key, signature, homepage, description, enabled, single_instance FROM {}",
            TABLE_EXTENSIONS
        );
            eprintln!("DEBUG: SQL Query before transformation: {}", sql);

            let results = SqlExecutor::query_select(conn, &sql, &[])?;
            eprintln!("DEBUG: Query returned {} results", results.len());

            let mut data = Vec::new();
            for row in results {
                // Wir erwarten die Werte in der Reihenfolge der SELECT-Anweisung
                let id = row[0]
                    .as_str()
                    .ok_or_else(|| DatabaseError::SerializationError {
                        reason: "Missing id field".to_string(),
                    })?
                    .to_string();

                let manifest = ExtensionManifest {
                    name: row[1]
                        .as_str()
                        .ok_or_else(|| DatabaseError::SerializationError {
                            reason: "Missing name field".to_string(),
                        })?
                        .to_string(),
                    version: row[2]
                        .as_str()
                        .ok_or_else(|| DatabaseError::SerializationError {
                            reason: "Missing version field".to_string(),
                        })?
                        .to_string(),
                    author: row[3].as_str().map(String::from),
                    entry: row[4].as_str().map(String::from),
                    icon: row[5].as_str().map(String::from),
                    public_key: row[6].as_str().unwrap_or("").to_string(),
                    signature: row[7].as_str().unwrap_or("").to_string(),
                    permissions: ExtensionPermissions::default(),
                    homepage: row[8].as_str().map(String::from),
                    description: row[9].as_str().map(String::from),
                    single_instance: row[11]
                        .as_bool()
                        .or_else(|| row[11].as_i64().map(|v| v != 0)),
                };

                let enabled = row[10]
                    .as_bool()
                    .or_else(|| row[10].as_i64().map(|v| v != 0))
                    .unwrap_or(false);

                data.push(ExtensionDataFromDb {
                    id,
                    manifest,
                    enabled,
                });
            }
            Ok(data)
        })?;

        // Schritt 2: Die gesammelten Daten verarbeiten (Dateisystem, State-Mutationen).
        let mut loaded_extension_ids = Vec::new();

        eprintln!("DEBUG: Found {} extensions in database", extensions.len());

        for extension_data in extensions {
            let extension_id = extension_data.id;
            eprintln!("DEBUG: Processing extension: {}", extension_id);

            // Use public_key/name/version path structure
            let extension_path = self.get_extension_dir(
                app_handle,
                &extension_data.manifest.public_key,
                &extension_data.manifest.name,
                &extension_data.manifest.version,
            )?;

            // Check if extension directory exists
            if !extension_path.exists() {
                eprintln!(
                    "DEBUG: Extension directory missing for: {} at {:?}",
                    extension_id, extension_path
                );
                self.missing_extensions
                    .lock()
                    .map_err(|e| ExtensionError::MutexPoisoned {
                        reason: e.to_string(),
                    })?
                    .push(MissingExtension {
                        id: extension_id.clone(),
                        public_key: extension_data.manifest.public_key.clone(),
                        name: extension_data.manifest.name.clone(),
                        version: extension_data.manifest.version.clone(),
                    });
                continue;
            }

            // Read haextension_dir from config if it exists, otherwise use default
            let config_path = extension_path.join("haextension.config.json");
            let haextension_dir = if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(config_content) => {
                        match serde_json::from_str::<serde_json::Value>(&config_content) {
                            Ok(config) => {
                                config
                                    .get("dev")
                                    .and_then(|dev| dev.get("haextension_dir"))
                                    .and_then(|dir| dir.as_str())
                                    .unwrap_or("haextension")
                                    .to_string()
                            }
                            Err(_) => "haextension".to_string(),
                        }
                    }
                    Err(_) => "haextension".to_string(),
                }
            } else {
                "haextension".to_string()
            };

            // Validate manifest.json path using helper function
            let manifest_relative_path = format!("{}/manifest.json", haextension_dir);
            if Self::validate_path_in_directory(&extension_path, &manifest_relative_path, true)?
                .is_none()
            {
                eprintln!(
                    "DEBUG: manifest.json missing or invalid for: {} at {}/manifest.json",
                    extension_id, haextension_dir
                );
                self.missing_extensions
                    .lock()
                    .map_err(|e| ExtensionError::MutexPoisoned {
                        reason: e.to_string(),
                    })?
                    .push(MissingExtension {
                        id: extension_id.clone(),
                        public_key: extension_data.manifest.public_key.clone(),
                        name: extension_data.manifest.name.clone(),
                        version: extension_data.manifest.version.clone(),
                    });
                continue;
            }

            eprintln!("DEBUG: Extension loaded successfully: {}", extension_id);

            let extension = Extension {
                id: extension_id.clone(),
                source: ExtensionSource::Production {
                    path: extension_path,
                    version: extension_data.manifest.version.clone(),
                },
                manifest: extension_data.manifest,
                enabled: extension_data.enabled,
                last_accessed: SystemTime::now(),
            };

            loaded_extension_ids.push(extension_id.clone());
            self.add_production_extension(extension)?;
        }

        Ok(loaded_extension_ids)
    }
}
