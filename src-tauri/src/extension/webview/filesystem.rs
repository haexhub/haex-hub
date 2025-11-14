use crate::extension::error::ExtensionError;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Clone, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveFileResult {
    pub path: String,
    pub success: bool,
}

#[tauri::command]
pub async fn webview_extension_fs_save_file(
    window: WebviewWindow,
    _state: State<'_, AppState>,
    data: Vec<u8>,
    default_path: Option<String>,
    title: Option<String>,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<SaveFileResult>, ExtensionError> {
    eprintln!("[Filesystem] save_file called with {} bytes", data.len());
    eprintln!("[Filesystem] save_file default_path: {:?}", default_path);
    eprintln!("[Filesystem] save_file first 10 bytes: {:?}", &data[..data.len().min(10)]);

    // Build save dialog
    let mut dialog = window.dialog().file();

    if let Some(path) = default_path {
        dialog = dialog.set_file_name(&path);
    }

    if let Some(t) = title {
        dialog = dialog.set_title(&t);
    }

    if let Some(f) = filters {
        for filter in f {
            let ext_refs: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &ext_refs);
        }
    }

    // Show dialog (blocking_save_file is safe in async commands)
    eprintln!("[Filesystem] Showing save dialog...");
    let file_path = dialog.blocking_save_file();

    if let Some(file_path) = file_path {
        // Convert FilePath to PathBuf
        let path_buf = file_path.as_path().ok_or_else(|| ExtensionError::ValidationError {
            reason: "Failed to get file path".to_string(),
        })?;

        eprintln!("[Filesystem] User selected path: {}", path_buf.display());
        eprintln!("[Filesystem] Writing {} bytes to file...", data.len());

        // Write file using std::fs
        std::fs::write(path_buf, &data)
            .map_err(|e| {
                eprintln!("[Filesystem] ERROR writing file: {}", e);
                ExtensionError::ValidationError {
                    reason: format!("Failed to write file: {}", e),
                }
            })?;

        eprintln!("[Filesystem] File written successfully");

        Ok(Some(SaveFileResult {
            path: path_buf.to_string_lossy().to_string(),
            success: true,
        }))
    } else {
        eprintln!("[Filesystem] User cancelled");
        // User cancelled
        Ok(None)
    }
}

#[tauri::command]
pub async fn webview_extension_fs_open_file(
    window: WebviewWindow,
    _state: State<'_, AppState>,
    data: Vec<u8>,
    file_name: String,
) -> Result<serde_json::Value, ExtensionError> {
    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&file_name);

    // Write file to temp directory using std::fs
    std::fs::write(&temp_file_path, data)
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to write temp file: {}", e),
        })?;

    // Open file with system's default viewer
    let path_str = temp_file_path.to_string_lossy().to_string();
    window.opener().open_path(path_str, None::<String>)
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to open file: {}", e),
        })?;

    Ok(serde_json::json!({
        "success": true
    }))
}
