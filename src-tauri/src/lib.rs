//mod browser;
//mod android_storage;
mod crdt;
mod database;
mod extension;

//mod models;

pub mod table_names {
    include!(concat!(env!("OUT_DIR"), "/tableNames.rs"));
}

use std::sync::{Arc, Mutex};

use crate::{crdt::hlc::HlcService, database::DbConnection, extension::core::ExtensionState};

/* use crate::{
    crdt::hlc::HlcService,
    database::{AppState, DbConnection},
    extension::core::ExtensionState,
}; */

pub struct AppState {
    pub db: DbConnection,
    pub hlc: Mutex<HlcService>, // Kein Arc hier nötig, da der ganze AppState von Tauri in einem Arc verwaltet wird.
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let protocol_name = "haex-extension";

    tauri::Builder::default()
        .register_uri_scheme_protocol(protocol_name, move |context, request| {
            match extension::core::extension_protocol_handler(&context, &request) {
                Ok(response) => response, // Wenn der Handler Ok ist, gib die Response direkt zurück
                Err(e) => {
                    // Wenn der Handler einen Fehler zurückgibt, logge ihn und erstelle eine Fehler-Response
                    eprintln!(
                        "Fehler im Custom Protocol Handler für URI '{}': {}",
                        request.uri(),
                        e
                    );
                    // Erstelle eine HTTP 500 Fehler-Response
                    // Du kannst hier auch spezifischere Fehler-Responses bauen, falls gewünscht.
                    tauri::http::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/plain") // Optional, aber gut für Klarheit
                        .body(Vec::from(format!(
                            "Interner Serverfehler im Protokollhandler: {}",
                            e
                        )))
                        .unwrap_or_else(|build_err| {
                            // Fallback, falls selbst das Erstellen der Fehler-Response fehlschlägt
                            eprintln!("Konnte Fehler-Response nicht erstellen: {}", build_err);
                            tauri::http::Response::builder()
                                .status(500)
                                .body(Vec::new())
                                .expect("Konnte minimale Fallback-Response nicht erstellen")
                        })
                }
            }
        })
        /* .manage(database::DbConnection(Arc::new(Mutex::new(None))))
        .manage(crdt::hlc::HlcService::new()) */
        .manage(AppState {
            db: DbConnection(Arc::new(Mutex::new(None))),
            hlc: Mutex::new(HlcService::new()), // Starte mit einem uninitialisierten HLC
        })
        .manage(ExtensionState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        //.plugin(tauri_plugin_android_fs::init())
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
            //database::update_hlc_from_remote,
            /* extension::copy_directory,
            extension::database::extension_sql_select, */
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/* fn extension_protocol_handler(
    app_handle: &tauri::AppHandle, // Beachten Sie die Signaturänderung in neueren Tauri-Versionen
    request: &tauri::http::Request<Vec<u8>>,
) -> Result<tauri::http::Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    let uri_str = request.uri().to_string();
let parsed_url = match Url::parse(&uri_str) {
    Ok(url) => url,
    Err(e) => {
        eprintln!("Fehler beim Parsen der URL '{}': {}", uri_str, e);
        return Ok(tauri::http::ResponseBuilder::new().status(400).body(Vec::from("Ungültige URL"))?);
    }
};

let plugin_id = parsed_url.host_str().ok_or_else(|| "Fehlende Plugin-ID in der URL".to_string())?;
let path_segments: Vec<&str> = parsed_url.path_segments().ok_or_else(|| "URL hat keinen Pfad".to_string())?.collect();

if path_segments.len() < 2 {
    eprintln!("Unvollständiger Pfad in URL: {}", uri_str);
    return Ok(tauri::http::Response::new().status(400).body(Vec::from("Unvollständiger Pfad"))?);
}

let version = path_segments;
let file_path = path_segments[1..].join("/");
    Ok(tauri::http::Response::builder()::new().status(404).body(Vec::new())?)
} */
