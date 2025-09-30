use crate::extension::core::{ExtensionInfoResponse, ExtensionManager};
use tauri::State;
pub mod core;
pub mod database;
pub mod error;
pub mod filesystem;
pub mod permission_manager;

#[tauri::command]
pub fn get_extension_info(
    extension_id: String,
    extension_manager: State<ExtensionManager>,
) -> Result<ExtensionInfoResponse, String> {
    let extension = extension_manager
        .get_extension(&extension_id)
        .ok_or_else(|| format!("Extension nicht gefunden: {}", extension_id))?;

    Ok(ExtensionInfoResponse::from_extension(&extension))
}
