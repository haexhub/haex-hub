use crate::database::core::with_connection;
use crate::database::error::DatabaseError;
use crate::extension::core::manifest::{EditablePermissions, ExtensionManifest, ExtensionPreview};
use crate::extension::core::types::{copy_directory, Extension, ExtensionSource};
use crate::extension::core::ExtensionPermissions;
use crate::extension::crypto::ExtensionCrypto;
use crate::extension::database::executor::{PkRemappingContext, SqlExecutor};
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::ExtensionPermission;
use crate::table_names::{TABLE_EXTENSIONS, TABLE_EXTENSION_PERMISSIONS};
use crate::AppState;
use std::collections::HashMap;
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

    /// Extrahiert eine Extension-ZIP-Datei und validiert das Manifest
    fn extract_and_validate_extension(
        bytes: Vec<u8>,
        temp_prefix: &str,
    ) -> Result<ExtractedExtension, ExtensionError> {
        let temp = std::env::temp_dir().join(format!("{}_{}", temp_prefix, uuid::Uuid::new_v4()));

        fs::create_dir_all(&temp)
            .map_err(|e| ExtensionError::filesystem_with_path(temp.display().to_string(), e))?;

        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
            ExtensionError::InstallationFailed {
                reason: format!("Invalid ZIP: {}", e),
            }
        })?;

        archive
            .extract(&temp)
            .map_err(|e| ExtensionError::InstallationFailed {
                reason: format!("Cannot extract ZIP: {}", e),
            })?;

        // Check if manifest.json is directly in temp or in a subdirectory
        let manifest_path = temp.join("manifest.json");
        let actual_dir = if manifest_path.exists() {
            temp.clone()
        } else {
            // manifest.json is in a subdirectory - find it
            let mut found_dir = None;
            for entry in fs::read_dir(&temp)
                .map_err(|e| ExtensionError::filesystem_with_path(temp.display().to_string(), e))?
            {
                let entry = entry.map_err(|e| ExtensionError::Filesystem { source: e })?;
                let path = entry.path();
                if path.is_dir() && path.join("manifest.json").exists() {
                    found_dir = Some(path);
                    break;
                }
            }

            found_dir.ok_or_else(|| ExtensionError::ManifestError {
                reason: "manifest.json not found in extension archive".to_string(),
            })?
        };

        let manifest_path = actual_dir.join("manifest.json");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        let content_hash = ExtensionCrypto::hash_directory(&actual_dir).map_err(|e| {
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
        let dev_extensions = self.dev_extensions.lock().map_err(|e| {
            ExtensionError::MutexPoisoned {
                reason: e.to_string(),
            }
        })?;

        for (id, ext) in dev_extensions.iter() {
            if ext.manifest.public_key == public_key && ext.manifest.name == name {
                return Ok(Some((id.clone(), ext.clone())));
            }
        }

        // 2. Check production extensions
        let prod_extensions = self.production_extensions.lock().map_err(|e| {
            ExtensionError::MutexPoisoned {
                reason: e.to_string(),
            }
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

    pub fn remove_extension(
        &self,
        public_key: &str,
        name: &str,
    ) -> Result<(), ExtensionError> {
        let (id, _) = self
            .find_extension_id_by_public_key_and_name(public_key, name)?
            .ok_or_else(|| ExtensionError::NotFound {
                public_key: public_key.to_string(),
                name: name.to_string(),
            })?;

        // Remove from dev extensions first
        {
            let mut dev_extensions = self.dev_extensions.lock().map_err(|e| {
                ExtensionError::MutexPoisoned {
                    reason: e.to_string(),
                }
            })?;
            if dev_extensions.remove(&id).is_some() {
                return Ok(());
            }
        }

        // Remove from production extensions
        {
            let mut prod_extensions = self.production_extensions.lock().map_err(|e| {
                ExtensionError::MutexPoisoned {
                    reason: e.to_string(),
                }
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
        eprintln!("DEBUG: Extension name: {}, version: {}", extension_name, extension_version);

        // Lösche Permissions und Extension-Eintrag in einer Transaktion
        with_connection(&state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
                reason: "Failed to lock HLC service".to_string(),
            })?;

            // Lösche alle Permissions mit extension_id
            eprintln!("DEBUG: Deleting permissions for extension_id: {}", extension.id);
            PermissionManager::delete_permissions_in_transaction(
                &tx,
                &hlc_service,
                &extension.id,
            )?;

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
        file_bytes: Vec<u8>,
    ) -> Result<ExtensionPreview, ExtensionError> {
        let extracted = Self::extract_and_validate_extension(file_bytes, "haexhub_preview")?;

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
        let extracted = Self::extract_and_validate_extension(file_bytes, "haexhub_ext")?;

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

            let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
                reason: "Failed to lock HLC service".to_string(),
            })?;

            // Erstelle PK-Remapping Context für die gesamte Transaktion
            // Dies ermöglicht automatisches FK-Remapping wenn ON CONFLICT bei Extension auftritt
            let mut pk_context = PkRemappingContext::new();

            // 1. Extension-Eintrag erstellen mit generierter UUID
            // WICHTIG: RETURNING wird vom CRDT-Transformer automatisch hinzugefügt
            let insert_ext_sql = format!(
                "INSERT INTO {} (id, name, version, author, entry, icon, public_key, signature, homepage, description, enabled) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
                TABLE_EXTENSIONS
            );

            let (_tables, returning_results) = SqlExecutor::query_internal_typed_with_context(
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
                ],
                &mut pk_context,
            )?;

            // Nutze die tatsächliche ID aus der Datenbank (wichtig bei ON CONFLICT)
            // Die haex_extensions Tabelle hat einen single-column PK namens "id"
            let actual_extension_id = returning_results
                .first()
                .and_then(|row| row.first())
                .and_then(|val| val.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| extension_id.clone());

            eprintln!(
                "DEBUG: Extension UUID - Generated: {}, Actual from DB: {}",
                extension_id, actual_extension_id
            );

            // 2. Permissions speichern (oder aktualisieren falls schon vorhanden)
            // Nutze einfaches INSERT - die CRDT-Transformation fügt automatisch ON CONFLICT hinzu
            // FK-Werte (extension_id) werden automatisch remapped wenn Extension ON CONFLICT hatte
            let insert_perm_sql = format!(
                "INSERT INTO {} (id, extension_id, resource_type, action, target, constraints, status) VALUES (?, ?, ?, ?, ?, ?, ?)",
                TABLE_EXTENSION_PERMISSIONS
            );

            for perm in &permissions {
                use crate::database::generated::HaexExtensionPermissions;
                let db_perm: HaexExtensionPermissions = perm.into();

                SqlExecutor::execute_internal_typed_with_context(
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
                    &mut pk_context,
                )?;
            }

            tx.commit().map_err(DatabaseError::from)?;
            Ok(actual_extension_id.clone())
        })?;

        let extension = Extension {
            id: actual_extension_id.clone(), // Nutze die actual_extension_id aus der Transaktion
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
            let sql = format!(
                "SELECT id, name, version, author, entry, icon, public_key, signature, homepage, description, enabled FROM {}",
                TABLE_EXTENSIONS
            );
            eprintln!("DEBUG: SQL Query before transformation: {}", sql);
            let results = SqlExecutor::select_internal(conn, &sql, &[])?;
            eprintln!("DEBUG: Query returned {} results", results.len());

            let mut data = Vec::new();
            for result in results {
                let id = result["id"]
                    .as_str()
                    .ok_or_else(|| DatabaseError::SerializationError {
                        reason: "Missing id field".to_string(),
                    })?
                    .to_string();

                let manifest = ExtensionManifest {
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

            if !extension_path.exists() || !extension_path.join("manifest.json").exists() {
                eprintln!(
                    "DEBUG: Extension files missing for: {} at {:?}",
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

            eprintln!(
                "DEBUG: Extension loaded successfully: {}",
                extension_id
            );

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
