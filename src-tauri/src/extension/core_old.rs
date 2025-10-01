/// src-tauri/src/extension/core.rs
use crate::extension::crypto::ExtensionCrypto;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::{
    Action, DbConstraints, ExtensionPermission, FsConstraints, HttpConstraints,
    PermissionConstraints, PermissionStatus, ResourceType, ShellConstraints,
};
use crate::AppState;
use mime;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::State;
use tauri::{
    http::{Request, Response},
    AppHandle, Manager, Runtime, UriSchemeContext,
};
use zip::ZipArchive;

#[derive(Serialize, Deserialize)]
pub struct ExtensionPreview {
    pub manifest: ExtensionManifest,
    pub is_valid_signature: bool,
    pub key_hash: String,
    pub editable_permissions: EditablePermissions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditablePermissions {
    pub permissions: Vec<EditablePermission>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditablePermission {
    pub resource_type: String,
    pub action: String,
    pub target: String,
    pub constraints: Option<serde_json::Value>,
    pub status: String,
}

impl EditablePermissions {
    /// Konvertiert EditablePermissions zu internen ExtensionPermissions
    pub fn to_internal_permissions(&self, extension_id: &str) -> Vec<ExtensionPermission> {
        self.permissions
            .iter()
            .map(|p| ExtensionPermission {
                id: uuid::Uuid::new_v4().to_string(),
                extension_id: extension_id.to_string(),
                resource_type: match p.resource_type.as_str() {
                    "fs" => ResourceType::Fs,
                    "http" => ResourceType::Http,
                    "db" => ResourceType::Db,
                    "shell" => ResourceType::Shell,
                    _ => ResourceType::Fs, // Fallback
                },
                action: match p.action.as_str() {
                    "read" => Action::Read,
                    "write" => Action::Write,
                    _ => Action::Read, // Fallback
                },
                target: p.target.clone(),
                constraints: p
                    .constraints
                    .as_ref()
                    .and_then(|c| Self::parse_constraints(&p.resource_type, c)),
                status: match p.status.as_str() {
                    "granted" => PermissionStatus::Granted,
                    "denied" => PermissionStatus::Denied,
                    "ask" => PermissionStatus::Ask,
                    _ => PermissionStatus::Denied, // Fallback
                },
                haex_timestamp: None,
                haex_tombstone: None,
            })
            .collect()
    }

    fn parse_constraints(
        resource_type: &str,
        json_value: &serde_json::Value,
    ) -> Option<PermissionConstraints> {
        match resource_type {
            "db" => serde_json::from_value::<DbConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Database),
            "fs" => serde_json::from_value::<FsConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Filesystem),
            "http" => serde_json::from_value::<HttpConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Http),
            "shell" => serde_json::from_value::<ShellConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Shell),
            _ => None,
        }
    }

    /// Filtert nur granted Permissions
    pub fn filter_granted(&self) -> Vec<EditablePermission> {
        self.permissions
            .iter()
            .filter(|p| p.status == "granted")
            .cloned()
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub entry: String,
    pub icon: Option<String>,
    pub public_key: String,
    pub signature: String,
    pub permissions: ExtensionManifestPermissions,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

impl ExtensionManifest {
    /// Berechnet den Key Hash für diese Extension
    pub fn calculate_key_hash(&self) -> Result<String, ExtensionError> {
        ExtensionCrypto::calculate_key_hash(&self.public_key)
            .map_err(|e| ExtensionError::InvalidPublicKey { reason: e })
    }

    /// Generiert die vollständige Extension ID mit Key Hash Prefix
    pub fn full_extension_id(&self) -> Result<String, ExtensionError> {
        let key_hash = self.calculate_key_hash()?;
        Ok(format!("{}-{}", key_hash, self.id))
    }
    pub fn to_editable_permissions(&self) -> EditablePermissions {
        let mut database = Vec::new();
        let mut filesystem = Vec::new();
        let mut http = Vec::new();

        if let Some(db) = &self.permissions.database {
            for resource in &db.read {
                database.push(EditableDatabasePermission {
                    operation: "read".to_string(),
                    resource: resource.clone(),
                    status: PermissionStatus::Granted,
                });
            }
            for resource in &db.write {
                database.push(EditableDatabasePermission {
                    operation: "write".to_string(),
                    resource: resource.clone(),
                    status: PermissionStatus::Granted,
                });
            }
        }

        if let Some(fs) = &self.permissions.filesystem {
            for path in &fs.read {
                filesystem.push(EditableFilesystemPermission {
                    operation: "read".to_string(),
                    path: path.clone(),
                    status: PermissionStatus::Granted,
                });
            }
            for path in &fs.write {
                filesystem.push(EditableFilesystemPermission {
                    operation: "write".to_string(),
                    path: path.clone(),
                    status: PermissionStatus::Granted,
                });
            }
        }

        if let Some(http_list) = &self.permissions.http {
            for domain in http_list {
                http.push(EditableHttpPermission {
                    domain: domain.clone(),
                    status: PermissionStatus::Granted,
                });
            }
        }

        EditablePermissions {
            database,
            filesystem,
            http,
        }
    }
}

impl ExtensionManifest {
    /// Konvertiert Manifest zu EditablePermissions (neue Version)
    pub fn to_editable_permissions(&self) -> EditablePermissions {
        let mut permissions = Vec::new();

        // Database Permissions
        if let Some(db) = &self.permissions.database {
            for resource in &db.read {
                permissions.push(EditablePermission {
                    resource_type: "db".to_string(),
                    action: "read".to_string(),
                    target: resource.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
            for resource in &db.write {
                permissions.push(EditablePermission {
                    resource_type: "db".to_string(),
                    action: "write".to_string(),
                    target: resource.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        // Filesystem Permissions
        if let Some(fs) = &self.permissions.filesystem {
            for path in &fs.read {
                permissions.push(EditablePermission {
                    resource_type: "fs".to_string(),
                    action: "read".to_string(),
                    target: path.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
            for path in &fs.write {
                permissions.push(EditablePermission {
                    resource_type: "fs".to_string(),
                    action: "write".to_string(),
                    target: path.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        // HTTP Permissions
        if let Some(http_list) = &self.permissions.http {
            for domain in http_list {
                permissions.push(EditablePermission {
                    resource_type: "http".to_string(),
                    action: "read".to_string(), // HTTP ist meist read
                    target: domain.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        // Shell Permissions
        if let Some(shell_list) = &self.permissions.shell {
            for command in shell_list {
                permissions.push(EditablePermission {
                    resource_type: "shell".to_string(),
                    action: "read".to_string(), // Shell hat keine action mehr im Schema
                    target: command.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        EditablePermissions { permissions }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionInfoResponse {
    pub key_hash: String,
    pub name: String,
    pub full_id: String,
    pub version: String,
    pub display_name: Option<String>,
    pub namespace: Option<String>,
    pub allowed_origin: String,
}

impl ExtensionInfoResponse {
    pub fn from_extension(extension: &Extension) -> Result<Self, ExtensionError> {
        // Bestimme die allowed_origin basierend auf Tauri-Konfiguration
        let allowed_origin = get_tauri_origin();
        let key_hash = extension
            .manifest
            .calculate_key_hash()
            .map_err(|e| ExtensionError::InvalidPublicKey { reason: e })?;
        let full_id = extension
            .manifest
            .full_extension_id()
            .map_err(|e| ExtensionError::InvalidPublicKey { reason: e })?;

        Ok(Self {
            key_hash,
            name: extension.manifest.name.clone(),
            full_id,
            version: extension.manifest.version.clone(),
            display_name: Some(extension.manifest.name.clone()),
            namespace: extension.manifest.author.clone(),
            allowed_origin,
        })
    }
}

fn get_tauri_origin() -> String {
    #[cfg(target_os = "windows")]
    {
        "https://tauri.localhost".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "android")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "ios")]
    {
        "tauri://localhost".to_string()
    }
}

/// Extension source type (production vs development)
#[derive(Debug, Clone)]
pub enum ExtensionSource {
    Production {
        path: PathBuf,
        version: String,
    },
    Development {
        dev_server_url: String,
        manifest_path: PathBuf,
        auto_reload: bool,
    },
}

/// Complete extension data structure
#[derive(Debug, Clone)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub source: ExtensionSource,
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub last_accessed: SystemTime,
}

/// Cached permission data for performance
#[derive(Debug, Clone)]
pub struct CachedPermission {
    pub permissions: Vec<DbExtensionPermission>,
    pub cached_at: SystemTime,
    pub ttl: Duration,
}

/// Enhanced extension manager
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

    pub fn get_base_extension_dir(&self, app_handle: AppHandle) -> Result<PathBuf, ExtensionError> {
        let path = app_handle
            .path()
            .app_local_data_dir() // Korrekt für Ressourcen
            // Wenn du stattdessen App Local Data willst: .app_local_data_dir()
            .map_err(|e| ExtensionError::Filesystem {
                source: std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()),
            })?
            .join("extensions");
        Ok(path)
    }

    pub fn get_extension_dir(
        &self,
        app_handle: AppHandle,
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

        // Validate filesystem permissions
        /* if let Some(fs_perms) = &extension.manifest.permissions.filesystem {
                   fs_perms.validate()?;
               }
        */
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

        // Validate filesystem permissions
        /* if let Some(fs_perms) = &extension.manifest.permissions.filesystem {
            fs_perms.validate()?;
        } */

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
        // Dev extensions take priority
        let dev_extensions = self.dev_extensions.lock().unwrap();
        if let Some(extension) = dev_extensions.get(extension_id) {
            return Some(extension.clone());
        }

        // Then check production
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
        app_handle: AppHandle,
        extension_id: String,
        extension_version: String,
        state: &State<'_, AppState>,
    ) -> Result<(), ExtensionError> {
        // Permissions löschen (verwendet jetzt die neue Methode)
        PermissionManager::delete_permissions(state, &extension_id).await?;

        // Extension aus Manager entfernen
        self.remove_extension(&extension_id)?;

        let extension_dir =
            self.get_extension_dir(app_handle, &extension_id, &extension_version)?;

        // Dateien löschen
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

        // ZIP in temp entpacken
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

        // Manifest laden
        let manifest_path = temp.join("manifest.json");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        // Signatur verifizieren
        let content_hash = ExtensionCrypto::hash_directory(&temp)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        let is_valid_signature = ExtensionCrypto::verify_signature(
            &manifest.public_key,
            &content_hash,
            &manifest.signature,
        )
        .is_ok();

        let key_hash = manifest.calculate_key_hash()?;

        // Editable permissions erstellen
        let editable_permissions = manifest.to_editable_permissions();

        // Cleanup
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

        // 1. ZIP entpacken
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

        // 2. Manifest laden
        let manifest_path = temp.join("manifest.json");
        let manifest_content =
            std::fs::read_to_string(&manifest_path).map_err(|e| ExtensionError::ManifestError {
                reason: format!("Cannot read manifest: {}", e),
            })?;

        let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

        // 3. Signatur verifizieren
        let content_hash = ExtensionCrypto::hash_directory(&temp)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        ExtensionCrypto::verify_signature(&manifest.public_key, &content_hash, &manifest.signature)
            .map_err(|e| ExtensionError::SignatureVerificationFailed { reason: e })?;

        // 4. Key Hash berechnen
        let key_hash = manifest.calculate_key_hash()?;
        let full_extension_id = format!("{}-{}", key_hash, manifest.id);

        // 5. Zielverzeichnis
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

        // 6. Dateien kopieren
        copy_directory(
            temp.to_string_lossy().to_string(),
            extensions_dir.to_string_lossy().to_string(),
        )?;

        // 7. Temp aufräumen
        std::fs::remove_dir_all(&temp).ok();

        // 8. Custom Permissions konvertieren und speichern
        let permissions = custom_permissions.to_internal_permissions(&full_extension_id);
        let granted_permissions = permissions.filter_granted();
        PermissionManager::save_permissions(&state.db, &granted_permissions).await?;

        // 9. Extension registrieren
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

        state
            .extension_manager
            .add_production_extension(extension)?;

        Ok(full_extension_id)
    }
}

// For backward compatibility
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

#[derive(Deserialize, Debug)]
struct ExtensionInfo {
    id: String,
    version: String,
}

#[derive(Debug)]
enum DataProcessingError {
    HexDecoding(hex::FromHexError),
    Utf8Conversion(std::string::FromUtf8Error),
    JsonParsing(serde_json::Error),
}

// Implementierung von Display für benutzerfreundliche Fehlermeldungen
impl fmt::Display for DataProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataProcessingError::HexDecoding(e) => write!(f, "Hex-Dekodierungsfehler: {}", e),
            DataProcessingError::Utf8Conversion(e) => {
                write!(f, "UTF-8-Konvertierungsfehler: {}", e)
            }
            DataProcessingError::JsonParsing(e) => write!(f, "JSON-Parsing-Fehler: {}", e),
        }
    }
}

// Implementierung von std::error::Error (optional, aber gute Praxis für bibliotheksähnlichen Code)
impl std::error::Error for DataProcessingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataProcessingError::HexDecoding(e) => Some(e),
            DataProcessingError::Utf8Conversion(e) => Some(e),
            DataProcessingError::JsonParsing(e) => Some(e),
        }
    }
}

// Implementierung von From-Traits für einfache Verwendung des '?'-Operators
impl From<hex::FromHexError> for DataProcessingError {
    fn from(err: hex::FromHexError) -> Self {
        DataProcessingError::HexDecoding(err)
    }
}

impl From<std::string::FromUtf8Error> for DataProcessingError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        DataProcessingError::Utf8Conversion(err)
    }
}

impl From<serde_json::Error> for DataProcessingError {
    fn from(err: serde_json::Error) -> Self {
        DataProcessingError::JsonParsing(err)
    }
}

pub fn copy_directory(source: String, destination: String) -> Result<(), ExtensionError> {
    println!(
        "Kopiere Verzeichnis von '{}' nach '{}'",
        source, destination
    );

    let source_path = PathBuf::from(&source);
    let destination_path = PathBuf::from(&destination);

    if !source_path.exists() || !source_path.is_dir() {
        return Err(ExtensionError::Filesystem {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory '{}' not found", source),
            ),
        });
    }

    // Optionen für fs_extra::dir::copy
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true; // Überschreibe Zieldateien, falls sie existieren
    options.copy_inside = true; // Kopiere den *Inhalt* des Quellordners in den Zielordner
                                // options.content_only = true; // Alternative: nur Inhalt kopieren, Zielordner muss existieren
    options.buffer_size = 64000; // Standard-Puffergröße, kann angepasst werden

    // Führe die Kopieroperation aus
    fs_extra::dir::copy(&source_path, &destination_path, &options).map_err(|e| {
        ExtensionError::Filesystem {
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        }
    })?;
    Ok(())
}

pub fn resolve_secure_extension_asset_path(
    app_handle: AppHandle,
    state: State<AppState>,
    extension_id: &str,
    extension_version: &str,
    requested_asset_path: &str,
) -> Result<PathBuf, ExtensionError> {
    // 1. Validiere die Extension ID
    if extension_id.is_empty()
        || !extension_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(ExtensionError::ValidationError {
            reason: format!("Invalid extension ID: {}", extension_id),
        });
    }

    if extension_version.is_empty()
        || !extension_version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(ExtensionError::ValidationError {
            reason: format!("Invalid extension version: {}", extension_version),
        });
    }

    // 3. Verzeichnis für die spezifische Erweiterung
    let specific_extension_dir =
        state
            .extension_manager
            .get_extension_dir(app_handle, extension_id, extension_version)?;

    // 4. Bereinige den angeforderten Asset-Pfad
    let clean_relative_path = requested_asset_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .split('/')
        .filter(|&part| !part.is_empty() && part != "." && part != "..")
        .collect::<PathBuf>();

    if clean_relative_path.as_os_str().is_empty() && requested_asset_path != "/" {
        return Err(ExtensionError::ValidationError {
            reason: "Empty or invalid asset path".to_string(),
        });
    }

    // 5. Setze den finalen Pfad zusammen
    let final_path = specific_extension_dir.join(clean_relative_path);

    // 6. SICHERHEITSCHECK
    match final_path.canonicalize() {
        Ok(canonical_path) => {
            let canonical_base = specific_extension_dir
                .canonicalize()
                .map_err(|e| ExtensionError::Filesystem { source: e })?;
            if canonical_path.starts_with(&canonical_base) {
                Ok(canonical_path)
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Path traversal attempt: {}", requested_asset_path),
                })
            }
        }
        Err(_) => {
            // Fehler bei canonicalize (z.B. Pfad existiert nicht)
            if final_path.starts_with(&specific_extension_dir) {
                Ok(final_path) // Nicht-kanonisierten Pfad zurückgeben
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Invalid asset path: {}", requested_asset_path),
                })
            }
        }
    }
}

pub fn extension_protocol_handler<R: Runtime>(
    state: State<AppState>,
    app_handle: AppHandle,
    request: &Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let uri_ref = request.uri();
    println!("Protokoll Handler für: {}", uri_ref);

    let host = uri_ref
        .host()
        .ok_or("Kein Host (Extension ID) in URI gefunden")?
        .to_string();

    let path_str = uri_ref.path();
    let segments_iter = path_str.split('/').filter(|s| !s.is_empty());
    let resource_segments: Vec<&str> = segments_iter.collect();
    let raw_asset_path = resource_segments.join("/");
    let asset_to_load = if raw_asset_path.is_empty() {
        "index.html"
    } else {
        &raw_asset_path
    };

    match process_hex_encoded_json(&host) {
        Ok(info) => {
            println!("Daten erfolgreich verarbeitet:");
            println!("  ID: {}", info.id);
            println!("  Version: {}", info.version);
            let absolute_secure_path = resolve_secure_extension_asset_path(
                app_handle,
                state,
                &info.id,
                &info.version,
                &asset_to_load,
            )?;

            println!("absolute_secure_path: {}", absolute_secure_path.display());

            if absolute_secure_path.exists() && absolute_secure_path.is_file() {
                match fs::read(&absolute_secure_path) {
                    Ok(content) => {
                        let mime_type = mime_guess::from_path(&absolute_secure_path)
                            .first_or(mime::APPLICATION_OCTET_STREAM)
                            .to_string();
                        let content_length = content.len();
                        println!(
                            "Liefere {} ({}, {} bytes) ", // Content-Length zum Log hinzugefügt
                            absolute_secure_path.display(),
                            mime_type,
                            content_length
                        );
                        Response::builder()
                            .status(200)
                            .header("Content-Type", mime_type)
                            .header("Content-Length", content_length.to_string()) // <-- HIER HINZUGEFÜGT
                            // Optional, aber gut für Streaming-Fähigkeit:
                            .header("Accept-Ranges", "bytes")
                            .body(content)
                            .map_err(|e| e.into())
                    }
                    Err(e) => {
                        eprintln!(
                            "Fehler beim Lesen der Datei {}: {}",
                            absolute_secure_path.display(),
                            e
                        );
                        let status_code = if e.kind() == std::io::ErrorKind::NotFound {
                            404
                        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                            403
                        } else {
                            500
                        };

                        Response::builder()
                            .status(status_code)
                            .body(Vec::new()) // Leerer Body für Fehler
                            .map_err(|e| e.into()) // Wandle http::Error in Box<dyn Error> um
                    }
                }
            } else {
                // Datei nicht gefunden oder es ist keine Datei
                eprintln!(
                    "Asset nicht gefunden oder ist kein File: {}",
                    absolute_secure_path.display()
                );
                Response::builder()
                    .status(404) // HTTP 404 Not Found
                    .body(Vec::new())
                    .map_err(|e| e.into())
            }
        }
        Err(e) => {
            eprintln!("Fehler bei der Datenverarbeitung: {}", e);

            Response::builder()
                .status(500)
                .body(Vec::new()) // Leerer Body für Fehler
                .map_err(|e| e.into())
        }
    }
}

fn process_hex_encoded_json(hex_input: &str) -> Result<ExtensionInfo, DataProcessingError> {
    // Schritt 1: Hex-String zu Bytes dekodieren
    let bytes = hex::decode(hex_input)?; // Konvertiert hex::FromHexError automatisch

    // Schritt 2: Bytes zu UTF-8-String konvertieren
    let json_string = String::from_utf8(bytes)?; // Konvertiert FromUtf8Error automatisch

    // Schritt 3: JSON-String zu Struktur parsen
    let extension_info: ExtensionInfo = serde_json::from_str(&json_string)?; // Konvertiert serde_json::Error automatisch

    Ok(extension_info)
}
