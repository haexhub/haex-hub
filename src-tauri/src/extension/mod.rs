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
    extension_id: String,
    state: State<AppState>,
) -> Result<ExtensionInfoResponse, String> {
    let extension = state
        .extension_manager
        .get_extension(&extension_id)
        .ok_or_else(|| format!("Extension nicht gefunden: {}", extension_id))?;

    ExtensionInfoResponse::from_extension(&extension).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub fn get_all_extensions(state: State<AppState>) -> Result<Vec<ExtensionInfoResponse>, String> {
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
    source_path: String,
) -> Result<ExtensionPreview, ExtensionError> {
    state
        .extension_manager
        .preview_extension_internal(source_path)
        .await
}

#[tauri::command]
pub async fn install_extension_with_permissions(
    app_handle: AppHandle,
    source_path: String,
    custom_permissions: EditablePermissions,
    state: State<'_, AppState>,
) -> Result<String, ExtensionError> {
    state
        .extension_manager
        .install_extension_with_permissions_internal(
            app_handle,
            source_path,
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
    extension_id: String,
    extension_version: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    state
        .extension_manager
        .remove_extension_internal(&app_handle, extension_id, extension_version, &state)
        .await
}

#[tauri::command]
pub fn is_extension_installed(
    extension_id: String,
    extension_version: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    if let Some(ext) = state.extension_manager.get_extension(&extension_id) {
        Ok(ext.manifest.version == extension_version)
    } else {
        Ok(false)
    }
}
