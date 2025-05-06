pub mod core;
pub mod database;
use tauri;

#[tauri::command]
pub async fn copy_directory(source: String, destination: String) -> Result<(), String> {
    core::copy_directory(source, destination)
}
