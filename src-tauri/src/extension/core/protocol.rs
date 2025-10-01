// src-tauri/src/extension/core/protocol.rs

use crate::extension::error::ExtensionError;
use crate::AppState;
use mime;
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use tauri::http::{Request, Response};
use tauri::{AppHandle, State};

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

impl std::error::Error for DataProcessingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataProcessingError::HexDecoding(e) => Some(e),
            DataProcessingError::Utf8Conversion(e) => Some(e),
            DataProcessingError::JsonParsing(e) => Some(e),
        }
    }
}

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

pub fn resolve_secure_extension_asset_path(
    app_handle: &AppHandle,
    state: State<AppState>,
    extension_id: &str,
    extension_version: &str,
    requested_asset_path: &str,
) -> Result<PathBuf, ExtensionError> {
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

    let specific_extension_dir =
        state
            .extension_manager
            .get_extension_dir(app_handle, extension_id, extension_version)?;

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

    let final_path = specific_extension_dir.join(clean_relative_path);

    match final_path.canonicalize() {
        Ok(canonical_path) => {
            let canonical_base = specific_extension_dir
                .canonicalize()
                .map_err(|e| ExtensionError::Filesystem { source: e })?;
            if canonical_path.starts_with(&canonical_base) {
                Ok(canonical_path)
            } else {
                eprintln!(
                    "SECURITY WARNING: Path traversal attempt blocked: {}",
                    requested_asset_path
                );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Path traversal attempt: {}", requested_asset_path),
                })
            }
        }
        Err(_) => {
            if final_path.starts_with(&specific_extension_dir) {
                Ok(final_path)
            } else {
                eprintln!(
                    "SECURITY WARNING: Invalid asset path: {}",
                    requested_asset_path
                );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Invalid asset path: {}", requested_asset_path),
                })
            }
        }
    }
}

pub fn extension_protocol_handler(
    state: State<AppState>,
    app_handle: &AppHandle,
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
                            "Liefere {} ({}, {} bytes) ",
                            absolute_secure_path.display(),
                            mime_type,
                            content_length
                        );
                        Response::builder()
                            .status(200)
                            .header("Content-Type", mime_type)
                            .header("Content-Length", content_length.to_string())
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
                            .body(Vec::new())
                            .map_err(|e| e.into())
                    }
                }
            } else {
                eprintln!(
                    "Asset nicht gefunden oder ist kein File: {}",
                    absolute_secure_path.display()
                );
                Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .map_err(|e| e.into())
            }
        }
        Err(e) => {
            eprintln!("Fehler bei der Datenverarbeitung: {}", e);

            Response::builder()
                .status(500)
                .body(Vec::new())
                .map_err(|e| e.into())
        }
    }
}

fn process_hex_encoded_json(hex_input: &str) -> Result<ExtensionInfo, DataProcessingError> {
    let bytes = hex::decode(hex_input)?;
    let json_string = String::from_utf8(bytes)?;
    let extension_info: ExtensionInfo = serde_json::from_str(&json_string)?;
    Ok(extension_info)
}
