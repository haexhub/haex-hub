use mime;
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use tauri::{
    http::{Request, Response},
    AppHandle, Error as TauriError, Manager, Runtime, UriSchemeContext,
};

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
    let asset_to_load = if raw_asset_path.is_empty() { "index.html"} else {&raw_asset_path};

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
