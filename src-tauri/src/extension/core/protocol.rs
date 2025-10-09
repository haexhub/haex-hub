// src-tauri/src/extension/core/protocol.rs

use crate::extension::core::types::get_tauri_origin;
use crate::extension::error::ExtensionError;
use crate::AppState;
use mime;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::http::Uri;
use tauri::http::{Request, Response};
use tauri::{AppHandle, State};

// Cache for modified HTML files (extension_id -> modified content)
lazy_static::lazy_static! {
    static ref HTML_CACHE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
    static ref EXTENSION_CACHE: Mutex<Option<ExtensionInfo>> = Mutex::new(None);
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ExtensionInfo {
    key_hash: String,
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
            DataProcessingError::HexDecoding(e) => write!(f, "Hex-Dekodierungsfehler: {}", e),
            DataProcessingError::Utf8Conversion(e) => {
                write!(f, "UTF-8-Konvertierungsfehler: {}", e)
            }
            DataProcessingError::JsonParsing(e) => write!(f, "JSON-Parsing-Fehler: {}", e),
            DataProcessingError::Custom(msg) => write!(f, "Datenverarbeitungsfehler: {}", msg),
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
    state: State<AppState>,
    key_hash: &str,
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
            reason: format!("Invalid extension name: {}", extension_name),
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

    let specific_extension_dir = state.extension_manager.get_extension_dir(
        app_handle,
        key_hash,
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
    // Get the origin from the request
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Only allow same-protocol requests (haex-extension://) or tauri origin
    // For null/empty origin (initial load), use wildcard
    let allowed_origin = if origin.starts_with("haex-extension://") || origin == get_tauri_origin()
    {
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

    println!("Protokoll Handler für: {}", uri_ref);
    println!("Origin: {}", origin);
    println!("Referer: {}", referer);

    /* let encoded_info =
    match parse_encoded_info_from_origin_or_uri_or_referer(&origin, uri_ref, &referer) {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Fehler beim Parsen des Origin für Extension-Info: {}", e);
            return Response::builder()
                .status(400)
                .header("Access-Control-Allow-Origin", allowed_origin)
                .body(Vec::from(format!("Ungültiger Origin: {}", e)))
                .map_err(|e| e.into());
        }
    }; */

    let info =
        match parse_encoded_info_from_origin_or_uri_or_referer_or_cache(&origin, uri_ref, &referer)
        {
            Ok(decoded) => {
                println!("=== Extension Protocol Handler ===");
                println!("Full URI: {}", uri_ref);
                println!(
                    "Encoded Info (aus Origin/URI/Referer/Cache): {}",
                    encode_hex_for_log(&decoded)
                ); // Hilfs-Log
                println!("Decoded info:");
                println!("  KeyHash: {}", decoded.key_hash);
                println!("  Name: {}", decoded.name);
                println!("  Version: {}", decoded.version);
                decoded
            }
            Err(e) => {
                eprintln!("Fehler beim Parsen (alle Fallbacks): {}", e);
                return Response::builder()
                    .status(400)
                    .header("Access-Control-Allow-Origin", allowed_origin)
                    .body(Vec::from(format!("Ungültige Anfrage: {}", e)))
                    .map_err(|e| e.into());
            }
        };

    let path_str = uri_ref.path();
    let segments_iter = path_str.split('/').filter(|s| !s.is_empty());
    let resource_segments: Vec<&str> = segments_iter.collect();
    let raw_asset_path = resource_segments.join("/");

    // Handle SPA routing - serve index.html for all non-asset paths
    let asset_to_load = if raw_asset_path.is_empty() {
        "index.html"
    } else if raw_asset_path.starts_with("_nuxt/")
        || raw_asset_path.ends_with(".js")
        || raw_asset_path.ends_with(".css")
        || raw_asset_path.ends_with(".json")
        || raw_asset_path.ends_with(".ico")
        || raw_asset_path.ends_with(".txt")
        || raw_asset_path.ends_with(".svg")
        || raw_asset_path.ends_with(".png")
        || raw_asset_path.ends_with(".jpg")
        || raw_asset_path.ends_with(".jpeg")
        || raw_asset_path.ends_with(".gif")
        || raw_asset_path.ends_with(".woff")
        || raw_asset_path.ends_with(".woff2")
        || raw_asset_path.ends_with(".ttf")
        || raw_asset_path.ends_with(".eot")
    {
        // Serve actual asset
        &raw_asset_path
    } else {
        // SPA fallback - serve index.html for routes
        "index.html"
    };

    println!("Path: {}", path_str);
    println!("Asset to load: {}", asset_to_load);

    /*  match process_hex_encoded_json(&encoded_info) {
           Ok(info) => {
               println!("=== Extension Protocol Handler ===");
               println!("Full URI: {}", uri_ref);
               println!("Origin: {}", origin);
               println!("Encoded Info (aus Origin): {}", encoded_info);
               println!("Path: {}", path_str);
               println!("Asset to load: {}", asset_to_load);
               println!("Decoded info:");
               println!("  KeyHash: {}", info.key_hash);
               println!("  Name: {}", info.name);
               println!("  Version: {}", info.version);

               let absolute_secure_path = resolve_secure_extension_asset_path(
                   app_handle,
                   state,
                   &info.key_hash,
                   &info.name,
                   &info.version,
                   &asset_to_load,
               )?;

               println!("Resolved path: {}", absolute_secure_path.display());
               println!("File exists: {}", absolute_secure_path.exists());

               if absolute_secure_path.exists() && absolute_secure_path.is_file() {
                   match fs::read(&absolute_secure_path) {
                       Ok(mut content) => {
                           let mime_type = mime_guess::from_path(&absolute_secure_path)
                               .first_or(mime::APPLICATION_OCTET_STREAM)
                               .to_string();

                           if asset_to_load == "index.html" && mime_type.contains("html") {
                               if let Ok(html_str) = String::from_utf8(content.clone()) {
                                   let base_tag = format!(r#"<base href="/{}/">"#, encoded_info);
                                   let modified_html = if let Some(head_pos) = html_str.find("<head>")
                                   {
                                       let insert_pos = head_pos + 6;
                                       format!(
                                           "{}{}<head>{}",
                                           &html_str[..insert_pos],
                                           base_tag,
                                           &html_str[insert_pos..]
                                       )
                                   } else {
                                       // Fallback: Prepend
                                       format!("{}{}", base_tag, html_str)
                                   };
                                   content = modified_html.into_bytes();
                               }
                           }
                           // Inject localStorage polyfill for HTML files with caching
                           if asset_to_load == "index.html" && mime_type.contains("html") {
                                                       // Create cache key: extension_id (from host)
                                                       let cache_key = format!("{}_{}", host, asset_to_load);

                                                       // Check cache first
                                                       if let Ok(cache) = HTML_CACHE.lock() {
                                                           if let Some(cached_content) = cache.get(&cache_key) {
                                                               println!("Serving cached HTML for: {}", cache_key);
                                                               content = cached_content.clone();

                                                               let content_length = content.len();
                                                               return Response::builder()
                                                                   .status(200)
                                                                   .header("Content-Type", mime_type)
                                                                   .header("Content-Length", content_length.to_string())
                                                                   .header("Accept-Ranges", "bytes")
                                                                   .header("X-HaexHub-Cache", "HIT")
                                                                   .header("Access-Control-Allow-Origin", allowed_origin)
                                                                   .header(
                                                                       "Access-Control-Allow-Methods",
                                                                       "GET, POST, OPTIONS",
                                                                   )
                                                                   .header("Access-Control-Allow-Headers", "*")
                                                                   .header("Access-Control-Allow-Credentials", "true")
                                                                   .body(content)
                                                                   .map_err(|e| e.into());
                                                           }
                                                       }

                                                       // Not in cache, modify and cache it
                                                       if let Ok(html_content) = String::from_utf8(content.clone()) {
                                                           let polyfill = r#"<script>
                           // HaexHub localStorage polyfill for custom protocol
                           (function() {
                             try {
                               // Test if localStorage is available
                               let localStorageAvailable = false;
                               try {
                                 window.localStorage.setItem('__test__', '1');
                                 window.localStorage.removeItem('__test__');
                                 localStorageAvailable = true;
                               } catch (e) {
                                 // localStorage is blocked
                               }

                               if (!localStorageAvailable) {
                                 // Create in-memory storage fallback
                                 const storage = new Map();
                                 const storagePolyfill = {
                                   getItem: (key) => storage.get(key) ?? null,
                                   setItem: (key, value) => storage.set(key, String(value)),
                                   removeItem: (key) => storage.delete(key),
                                   clear: () => storage.clear(),
                                   get length() { return storage.size; },
                                   key: (index) => Array.from(storage.keys())[index] ?? null,
                                 };

                                 // Try to replace localStorage
                                 try {
                                   delete window.localStorage;
                                   window.localStorage = storagePolyfill;
                                 } catch (e) {
                                   // On some platforms, we can't delete localStorage
                                   // Try to override methods instead
                                   Object.defineProperty(window, 'localStorage', {
                                     value: storagePolyfill,
                                     writable: true,
                                     configurable: true
                                   });
                                 }

                                 // Also replace sessionStorage
                                 try {
                                   delete window.sessionStorage;
                                   window.sessionStorage = {
                                     getItem: (key) => null,
                                     setItem: () => {},
                                     removeItem: () => {},
                                     clear: () => {},
                                     get length() { return 0; },
                                     key: () => null,
                                   };
                                 } catch (e) {
                                   // sessionStorage replacement failed, not critical
                                 }

                                 console.log('[HaexHub] localStorage replaced with in-memory polyfill');
                               }
                             } catch (e) {
                               console.error('[HaexHub] Polyfill initialization failed:', e);
                             }
                           })();
                           </script>"#;

                                                           // Inject as the FIRST thing in <head>, before any other script
                                                           let modified_html = if let Some(head_pos) =
                                                               html_content.find("<head>")
                                                           {
                                                               let insert_pos = head_pos + 6; // After <head>
                                                               format!(
                                                                   "{}{}{}",
                                                                   &html_content[..insert_pos],
                                                                   polyfill,
                                                                   &html_content[insert_pos..]
                                                               )
                                                           } else if let Some(charset_pos) = html_content.find("<meta charset")
                                                           {
                                                               // Insert before first meta tag
                                                               format!(
                                                                   "{}<head>{}</head>{}",
                                                                   &html_content[..charset_pos],
                                                                   polyfill,
                                                                   &html_content[charset_pos..]
                                                               )
                                                           } else if let Some(html_pos) = html_content.find("<!DOCTYPE html>")
                                                           {
                                                               // Insert right after DOCTYPE
                                                               let insert_pos = html_pos + 15; // After <!DOCTYPE html>
                                                               format!(
                                                                   "{}<head>{}</head>{}",
                                                                   &html_content[..insert_pos],
                                                                   polyfill,
                                                                   &html_content[insert_pos..]
                                                               )
                                                           } else {
                                                               // Prepend to entire file
                                                               format!("{}{}", polyfill, html_content)
                                                           };

                                                           content = modified_html.into_bytes();

                                                           // Cache the modified HTML
                                                           if let Ok(mut cache) = HTML_CACHE.lock() {
                                                               cache.insert(cache_key, content.clone());
                                                               println!("Cached modified HTML for future requests");
                                                           }
                                                       }
                                                   }

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
                               .header(
                                   "X-HaexHub-Cache",
                                   if asset_to_load == "index.html" && mime_type.contains("html") {
                                       "MISS"
                                   } else {
                                       "N/A"
                                   },
                               )
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
           Err(e) => {
               eprintln!("Fehler bei der Datenverarbeitung: {}", e);

               Response::builder()
                   .status(500)
                   .header("Access-Control-Allow-Origin", allowed_origin)
                   .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                   .header("Access-Control-Allow-Headers", "*")
                   .body(Vec::new())
                   .map_err(|e| e.into())
           }
       }
    */

    let absolute_secure_path = resolve_secure_extension_asset_path(
        app_handle,
        state,
        &info.key_hash,
        &info.name,
        &info.version,
        &asset_to_load,
    )?;

    println!("Resolved path: {}", absolute_secure_path.display());
    println!("File exists: {}", absolute_secure_path.exists());

    if absolute_secure_path.exists() && absolute_secure_path.is_file() {
        match fs::read(&absolute_secure_path) {
            Ok(mut content) => {
                let mime_type = mime_guess::from_path(&absolute_secure_path)
                    .first_or(mime::APPLICATION_OCTET_STREAM)
                    .to_string();

                // Für index.html – injiziere <base> Tag + localStorage-Polyfill
                if asset_to_load == "index.html" && mime_type.contains("html") {
                    // Cache-Key erstellen (extension-host + asset)
                    let host = uri_ref
                        .host()
                        .map_or("unknown".to_string(), |h| h.to_string());
                    let cache_key = format!("{}_{}", host, asset_to_load);

                    // Cache checken (aus deinem alten Code)
                    if let Ok(cache_guard) = HTML_CACHE.lock() {
                        if let Some(cached_content) = cache_guard.get(&cache_key) {
                            println!("Serving cached HTML for: {}", cache_key);
                            let content_length = cached_content.len();
                            return Response::builder()
                                .status(200)
                                .header("Content-Type", &mime_type)
                                .header("Content-Length", content_length.to_string())
                                .header("Accept-Ranges", "bytes")
                                .header("X-HaexHub-Cache", "HIT")
                                .header("Access-Control-Allow-Origin", allowed_origin)
                                .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                                .header("Access-Control-Allow-Headers", "*")
                                .header("Access-Control-Allow-Credentials", "true")
                                .body(cached_content.clone())
                                .map_err(|e| e.into());
                        }
                    }

                    // Nicht gecacht: Modifiziere HTML
                    if let Ok(html_str) = String::from_utf8(content.clone()) {
                        // 1. Polyfill injizieren (als ERSTES in <head>)
                        let polyfill_script = r#"<script>
(function() {
  'use strict';

    console.log('[HaexHub] Storage Polyfill loading immediately');

  // Test ob localStorage verfügbar ist
  let localStorageWorks = false;
  try {
    const testKey = '__ls_test__';
    localStorage.setItem(testKey, testKey);
    localStorage.removeItem(testKey);
    localStorageWorks = true;
  } catch (e) {
    console.warn('[HaexHub] localStorage blocked – using in-memory fallback');
  }

  // Wenn blockiert: Ersetze mit In-Memory Storage
  if (!localStorageWorks) {
    const lsStorage = new Map();
    const localStoragePoly = {
      getItem: function(key) {
        return lsStorage.get(key) || null;
      },
      setItem: function(key, value) {
        lsStorage.set(key, String(value));
      },
      removeItem: function(key) {
        lsStorage.delete(key);
      },
      clear: function() {
        lsStorage.clear();
      },
      get length() {
        return lsStorage.size;
      },
      key: function(index) {
        return Array.from(lsStorage.keys())[index] || null;
      }
    };

    try {
      Object.defineProperty(window, 'localStorage', {
        value: localStoragePoly,
        writable: true,
        configurable: true
      });
    } catch (e) {
      // Fallback: Direct assignment
      window.localStorage = localStoragePoly;
    }
  }

  // SessionStorage Polyfill (immer ersetzen)
  try {
    const sessionStoragePoly = {
      getItem: function(key) { return null; },
      setItem: function() {},
      removeItem: function() {},
      clear: function() {},
      get length() { return 0; },
      key: function() { return null; }
    };

    Object.defineProperty(window, 'sessionStorage', {
      value: sessionStoragePoly,
      writable: true,
      configurable: true
    });
  } catch (e) {
    // Fallback: Direct assignment
    window.sessionStorage = {
      getItem: function(key) { return null; },
      setItem: function() {},
      removeItem: function() {},
      clear: function() {},
      get length() { return 0; },
      key: function() { return null; }
    };
  }

  // Cookie Polyfill - Test if cookies are available
  let cookiesWork = false;
  try {
    document.cookie = '__cookie_test__=1';
    cookiesWork = document.cookie.indexOf('__cookie_test__') !== -1;
    document.cookie = '__cookie_test__=; expires=Thu, 01 Jan 1970 00:00:00 GMT';
  } catch (e) {
    console.warn('[HaexHub] Cookies blocked – using in-memory fallback');
  }

  if (!cookiesWork) {
    const cookieStore = new Map();

    const parseCookie = function(str) {
      return str.split(';').reduce((acc, pair) => {
        const [key, value] = pair.trim().split('=');
        if (key) acc[key] = value || '';
        return acc;
      }, {});
    };

    const serializeCookie = function(key, value, options = {}) {
      let cookie = `${key}=${value}`;

      if (options.expires) {
        cookie += `; expires=${options.expires}`;
      }
      if (options.path) {
        cookie += `; path=${options.path}`;
      }
      if (options.domain) {
        cookie += `; domain=${options.domain}`;
      }
      if (options.secure) {
        cookie += '; secure';
      }
      if (options.httpOnly) {
        cookie += '; httponly';
      }
      if (options.sameSite) {
        cookie += `; samesite=${options.sameSite}`;
      }

      return cookie;
    };

    Object.defineProperty(document, 'cookie', {
      get: function() {
        const cookies = [];
        cookieStore.forEach((value, key) => {
          cookies.push(`${key}=${value}`);
        });
        return cookies.join('; ');
      },
      set: function(cookieString) {
        const parts = cookieString.split(';').map(p => p.trim());
        const [keyValue] = parts;
        const [key, value] = keyValue.split('=');

        if (!key) return;

        // Parse options
        const options = {};
        for (let i = 1; i < parts.length; i++) {
          const [optKey, optValue] = parts[i].split('=');
          options[optKey.toLowerCase()] = optValue || true;
        }

        // Check for deletion (expires in past)
        if (options.expires) {
          const expiresDate = new Date(options.expires);
          if (expiresDate < new Date()) {
            cookieStore.delete(key);
            return;
          }
        }

        // Check for max-age=0 deletion
        if (options['max-age'] === '0' || options['max-age'] === 0) {
          cookieStore.delete(key);
          return;
        }

        // Store cookie
        cookieStore.set(key, value || '');
      },
      configurable: true
    });

    console.log('[HaexHub] Cookie polyfill installed');
  }

  // HISTORY PATCH - läuft auch sofort
  document.addEventListener('DOMContentLoaded', function() {
    console.log('[HaexHub] History Patch loading');

    // HISTORY PATCH (erweitert für Nuxt)
    const originalPushState = history.pushState;
    const originalReplaceState = history.replaceState;
    let skipNextPush = false;
    let skipNextReplace = false;

    history.pushState = function(state, title, url) {
      console.log('[HaexHub] pushState called:', url, 'skip:', skipNextPush);

      if (skipNextPush) {
        skipNextPush = false;
        console.log('[HaexHub] pushState skipped');
        return;
      }

      try {
        return originalPushState.call(this, state, title, url);
      } catch (e) {
        if (e.name === 'SecurityError') {
          // Remove duplicate /#/ prefix
          let hashUrl = url.replace(/^\/#/, '');
          hashUrl = hashUrl.startsWith('#') ? hashUrl : '#' + hashUrl;
          console.log('[HaexHub] SecurityError - setting hash to:', hashUrl);
          skipNextPush = true;
          window.location.hash = hashUrl.replace(/^#/, '');
          return;  // Silent
        }
        throw e;
      }
    };

    history.replaceState = function(state, title, url) {
      console.log('[HaexHub] replaceState called:', url, 'skip:', skipNextReplace);

      if (skipNextReplace) {
        skipNextReplace = false;
        console.log('[HaexHub] replaceState skipped');
        return;
      }

      try {
        return originalReplaceState.call(this, state, title, url);
      } catch (e) {
        if (e.name === 'SecurityError') {
          // Remove duplicate /#/ prefix
          let hashUrl = url.replace(/^\/#/, '');
          hashUrl = hashUrl.startsWith('#') ? hashUrl : '#' + hashUrl;
          console.log('[HaexHub] SecurityError - setting hash to:', hashUrl);
          skipNextReplace = true;
          window.location.hash = hashUrl.replace(/^#/, '');
          return;  // Silent
        }
        throw e;
      }
    };

    console.log('[HaexHub] Polyfill loaded – Storage & History patched');
  }, { once: true });  // DOMContentLoaded only once
})();
</script>"#;

                        // 2. Base-Tag erstellen
                        let base_tag = format!(r#"<base href="/{}/">"#, encode_hex_for_log(&info));

                        // 3. Beide in <head> injizieren: Polyfill zuerst, dann Base-Tag
                        let modified_html = if let Some(head_pos) = html_str.find("<head>") {
                            let insert_pos = head_pos + 6; // Nach <head>
                            format!(
                                "{}{}{}{}",
                                &html_str[..insert_pos],
                                polyfill_script,
                                base_tag,
                                &html_str[insert_pos..]
                            )
                        } else {
                            // Kein <head> gefunden - prepend
                            format!("{}{}{}", polyfill_script, base_tag, html_str)
                        };

                        content = modified_html.into_bytes();

                        // Cache die modifizierte HTML (aus deinem alten Code)
                        if let Ok(mut cache_guard) = HTML_CACHE.lock() {
                            cache_guard.insert(cache_key, content.clone());
                            println!("Cached modified HTML for future requests");
                        }
                    }
                }

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
                    .header(
                        "X-HaexHub-Cache",
                        if asset_to_load == "index.html" && mime_type.contains("html") {
                            "MISS"
                        } else {
                            "N/A"
                        },
                    )
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
                println!("Parsed und gecached aus Origin: {}", hex);
                return Ok(info);
            }
        }
    }

    println!("Fallback zu URI-Parsing");
    if let Ok(hex) = parse_from_uri_path(uri_ref) {
        if let Ok(info) = process_hex_encoded_json(&hex) {
            cache_extension_info(&info); // Cache setzen
            println!("Parsed und gecached aus URI: {}", hex);
            return Ok(info);
        }
    }

    println!("Fallback zu Referer-Parsing: {}", referer);
    if !referer.is_empty() && referer != "null" {
        if let Ok(hex) = parse_from_uri_string(referer) {
            if let Ok(info) = process_hex_encoded_json(&hex) {
                cache_extension_info(&info); // Cache setzen
                println!("Parsed und gecached aus Referer: {}", hex);
                return Ok(info);
            }
        }
    }

    // 4. Fallback: Globaler Cache (für Assets in derselben Session)
    println!("Fallback zu Cache");
    if let Some(cached_info) = get_cached_extension_info() {
        println!(
            "Gecached Info verwendet: KeyHash={}, Name={}, Version={}",
            cached_info.key_hash, cached_info.name, cached_info.version
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

fn encode_hex_for_log(info: &ExtensionInfo) -> String {
    let json_str = serde_json::to_string(info).unwrap_or_default();
    hex::encode(json_str.as_bytes())
}
