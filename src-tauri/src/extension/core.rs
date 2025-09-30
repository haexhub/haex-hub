/// src-tauri/src/extension/core.rs
use crate::extension::database::permissions::DbExtensionPermission;
use crate::extension::error::ExtensionError;
use crate::extension::permission_manager::ExtensionPermissions;
use mime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::{
    http::{Request, Response},
    AppHandle, Error as TauriError, Manager, Runtime, UriSchemeContext,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub fn from_extension(extension: &Extension) -> Self {
        // Bestimme die allowed_origin basierend auf Tauri-Konfiguration
        let allowed_origin = get_tauri_origin();

        Self {
            key_hash: calculate_key_hash(&extension.manifest.id),
            name: extension.manifest.name.clone(),
            full_id: format!(
                "{}/{}@{}",
                calculate_key_hash(&extension.manifest.id),
                extension.manifest.name,
                extension.manifest.version
            ),
            version: extension.manifest.version.clone(),
            display_name: Some(extension.manifest.name.clone()),
            namespace: extension.manifest.author.clone(),
            allowed_origin,
        }
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

// Dummy-Funktion für Key Hash (du implementierst das richtig mit SHA-256)
fn calculate_key_hash(id: &str) -> String {
    // TODO: Implementiere SHA-256 Hash vom Public Key
    // Für jetzt nur Placeholder
    format!("{:0<20}", id.chars().take(20).collect::<String>())
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

pub fn copy_directory(source: String, destination: String) -> Result<(), String> {
    println!(
        "Kopiere Verzeichnis von '{}' nach '{}'",
        source, destination
    );

    let source_path = PathBuf::from(&source);
    let destination_path = PathBuf::from(&destination);

    if !source_path.exists() || !source_path.is_dir() {
        return Err(format!(
            "Quellverzeichnis '{}' nicht gefunden oder ist kein Verzeichnis.",
            source
        ));
    }

    // Optionen für fs_extra::dir::copy
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true; // Überschreibe Zieldateien, falls sie existieren
    options.copy_inside = true; // Kopiere den *Inhalt* des Quellordners in den Zielordner
                                // options.content_only = true; // Alternative: nur Inhalt kopieren, Zielordner muss existieren
    options.buffer_size = 64000; // Standard-Puffergröße, kann angepasst werden

    // Führe die Kopieroperation aus
    match fs_extra::dir::copy(&source_path, &destination_path, &options) {
        Ok(bytes_copied) => {
            println!("Verzeichnis erfolgreich kopiert ({} bytes)", bytes_copied);
            Ok(()) // Erfolg signalisieren
        }
        Err(e) => {
            eprintln!("Fehler beim Kopieren des Verzeichnisses: {}", e);
            Err(format!("Fehler beim Kopieren: {}", e.to_string())) // Fehler als String zurückgeben
        }
    }
}

pub fn resolve_secure_extension_asset_path<R: Runtime>(
    app_handle: &AppHandle<R>,
    extension_id: &str,
    extension_version: &str,
    requested_asset_path: &str,
) -> Result<PathBuf, String> {
    // 1. Validiere die Extension ID
    if extension_id.is_empty()
        || !extension_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(format!("Ungültige Extension ID: {}", extension_id));
    }

    if extension_version.is_empty()
        || !extension_version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(format!(
            "Ungültige Extension Version: {}",
            extension_version
        ));
    }

    // 2. Bestimme das Basisverzeichnis für alle Erweiterungen (Resource Directory)
    let base_extensions_dir = app_handle
        .path()
        .app_data_dir() // Korrekt für Ressourcen
        // Wenn du stattdessen App Local Data willst: .app_local_data_dir()
        .map_err(|e: TauriError| format!("Basis-Verzeichnis nicht gefunden: {}", e))?
        .join("extensions");

    // 3. Verzeichnis für die spezifische Erweiterung
    let specific_extension_dir =
        base_extensions_dir.join(format!("{}/{}", extension_id, extension_version));

    // 4. Bereinige den angeforderten Asset-Pfad
    let clean_relative_path = requested_asset_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .split('/')
        .filter(|&part| !part.is_empty() && part != "." && part != "..")
        .collect::<PathBuf>();

    if clean_relative_path.as_os_str().is_empty() && requested_asset_path != "/" {
        return Err("Leerer oder ungültiger Asset-Pfad".to_string());
    }

    // 5. Setze den finalen Pfad zusammen
    let final_path = specific_extension_dir.join(clean_relative_path);

    // 6. SICHERHEITSCHECK (wie vorher)
    match final_path.canonicalize() {
        Ok(canonical_path) => {
            let canonical_base = specific_extension_dir.canonicalize().map_err(|e| {
                format!(
                    "Kann Basis-Pfad '{}' nicht kanonisieren: {}",
                    specific_extension_dir.display(),
                    e
                )
            })?;
            if canonical_path.starts_with(&canonical_base) {
                Ok(canonical_path)
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err("Ungültiger oder nicht erlaubter Asset-Pfad (kanonisch)".to_string())
            }
        }
        Err(_) => {
            // Fehler bei canonicalize (z.B. Pfad existiert nicht)
            if final_path.starts_with(&specific_extension_dir) {
                Ok(final_path) // Nicht-kanonisierten Pfad zurückgeben
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err("Ungültiger oder nicht erlaubter Asset-Pfad (nicht kanonisiert)".to_string())
            }
        }
    }
}

pub fn extension_protocol_handler<R: Runtime>(
    context: &UriSchemeContext<'_, R>,
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
                context.app_handle(),
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
