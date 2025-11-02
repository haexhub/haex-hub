// src-tauri/src/extension/core/protocol.rs

use crate::extension::core::types::get_tauri_origin;
use crate::extension::error::ExtensionError;
use crate::AppState;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use mime;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::http::Uri;
use tauri::http::{Request, Response};
use tauri::{AppHandle, State};

// Extension protocol name constant
pub const EXTENSION_PROTOCOL_NAME: &str = "haex-extension";

// Cache for extension info (used for asset loading without origin header)
lazy_static::lazy_static! {
    static ref EXTENSION_CACHE: Mutex<Option<ExtensionInfo>> = Mutex::new(None);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ExtensionInfo {
    public_key: String,
    name: String,
    version: String,
}

#[derive(Debug)]
enum DataProcessingError {
    HexDecoding(hex::FromHexError),
    Utf8Conversion(std::string::FromUtf8Error),
    JsonParsing(serde_json::Error),
    Custom(String),
}

impl fmt::Display for DataProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataProcessingError::HexDecoding(e) => write!(f, "Hex-Dekodierungsfehler: {e}"),
            DataProcessingError::Utf8Conversion(e) => {
                write!(f, "UTF-8-Konvertierungsfehler: {e}")
            }
            DataProcessingError::JsonParsing(e) => write!(f, "JSON-Parsing-Fehler: {e}"),
            DataProcessingError::Custom(msg) => write!(f, "Datenverarbeitungsfehler: {msg}"),
        }
    }
}

impl std::error::Error for DataProcessingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DataProcessingError::HexDecoding(e) => Some(e),
            DataProcessingError::Utf8Conversion(e) => Some(e),
            DataProcessingError::JsonParsing(e) => Some(e),
            DataProcessingError::Custom(_) => None,
        }
    }
}

impl From<String> for DataProcessingError {
    fn from(msg: String) -> Self {
        DataProcessingError::Custom(msg)
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
    state: &State<AppState>,
    public_key: &str,
    extension_name: &str,
    extension_version: &str,
    requested_asset_path: &str,
) -> Result<PathBuf, ExtensionError> {
    if extension_name.is_empty()
        || !extension_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(ExtensionError::ValidationError {
            reason: format!("Invalid extension name: {extension_name}"),
        });
    }

    if extension_version.is_empty()
        || !extension_version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(ExtensionError::ValidationError {
            reason: format!("Invalid extension version: {extension_version}"),
        });
    }

    let specific_extension_dir = state.extension_manager.get_extension_dir(
        app_handle,
        public_key,
        extension_name,
        extension_version,
    )?;

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
                    "SECURITY WARNING: Path traversal attempt blocked: {requested_asset_path}"
                );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Path traversal attempt: {requested_asset_path}"),
                })
            }
        }
        Err(_) => {
            if final_path.starts_with(&specific_extension_dir) {
                Ok(final_path)
            } else {
                eprintln!(
                    "SECURITY WARNING: Invalid asset path: {requested_asset_path}"
                );
                Err(ExtensionError::SecurityViolation {
                    reason: format!("Invalid asset path: {requested_asset_path}"),
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
    // Get the origin from the request
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Only allow same-protocol requests or tauri origin
    // For null/empty origin (initial load), use wildcard
    let protocol_prefix = format!("{EXTENSION_PROTOCOL_NAME}://");
    let allowed_origin = if origin.starts_with(&protocol_prefix) || origin == get_tauri_origin() {
        origin
    } else if origin.is_empty() || origin == "null" {
        "*" // Allow initial load without origin
    } else {
        // Reject other origins
        return Response::builder()
            .status(403)
            .body(Vec::from("Origin not allowed"))
            .map_err(|e| e.into());
    };

    // Handle OPTIONS requests for CORS preflight
    if request.method() == "OPTIONS" {
        return Response::builder()
            .status(200)
            .header("Access-Control-Allow-Origin", allowed_origin)
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Allow-Credentials", "true")
            .body(Vec::new())
            .map_err(|e| e.into());
    }

    let uri_ref = request.uri();
    let referer = request
        .headers()
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    println!("Protokoll Handler für: {uri_ref}");
    println!("Origin: {origin}");
    println!("Referer: {referer}");

    let path_str = uri_ref.path();

    // Try to decode base64-encoded extension info from URI
    // Format:
    // - Desktop: haex-extension://<base64>/{assetPath}
    // - Android: http://localhost/{base64}/{assetPath}
    let host = uri_ref.host().unwrap_or("");
    println!("URI Host: {host}");

    let (info, segments_after_version) = if host == "localhost" || host == format!("{EXTENSION_PROTOCOL_NAME}.localhost").as_str() {
        // Android format: http://haex-extension.localhost/{base64}/{assetPath}
        // Extract base64 from first path segment
        println!("Android format detected: http://{host}/...");
        let mut segments_iter = path_str.split('/').filter(|s| !s.is_empty());

        if let Some(first_segment) = segments_iter.next() {
            println!("First path segment (base64): {first_segment}");
            match BASE64_STANDARD.decode(first_segment) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(json_str) => match serde_json::from_str::<ExtensionInfo>(&json_str) {
                        Ok(info) => {
                            println!("=== Extension Info from path (Android) ===");
                            println!("  PublicKey: {}", info.public_key);
                            println!("  Name: {}", info.name);
                            println!("  Version: {}", info.version);
                            cache_extension_info(&info);

                            // Remaining segments after base64 are the asset path
                            let remaining: Vec<String> = segments_iter.map(|s| s.to_string()).collect();
                            (info, remaining)
                        }
                        Err(e) => {
                            eprintln!("Failed to parse JSON from base64 path: {e}");
                            return Response::builder()
                                .status(400)
                                .header("Access-Control-Allow-Origin", allowed_origin)
                                .body(Vec::from(format!("Invalid extension info in base64 path: {e}")))
                                .map_err(|e| e.into());
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to decode UTF-8 from base64 path: {e}");
                        return Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", allowed_origin)
                            .body(Vec::from(format!("Invalid UTF-8 in base64 path: {e}")))
                            .map_err(|e| e.into());
                    }
                },
                Err(e) => {
                    eprintln!("Failed to decode base64 from path: {e}");
                    return Response::builder()
                        .status(400)
                        .header("Access-Control-Allow-Origin", allowed_origin)
                        .body(Vec::from(format!("Invalid base64 in path: {e}")))
                        .map_err(|e| e.into());
                }
            }
        } else {
            eprintln!("No path segment found for Android format");
            return Response::builder()
                .status(400)
                .header("Access-Control-Allow-Origin", allowed_origin)
                .body(Vec::from("No base64 segment found in path"))
                .map_err(|e| e.into());
        }
    } else if host != "localhost" && !host.is_empty() {
        // Desktop format: haex-extension://<base64>/{assetPath}
        println!("Desktop format detected: haex-extension://<base64>/...");
        match BASE64_STANDARD.decode(host) {
            Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                Ok(json_str) => match serde_json::from_str::<ExtensionInfo>(&json_str) {
                    Ok(info) => {
                        println!("=== Extension Info from base64-encoded host ===");
                        println!("  PublicKey: {}", info.public_key);
                        println!("  Name: {}", info.name);
                        println!("  Version: {}", info.version);
                        cache_extension_info(&info);

                        // Parse path segments as asset path
                        // Format: haex-extension://<base64>/{asset_path}
                        // All extension info is in the base64-encoded host
                        let segments: Vec<String> = path_str
                            .split('/')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();

                        (info, segments)
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON from base64 host: {e}");
                        return Response::builder()
                            .status(400)
                            .header("Access-Control-Allow-Origin", allowed_origin)
                            .body(Vec::from(format!("Invalid extension info in base64 host: {e}")))
                            .map_err(|e| e.into());
                    }
                },
                Err(e) => {
                    eprintln!("Failed to decode UTF-8 from base64 host: {e}");
                    return Response::builder()
                        .status(400)
                        .header("Access-Control-Allow-Origin", allowed_origin)
                        .body(Vec::from(format!("Invalid UTF-8 in base64 host: {e}")))
                        .map_err(|e| e.into());
                }
            },
            Err(e) => {
                eprintln!("Failed to decode base64 host: {e}");
                return Response::builder()
                    .status(400)
                    .header("Access-Control-Allow-Origin", allowed_origin)
                    .body(Vec::from(format!("Invalid base64 in host: {e}")))
                    .map_err(|e| e.into());
            }
        }
    } else {
        // No base64 host - use path-based parsing (for localhost/Android/Windows)
        parse_extension_info_from_path(path_str, origin, uri_ref, referer, allowed_origin)?
    };

    // Construct asset path from remaining segments
    let raw_asset_path = segments_after_version.join("/");

    // Simple asset loading: if path is empty, serve index.html, otherwise try to load the asset
    // This is framework-agnostic and lets the file system determine if it exists
    let asset_to_load = if raw_asset_path.is_empty() {
        "index.html"
    } else {
        &raw_asset_path
    };

    println!("Path: {path_str}");
    println!("Asset to load: {asset_to_load}");

    let absolute_secure_path = resolve_secure_extension_asset_path(
        app_handle,
        &state,
        &info.public_key,
        &info.name,
        &info.version,
        asset_to_load,
    )?;

    println!("Resolved path: {}", absolute_secure_path.display());
    println!("File exists: {}", absolute_secure_path.exists());

    if absolute_secure_path.exists() && absolute_secure_path.is_file() {
        match fs::read(&absolute_secure_path) {
            Ok(content) => {
                let mime_type = mime_guess::from_path(&absolute_secure_path)
                    .first_or(mime::APPLICATION_OCTET_STREAM)
                    .to_string();

                // Note: Base tag and polyfills are now injected by the SDK at runtime
                // No server-side HTML modification needed

                let content_length = content.len();
                println!(
                    "Liefere {} ({}, {} bytes) ",
                    absolute_secure_path.display(),
                    mime_type,
                    content_length
                );
                Response::builder()
                    .status(200)
                    .header("Content-Type", &mime_type)
                    .header("Content-Length", content_length.to_string())
                    .header("Accept-Ranges", "bytes")
                    .header("Access-Control-Allow-Origin", allowed_origin)
                    .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "*")
                    .header("Access-Control-Allow-Credentials", "true")
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
                    .header("Access-Control-Allow-Origin", allowed_origin)
                    .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "*")
                    .body(Vec::new())
                    .map_err(|e| e.into())
            }
        }
    } else {
        // Asset not found - try index.html fallback for SPA routing
        // This allows client-side routing to work (e.g., /settings -> index.html)
        if asset_to_load != "index.html" {
            eprintln!(
                "Asset nicht gefunden: {}, versuche index.html fallback für SPA routing",
                absolute_secure_path.display()
            );

            let index_path = resolve_secure_extension_asset_path(
                app_handle,
                &state,
                &info.public_key,
                &info.name,
                &info.version,
                "index.html",
            )?;

            if index_path.exists() && index_path.is_file() {
                match fs::read(&index_path) {
                    Ok(content) => {
                        let mime_type = "text/html";

                        // Note: Base tag and polyfills are injected by SDK at runtime

                        let content_length = content.len();
                        return Response::builder()
                            .status(200)
                            .header("Content-Type", mime_type)
                            .header("Content-Length", content_length.to_string())
                            .header("Accept-Ranges", "bytes")
                            .header("Access-Control-Allow-Origin", allowed_origin)
                            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                            .header("Access-Control-Allow-Headers", "*")
                            .header("Access-Control-Allow-Credentials", "true")
                            .body(content)
                            .map_err(|e| e.into());
                    }
                    Err(_) => {
                        // Fall through to 404
                    }
                }
            }
        }

        // No fallback available - return 404
        eprintln!(
            "Asset nicht gefunden oder ist kein File: {}",
            absolute_secure_path.display()
        );
        Response::builder()
            .status(404)
            .header("Access-Control-Allow-Origin", allowed_origin)
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "*")
            .body(Vec::new())
            .map_err(|e| e.into())
    }
}

fn process_hex_encoded_json(hex_input: &str) -> Result<ExtensionInfo, DataProcessingError> {
    let bytes = hex::decode(hex_input)?;
    let json_string = String::from_utf8(bytes)?;
    let extension_info: ExtensionInfo = serde_json::from_str(&json_string)?;
    Ok(extension_info)
}

fn parse_encoded_info_from_origin_or_uri_or_referer_or_cache(
    origin: &str,
    uri_ref: &Uri,
    referer: &str,
) -> Result<ExtensionInfo, DataProcessingError> {
    // Return direkt ExtensionInfo (dekodiert)
    // 1-3. Bestehende Fallbacks (wie vorher, aber return decoded Info statt hex)
    if !origin.is_empty() && origin != "null" {
        if let Ok(hex) = parse_from_origin(origin) {
            if let Ok(info) = process_hex_encoded_json(&hex) {
                cache_extension_info(&info); // Cache setzen
                println!("Parsed und gecached aus Origin: {hex}");
                return Ok(info);
            }
        }
    }

    println!("Fallback zu URI-Parsing");
    if let Ok(hex) = parse_from_uri_path(uri_ref) {
        if let Ok(info) = process_hex_encoded_json(&hex) {
            cache_extension_info(&info); // Cache setzen
            println!("Parsed und gecached aus URI: {hex}");
            return Ok(info);
        }
    }

    println!("Fallback zu Referer-Parsing: {referer}");
    if !referer.is_empty() && referer != "null" {
        if let Ok(hex) = parse_from_uri_string(referer) {
            if let Ok(info) = process_hex_encoded_json(&hex) {
                cache_extension_info(&info); // Cache setzen
                println!("Parsed und gecached aus Referer: {hex}");
                return Ok(info);
            }
        }
    }

    // 4. Fallback: Globaler Cache (für Assets in derselben Session)
    println!("Fallback zu Cache");
    if let Some(cached_info) = get_cached_extension_info() {
        println!(
            "Gecached Info verwendet: PublicKey={}, Name={}, Version={}",
            cached_info.public_key, cached_info.name, cached_info.version
        );
        return Ok(cached_info);
    }

    Err(
        "Kein gültiger Hex in Origin, URI, Referer oder Cache gefunden"
            .to_string()
            .into(),
    )
}

// NEU: Cache-Helper (Mutex-sicher)
fn cache_extension_info(info: &ExtensionInfo) {
    if let Ok(mut cache) = EXTENSION_CACHE.lock() {
        *cache = Some(info.clone());
    }
}

fn get_cached_extension_info() -> Option<ExtensionInfo> {
    if let Ok(cache) = EXTENSION_CACHE.lock() {
        cache.clone()
    } else {
        None
    }
}

fn parse_hex_from_url_string(url_str: &str) -> Result<String, DataProcessingError> {
    // Suche nach Scheme-Ende (://)
    let scheme_end = match url_str.find("://") {
        Some(pos) => pos + 3, // Nach "://"
        _none => return Err("Kein Scheme in URL".to_string().into()),
    };

    let after_scheme = &url_str[scheme_end..];
    let path_start = match after_scheme.find('/') {
        Some(pos) => pos,
        _none => return Err("Kein Path in URL".to_string().into()),
    };

    let path = &after_scheme[path_start..]; // z.B. "/7b22.../index.html"
    let mut segments = path.split('/').filter(|s| !s.is_empty());

    let first_segment = match segments.next() {
        Some(seg) => seg,
        _none => return Err("Kein Path-Segment in URL".to_string().into()),
    };

    validate_and_return_hex(first_segment)
}

// Vereinfachte parse_from_origin
fn parse_from_origin(origin: &str) -> Result<String, DataProcessingError> {
    parse_hex_from_url_string(origin)
}

// Vereinfachte parse_from_uri_path
fn parse_from_uri_path(uri_ref: &Uri) -> Result<String, DataProcessingError> {
    let uri_str = uri_ref.to_string();
    parse_hex_from_url_string(&uri_str)
}

// Vereinfachte parse_from_uri_string (für Referer)
fn parse_from_uri_string(uri_str: &str) -> Result<String, DataProcessingError> {
    parse_hex_from_url_string(uri_str)
}

// validate_and_return_hex bleibt unverändert (aus letztem Vorschlag)
fn validate_and_return_hex(segment: &str) -> Result<String, DataProcessingError> {
    if segment.is_empty() {
        return Err("Kein Extension-Info (hex) im Path".to_string().into());
    }
    if segment.len() % 2 != 0 {
        return Err("Ungültiger Hex: Ungerade Länge".to_string().into());
    }
    if !segment.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Ungültiger Hex: Ungültige Zeichen".to_string().into());
    }
    Ok(segment.to_string())
}

// Helper function to parse extension info from path segments
fn parse_extension_info_from_path(
    path_str: &str,
    origin: &str,
    uri_ref: &Uri,
    referer: &str,
    allowed_origin: &str,
) -> Result<(ExtensionInfo, Vec<String>), Box<dyn std::error::Error>> {
    let mut segments_iter = path_str.split('/').filter(|s| !s.is_empty());

    match (segments_iter.next(), segments_iter.next(), segments_iter.next()) {
        (Some(public_key), Some(name), Some(version)) => {
            println!("=== Extension Protocol Handler (path-based) ===");
            println!("Full URI: {uri_ref}");
            println!("Parsed from path segments:");
            println!("  PublicKey: {public_key}");
            println!("  Name: {name}");
            println!("  Version: {version}");

            let info = ExtensionInfo {
                public_key: public_key.to_string(),
                name: name.to_string(),
                version: version.to_string(),
            };

            cache_extension_info(&info);

            // Collect remaining segments as asset path (owned strings)
            let remaining: Vec<String> = segments_iter.map(|s| s.to_string()).collect();

            Ok((info, remaining))
        }
        _ => {
            // Fallback: Try hex-encoded format for backwards compatibility
            match parse_encoded_info_from_origin_or_uri_or_referer_or_cache(
                origin, uri_ref, referer,
            ) {
                Ok(decoded) => {
                    println!("=== Extension Protocol Handler (legacy hex format) ===");
                    println!("Full URI: {uri_ref}");
                    println!("Decoded info:");
                    println!("  PublicKey: {}", decoded.public_key);
                    println!("  Name: {}", decoded.name);
                    println!("  Version: {}", decoded.version);

                    // For legacy format, collect all segments after parsing (owned strings)
                    let segments: Vec<String> = path_str
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .skip(1) // Skip the hex segment
                        .map(|s| s.to_string())
                        .collect();

                    Ok((decoded, segments))
                }
                Err(e) => {
                    eprintln!("Fehler beim Parsen (alle Fallbacks): {e}");
                    Err(format!("Ungültige Anfrage: {e}").into())
                }
            }
        }
    }
}
