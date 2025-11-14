use crate::event_names::EVENT_EXTENSION_WINDOW_CLOSED;
use crate::extension::error::ExtensionError;
use crate::extension::ExtensionManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Verwaltet native WebviewWindows für Extensions (nur Desktop-Plattformen)
pub struct ExtensionWebviewManager {
    /// Map: window_id -> extension_id
    /// Das window_id ist ein eindeutiger Identifier (Tauri-kompatibel, keine Bindestriche)
    /// und wird gleichzeitig als Tauri WebviewWindow label verwendet
    pub windows: Arc<Mutex<HashMap<String, String>>>,
}

impl ExtensionWebviewManager {
    pub fn new() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Öffnet eine Extension in einem nativen WebviewWindow
    ///
    /// # Arguments
    /// * `app_handle` - Tauri AppHandle
    /// * `extension_manager` - Extension Manager für Zugriff auf Extension-Daten
    /// * `extension_id` - ID der zu öffnenden Extension
    /// * `title` - Fenstertitel
    /// * `width` - Fensterbreite
    /// * `height` - Fensterhöhe
    /// * `x` - X-Position (optional)
    /// * `y` - Y-Position (optional)
    ///
    /// # Returns
    /// Das window_id des erstellten Fensters
    pub fn open_extension_window(
        &self,
        app_handle: &AppHandle,
        extension_manager: &ExtensionManager,
        extension_id: String,
        title: String,
        width: f64,
        height: f64,
        x: Option<f64>,
        y: Option<f64>,
    ) -> Result<String, ExtensionError> {
        // Extension aus Manager holen
        let extension = extension_manager
            .get_extension(&extension_id)
            .ok_or_else(|| ExtensionError::NotFound {
                public_key: "".to_string(),
                name: extension_id.clone(),
            })?;

        // URL für Extension generieren (analog zum Frontend)
        use crate::extension::core::types::ExtensionSource;
        let url = match &extension.source {
            ExtensionSource::Production { .. } => {
                // Für Production Extensions: custom protocol
                #[cfg(target_os = "android")]
                let protocol = "http";
                #[cfg(not(target_os = "android"))]
                let protocol = "haex-extension";

                // Extension Info Base64-codieren (wie im Frontend)
                let extension_info = serde_json::json!({
                    "publicKey": extension.manifest.public_key,
                    "name": extension.manifest.name,
                    "version": match &extension.source {
                        ExtensionSource::Production { version, .. } => version,
                        _ => "",
                    }
                });
                let extension_info_str = serde_json::to_string(&extension_info)
                    .map_err(|e| ExtensionError::ValidationError {
                        reason: format!("Failed to serialize extension info: {}", e),
                    })?;
                let extension_info_base64 =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, extension_info_str.as_bytes());

                #[cfg(target_os = "android")]
                let host = "haex-extension.localhost";
                #[cfg(not(target_os = "android"))]
                let host = "localhost";

                let entry = extension.manifest.entry.as_deref().unwrap_or("index.html");
                format!("{}://{}/{}/{}", protocol, host, extension_info_base64, entry)
            }
            ExtensionSource::Development { dev_server_url, .. } => {
                // Für Dev Extensions: direkt Dev-Server URL
                dev_server_url.clone()
            }
        };

        // Eindeutige Window-ID generieren (wird auch als Tauri label verwendet, keine Bindestriche erlaubt)
        let window_id = format!("ext_{}", uuid::Uuid::new_v4().simple());

        eprintln!("Opening extension window: {} with URL: {}", window_id, url);

        // WebviewWindow erstellen
        let webview_url = WebviewUrl::External(url.parse().map_err(|e| {
            ExtensionError::ValidationError {
                reason: format!("Invalid URL: {}", e),
            }
        })?);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let mut builder = WebviewWindowBuilder::new(app_handle, &window_id, webview_url)
            .title(&title)
            .inner_size(width, height)
            .decorations(true) // Native Decorations (Titlebar, etc.)
            .resizable(true)
            .skip_taskbar(false) // In Taskbar anzeigen
            .center(); // Fenster zentrieren

        #[cfg(any(target_os = "android", target_os = "ios"))]
        let mut builder = WebviewWindowBuilder::new(app_handle, &window_id, webview_url)
            .inner_size(width, height);

        // Position setzen, falls angegeben (nur Desktop)
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let (Some(x_pos), Some(y_pos)) = (x, y) {
            builder = builder.position(x_pos, y_pos);
        }

        // Fenster erstellen
        let webview_window = builder.build().map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to create webview window: {}", e),
        })?;

        // Event-Listener für das Schließen des Fensters registrieren
        let window_id_for_event = window_id.clone();
        let app_handle_for_event = app_handle.clone();
        let windows_for_event = self.windows.clone();

        webview_window.on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event {
                eprintln!("WebviewWindow destroyed: {}", window_id_for_event);

                // Registry cleanup
                if let Ok(mut windows) = windows_for_event.lock() {
                    windows.remove(&window_id_for_event);
                }

                // Emit event an Frontend, damit das Tracking aktualisiert wird
                let _ = app_handle_for_event.emit(EVENT_EXTENSION_WINDOW_CLOSED, &window_id_for_event);
            }
        });

        // In Registry speichern
        let mut windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;
        windows.insert(window_id.clone(), extension_id.clone());

        eprintln!("Extension window opened successfully: {}", window_id);
        Ok(window_id)
    }

    /// Schließt ein Extension-Fenster
    pub fn close_extension_window(
        &self,
        app_handle: &AppHandle,
        window_id: &str,
    ) -> Result<(), ExtensionError> {
        let mut windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

        if windows.remove(window_id).is_some() {
            drop(windows); // Release lock before potentially blocking operation

            // Webview Window schließen (nur Desktop)
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app_handle.get_webview_window(window_id) {
                window.close().map_err(|e| ExtensionError::ValidationError {
                    reason: format!("Failed to close window: {}", e),
                })?;
            }
            eprintln!("Extension window closed: {}", window_id);
            Ok(())
        } else {
            Err(ExtensionError::NotFound {
                public_key: "".to_string(),
                name: window_id.to_string(),
            })
        }
    }

    /// Fokussiert ein Extension-Fenster
    pub fn focus_extension_window(
        &self,
        app_handle: &AppHandle,
        window_id: &str,
    ) -> Result<(), ExtensionError> {
        let windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

        let exists = windows.contains_key(window_id);
        drop(windows); // Release lock

        if exists {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app_handle.get_webview_window(window_id) {
                window.set_focus().map_err(|e| ExtensionError::ValidationError {
                    reason: format!("Failed to focus window: {}", e),
                })?;
                // Zusätzlich nach vorne bringen
                window.set_always_on_top(true).ok();
                window.set_always_on_top(false).ok();
            }
            Ok(())
        } else {
            Err(ExtensionError::NotFound {
                public_key: "".to_string(),
                name: window_id.to_string(),
            })
        }
    }

    /// Aktualisiert Position eines Extension-Fensters
    pub fn update_extension_window_position(
        &self,
        app_handle: &AppHandle,
        window_id: &str,
        x: f64,
        y: f64,
    ) -> Result<(), ExtensionError> {
        let windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

        let exists = windows.contains_key(window_id);
        drop(windows); // Release lock

        if exists {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app_handle.get_webview_window(window_id) {
                use tauri::Position;
                window
                    .set_position(Position::Physical(tauri::PhysicalPosition {
                        x: x as i32,
                        y: y as i32,
                    }))
                    .map_err(|e| ExtensionError::ValidationError {
                        reason: format!("Failed to set window position: {}", e),
                    })?;
            }
            Ok(())
        } else {
            Err(ExtensionError::NotFound {
                public_key: "".to_string(),
                name: window_id.to_string(),
            })
        }
    }

    /// Aktualisiert Größe eines Extension-Fensters
    pub fn update_extension_window_size(
        &self,
        app_handle: &AppHandle,
        window_id: &str,
        width: f64,
        height: f64,
    ) -> Result<(), ExtensionError> {
        let windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

        let exists = windows.contains_key(window_id);
        drop(windows); // Release lock

        if exists {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app_handle.get_webview_window(window_id) {
                use tauri::Size;
                window
                    .set_size(Size::Physical(tauri::PhysicalSize {
                        width: width as u32,
                        height: height as u32,
                    }))
                    .map_err(|e| ExtensionError::ValidationError {
                        reason: format!("Failed to set window size: {}", e),
                    })?;
            }
            Ok(())
        } else {
            Err(ExtensionError::NotFound {
                public_key: "".to_string(),
                name: window_id.to_string(),
            })
        }
    }

    /// Emits an event to all extension webview windows
    pub fn emit_to_all_extensions<S: serde::Serialize + Clone>(
        &self,
        app_handle: &AppHandle,
        event: &str,
        payload: S,
    ) -> Result<(), ExtensionError> {
        let windows = self.windows.lock().map_err(|e| ExtensionError::MutexPoisoned {
            reason: e.to_string(),
        })?;

        eprintln!("[Manager] Emitting event '{}' to {} webview windows", event, windows.len());

        // Iterate over all window IDs
        for window_id in windows.keys() {
            eprintln!("[Manager] Trying to emit to window: {}", window_id);
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if let Some(window) = app_handle.get_webview_window(window_id) {
                // Emit event to this specific webview window
                match window.emit(event, payload.clone()) {
                    Ok(_) => eprintln!("[Manager] Successfully emitted event '{}' to window {}", event, window_id),
                    Err(e) => eprintln!("[Manager] Failed to emit event {} to window {}: {}", event, window_id, e),
                }
            } else {
                eprintln!("[Manager] Window not found: {}", window_id);
            }
        }

        Ok(())
    }
}

impl Default for ExtensionWebviewManager {
    fn default() -> Self {
        Self::new()
    }
}
