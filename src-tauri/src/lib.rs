mod browser;
mod database;
mod extension;
mod models;

use database::DbConnection;
//use models::ExtensionState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(DbConnection(Mutex::new(None)))
        //.manage(ExtensionState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            database::create_encrypted_database,
            database::open_encrypted_database,
            database::sql_execute,
            database::sql_select,
            extension::database::extension_sql_execute,
            extension::database::extension_sql_select,
            browser::create_tab
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
