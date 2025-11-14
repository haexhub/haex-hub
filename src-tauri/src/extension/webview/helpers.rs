use crate::extension::core::protocol::ExtensionInfo;
use crate::extension::error::ExtensionError;
use crate::AppState;
use tauri::{State, WebviewWindow};

/// Get extension_id from window (SECURITY: window_id from Tauri, cannot be spoofed)
pub fn get_extension_id(window: &WebviewWindow, state: &State<AppState>) -> Result<String, ExtensionError> {
    let window_id = window.label();
    eprintln!("[webview_api] Looking up extension_id for window: {}", window_id);

    let windows = state
        .extension_webview_manager
        .windows
        .lock()
        .map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

    eprintln!("[webview_api] HashMap contents: {:?}", *windows);

    let extension_id = windows
        .get(window_id)
        .cloned()
        .ok_or_else(|| ExtensionError::ValidationError {
            reason: format!("Window {} is not registered as an extension window", window_id),
        })?;

    eprintln!("[webview_api] Found extension_id: {}", extension_id);
    Ok(extension_id)
}

/// Get full extension info (public_key, name, version) from window
pub fn get_extension_info_from_window(
    window: &WebviewWindow,
    state: &State<AppState>,
) -> Result<ExtensionInfo, ExtensionError> {
    let extension_id = get_extension_id(window, state)?;

    // Get extension from ExtensionManager using the database UUID
    let extension = state
        .extension_manager
        .get_extension(&extension_id)
        .ok_or_else(|| ExtensionError::ValidationError {
            reason: format!("Extension with ID {} not found", extension_id),
        })?;

    let version = match &extension.source {
        crate::extension::core::types::ExtensionSource::Production { version, .. } => version.clone(),
        crate::extension::core::types::ExtensionSource::Development { .. } => "dev".to_string(),
    };

    Ok(ExtensionInfo {
        public_key: extension.manifest.public_key,
        name: extension.manifest.name,
        version,
    })
}
