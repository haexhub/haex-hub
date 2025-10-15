/// src-tauri/src/extension/mod.rs
use crate::{
    extension::{
        core::{EditablePermissions, ExtensionInfoResponse, ExtensionPreview},
        error::ExtensionError,
    },
    AppState,
};
use tauri::{AppHandle, State};
pub mod core;
pub mod crypto;
pub mod database;
pub mod error;
pub mod filesystem;
pub mod permissions;

#[tauri::command]
pub fn get_extension_info(
    public_key: String,
    name: String,
    state: State<AppState>,
) -> Result<ExtensionInfoResponse, ExtensionError> {
    let extension = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
        .ok_or_else(|| ExtensionError::NotFound {
            public_key: public_key.clone(),
            name: name.clone(),
        })?;

    ExtensionInfoResponse::from_extension(&extension)
}

#[tauri::command]
pub async fn get_all_extensions(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ExtensionInfoResponse>, String> {
    // Check if extensions are loaded, if not load them first
    let needs_loading = {
        let prod_exts = state
            .extension_manager
            .production_extensions
            .lock()
            .unwrap();
        let dev_exts = state.extension_manager.dev_extensions.lock().unwrap();
        prod_exts.is_empty() && dev_exts.is_empty()
    };

    if needs_loading {
        state
            .extension_manager
            .load_installed_extensions(&app_handle, &state)
            .await
            .map_err(|e| format!("Failed to load extensions: {:?}", e))?;
    }

    let mut extensions = Vec::new();

    // Production Extensions
    {
        let prod_exts = state
            .extension_manager
            .production_extensions
            .lock()
            .unwrap();
        for ext in prod_exts.values() {
            extensions.push(ExtensionInfoResponse::from_extension(ext)?);
        }
    }

    // Dev Extensions
    {
        let dev_exts = state.extension_manager.dev_extensions.lock().unwrap();
        for ext in dev_exts.values() {
            extensions.push(ExtensionInfoResponse::from_extension(ext)?);
        }
    }

    Ok(extensions)
}

#[tauri::command]
pub async fn preview_extension(
    state: State<'_, AppState>,
    file_bytes: Vec<u8>,
) -> Result<ExtensionPreview, ExtensionError> {
    state
        .extension_manager
        .preview_extension_internal(file_bytes)
        .await
}

#[tauri::command]
pub async fn install_extension_with_permissions(
    app_handle: AppHandle,
    file_bytes: Vec<u8>,
    custom_permissions: EditablePermissions,
    state: State<'_, AppState>,
) -> Result<String, ExtensionError> {
    state
        .extension_manager
        .install_extension_with_permissions_internal(
            app_handle,
            file_bytes,
            custom_permissions,
            &state,
        )
        .await
}
/* #[tauri::command]
pub async fn install_extension(
    app_handle: AppHandle,
    source_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let source = PathBuf::from(&source_path);

    // Manifest laden
    let manifest_path = source.join("manifest.json");
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Manifest konnte nicht gelesen werden: {}", e))?;

    let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Manifest ist ungültig: {}", e))?;

    // Signatur verifizieren
    let content_hash = ExtensionCrypto::hash_directory(&source)?;
    ExtensionCrypto::verify_signature(&manifest.public_key, &content_hash, &manifest.signature)?;

    // Key Hash berechnen
    let key_hash = manifest.calculate_key_hash()?;
    let full_extension_id = format!("{}-{}", key_hash, manifest.id);

    // Zielverzeichnis mit Key Hash Prefix
    let extensions_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("App-Datenverzeichnis nicht gefunden: {}", e))?
        .join("extensions")
        .join(&full_extension_id) // <- z.B. "a3f5b9c2d1e8f4-haex-pass"
        .join(&manifest.version);

    // Extension-Dateien kopieren
    std::fs::create_dir_all(&extensions_dir)
        .map_err(|e| format!("Verzeichnis konnte nicht erstellt werden: {}", e))?;

    let source_to_copy = if source.join("dist").exists() {
        source.join("dist") // Kopiere aus dist/
    } else {
        source.clone() // Kopiere direkt
    };

    copy_directory(
        source_to_copy.to_string_lossy().to_string(),
        extensions_dir.to_string_lossy().to_string(),
    )?;

    // Permissions speichern
    let permissions = manifest.to_internal_permissions();
    PermissionManager::save_permissions(&state.db, &permissions)
        .await
        .map_err(|e| format!("Fehler beim Speichern der Permissions: {:?}", e))?;

    // Extension registrieren
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
        .add_production_extension(extension)
        .map_err(|e| format!("Extension konnte nicht hinzugefügt werden: {:?}", e))?;

    Ok(full_extension_id)
}
 */
#[tauri::command]
pub async fn remove_extension(
    app_handle: AppHandle,
    public_key: String,
    name: String,
    version: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    state
        .extension_manager
        .remove_extension_internal(
            &app_handle,
            &public_key,
            &name,
            &version,
            &state,
        )
        .await
}

#[tauri::command]
pub fn is_extension_installed(
    public_key: String,
    name: String,
    extension_version: String,
    state: State<'_, AppState>,
) -> Result<bool, ExtensionError> {
    if let Some(ext) = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
    {
        Ok(ext.manifest.version == extension_version)
    } else {
        Ok(false)
    }
}

#[derive(serde::Deserialize, Debug)]
struct HaextensionConfig {
    dev: DevConfig,
}

#[derive(serde::Deserialize, Debug)]
struct DevConfig {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_host")]
    host: String,
}

fn default_port() -> u16 {
    5173
}

fn default_host() -> String {
    "localhost".to_string()
}

/// Check if a dev server is reachable by making a simple HTTP request
async fn check_dev_server_health(url: &str) -> bool {
    use tauri_plugin_http::reqwest;
    use std::time::Duration;

    // Try to connect with a short timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build();

    if let Ok(client) = client {
        // Just check if the root responds (most dev servers respond to / with their app)
        if let Ok(response) = client.get(url).send().await {
            // Accept any response (200, 404, etc.) - we just want to know the server is running
            return response.status().as_u16() < 500;
        }
    }

    false
}

#[tauri::command]
pub async fn load_dev_extension(
    extension_path: String,
    state: State<'_, AppState>,
) -> Result<String, ExtensionError> {
    use crate::extension::core::{
        manifest::ExtensionManifest,
        types::{Extension, ExtensionSource},
    };
    use std::path::PathBuf;
    use std::time::SystemTime;

    let extension_path_buf = PathBuf::from(&extension_path);

    // 1. Read haextension.json to get dev server config
    let config_path = extension_path_buf.join("haextension.json");
    let (host, port) = if config_path.exists() {
        let config_content = std::fs::read_to_string(&config_path).map_err(|e| {
            ExtensionError::ValidationError {
                reason: format!("Failed to read haextension.json: {}", e),
            }
        })?;

        let config: HaextensionConfig = serde_json::from_str(&config_content).map_err(|e| {
            ExtensionError::ValidationError {
                reason: format!("Failed to parse haextension.json: {}", e),
            }
        })?;

        (config.dev.host, config.dev.port)
    } else {
        // Default values if config doesn't exist
        (default_host(), default_port())
    };

    let dev_server_url = format!("http://{}:{}", host, port);
    eprintln!("📡 Dev server URL: {}", dev_server_url);

    // 1.5. Check if dev server is running
    if !check_dev_server_health(&dev_server_url).await {
        return Err(ExtensionError::ValidationError {
            reason: format!(
                "Dev server at {} is not reachable. Please start your dev server first (e.g., 'npm run dev')",
                dev_server_url
            ),
        });
    }
    eprintln!("✅ Dev server is reachable");

    // 2. Build path to manifest: <extension_path>/haextension/manifest.json
    let manifest_path = extension_path_buf.join("haextension").join("manifest.json");

    // Check if manifest exists
    if !manifest_path.exists() {
        return Err(ExtensionError::ManifestError {
            reason: format!(
                "Manifest not found at: {}. Make sure you run 'npx @haexhub/sdk init' first.",
                manifest_path.display()
            ),
        });
    }

    // 3. Read and parse manifest
    let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        ExtensionError::ManifestError {
            reason: format!("Failed to read manifest: {}", e),
        }
    })?;

    let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)?;

    // 4. Generate a unique ID for dev extension: dev_<public_key_first_8>_<name>
    let key_prefix = manifest
        .public_key
        .chars()
        .take(8)
        .collect::<String>();
    let extension_id = format!("dev_{}_{}", key_prefix, manifest.name);

    // 5. Check if dev extension already exists (allow reload)
    if let Some(existing) = state
        .extension_manager
        .get_extension_by_public_key_and_name(&manifest.public_key, &manifest.name)?
    {
        // If it's already a dev extension, remove it first (to allow reload)
        if let ExtensionSource::Development { .. } = &existing.source {
            state
                .extension_manager
                .remove_extension(&manifest.public_key, &manifest.name)?;
        }
        // Note: Production extensions can coexist with dev extensions
        // Dev extensions have priority during lookup
    }

    // 6. Create dev extension
    let extension = Extension {
        id: extension_id.clone(),
        source: ExtensionSource::Development {
            dev_server_url: dev_server_url.clone(),
            manifest_path: manifest_path.clone(),
            auto_reload: true,
        },
        manifest: manifest.clone(),
        enabled: true,
        last_accessed: SystemTime::now(),
    };

    // 7. Add to dev extensions (no database entry for dev extensions)
    state.extension_manager.add_dev_extension(extension)?;

    eprintln!(
        "✅ Dev extension loaded: {} v{} ({})",
        manifest.name, manifest.version, dev_server_url
    );

    Ok(extension_id)
}

#[tauri::command]
pub fn remove_dev_extension(
    public_key: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    // Only remove from dev_extensions, not production_extensions
    let mut dev_exts = state
        .extension_manager
        .dev_extensions
        .lock()
        .map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

    // Find and remove by public_key and name
    let to_remove = dev_exts
        .iter()
        .find(|(_, ext)| ext.manifest.public_key == public_key && ext.manifest.name == name)
        .map(|(id, _)| id.clone());

    if let Some(id) = to_remove {
        dev_exts.remove(&id);
        eprintln!("✅ Dev extension removed: {}", name);
        Ok(())
    } else {
        Err(ExtensionError::NotFound {
            public_key,
            name,
        })
    }
}

#[tauri::command]
pub fn get_all_dev_extensions(
    state: State<'_, AppState>,
) -> Result<Vec<ExtensionInfoResponse>, ExtensionError> {
    let dev_exts = state.extension_manager.dev_extensions.lock().map_err(|e| {
        ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        }
    })?;

    let mut extensions = Vec::new();
    for ext in dev_exts.values() {
        extensions.push(ExtensionInfoResponse::from_extension(ext)?);
    }

    Ok(extensions)
}
