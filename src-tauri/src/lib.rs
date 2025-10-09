mod crdt;
mod database;
mod extension;
use crate::{crdt::hlc::HlcService, database::DbConnection, extension::core::ExtensionManager};
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub mod table_names {
    include!(concat!(env!("OUT_DIR"), "/tableNames.rs"));
}

pub struct AppState {
    pub db: DbConnection,
    pub hlc: Mutex<HlcService>,
    pub extension_manager: ExtensionManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use extension::core::EXTENSION_PROTOCOL_NAME;

    tauri::Builder::default()
        .register_uri_scheme_protocol(EXTENSION_PROTOCOL_NAME, move |context, request| {
            // Hole den AppState aus dem Context
            let app_handle = context.app_handle();
            let state = app_handle.state::<AppState>();

            // Rufe den Handler mit allen benötigten Parametern auf
            match extension::core::extension_protocol_handler(state, &app_handle, &request) {
                Ok(response) => response,
                Err(e) => {
                    eprintln!(
                        "Fehler im Custom Protocol Handler für URI '{}': {}",
                        request.uri(),
                        e
                    );
                    tauri::http::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/plain")
                        .body(Vec::from(format!(
                            "Interner Serverfehler im Protokollhandler: {}",
                            e
                        )))
                        .unwrap_or_else(|build_err| {
                            eprintln!("Konnte Fehler-Response nicht erstellen: {}", build_err);
                            tauri::http::Response::builder()
                                .status(500)
                                .body(Vec::new())
                                .expect("Konnte minimale Fallback-Response nicht erstellen")
                        })
                }
            }
        })
        .manage(AppState {
            db: DbConnection(Arc::new(Mutex::new(None))),
            hlc: Mutex::new(HlcService::new()),
            extension_manager: ExtensionManager::new(),
        })
        //.manage(ExtensionState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            database::create_encrypted_database,
            database::delete_vault,
            database::list_vaults,
            database::open_encrypted_database,
            database::sql_execute,
            database::sql_select,
            database::vault_exists,
            extension::database::extension_sql_execute,
            extension::database::extension_sql_select,
            extension::get_all_extensions,
            extension::get_extension_info,
            extension::install_extension_with_permissions,
            extension::is_extension_installed,
            extension::preview_extension,
            extension::remove_extension,
            extension::remove_extension_by_full_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
