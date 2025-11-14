mod crdt;
mod database;
mod extension;
use crate::{
    crdt::hlc::HlcService,
    database::DbConnection,
    extension::core::ExtensionManager,
};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::extension::webview::ExtensionWebviewManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub mod table_names {
    include!(concat!(env!("OUT_DIR"), "/tableNames.rs"));
}

pub mod event_names {
    include!(concat!(env!("OUT_DIR"), "/eventNames.rs"));
}

pub struct AppState {
    pub db: DbConnection,
    pub hlc: Mutex<HlcService>,
    pub extension_manager: ExtensionManager,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub extension_webview_manager: ExtensionWebviewManager,
    pub context: Arc<Mutex<extension::webview::web::ApplicationContext>>,
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
            match extension::core::extension_protocol_handler(state, app_handle, &request) {
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
                            "Interner Serverfehler im Protokollhandler: {e}"
                        )))
                        .unwrap_or_else(|build_err| {
                            eprintln!("Konnte Fehler-Response nicht erstellen: {build_err}");
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
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension_webview_manager: ExtensionWebviewManager::new(),
            context: Arc::new(Mutex::new(extension::webview::web::ApplicationContext {
                theme: "dark".to_string(),
                locale: "en".to_string(),
                platform: std::env::consts::OS.to_string(),
            })),
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
            database::move_vault_to_trash,
            database::list_vaults,
            database::open_encrypted_database,
            database::sql_execute_with_crdt,
            database::sql_execute,
            database::sql_query_with_crdt,
            database::sql_select_with_crdt,
            database::sql_select,
            database::vault_exists,
            extension::database::extension_sql_execute,
            extension::database::extension_sql_select,
            extension::web::extension_web_fetch,
            extension::web::extension_web_open,
            extension::permissions::check::check_web_permission,
            extension::permissions::check::check_database_permission,
            extension::permissions::check::check_filesystem_permission,
            extension::get_all_dev_extensions,
            extension::get_all_extensions,
            extension::get_extension_info,
            extension::install_extension_with_permissions,
            extension::is_extension_installed,
            extension::load_dev_extension,
            extension::preview_extension,
            extension::remove_dev_extension,
            extension::remove_extension,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::open_extension_webview_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::close_extension_webview_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::focus_extension_webview_window,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::update_extension_webview_window_position,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::update_extension_webview_window_size,
            // WebView API commands (for native window extensions, desktop only)
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_get_info,
            extension::webview::web::webview_extension_context_get,
            extension::webview::web::webview_extension_context_set,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::database::webview_extension_db_query,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::database::webview_extension_db_execute,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_check_web_permission,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_check_database_permission,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_check_filesystem_permission,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_web_open,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_web_request,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::web::webview_extension_emit_to_all,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::filesystem::webview_extension_fs_save_file,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            extension::webview::filesystem::webview_extension_fs_open_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
