//mod browser;
mod database;
mod extension;
mod models;

use database::DbConnection;
use models::ExtensionState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //let protocol_name = "haex-extension";

    tauri::Builder::default()
        /* .register_uri_scheme_protocol(protocol_name, move |app_handle, request| {
            // Extrahiere den Request aus dem Kontext
            //let request = context.request();
            // Rufe die Handler-Logik auf
            match extension::core::handle_extension_protocol(0, &request) {
                Ok(response) => response, // Gib die erfolgreiche Response zurück
                Err(e) => {
                    // Logge den Fehler
                    eprintln!("Fehler im Protokoll-Handler für '{}': {}", request.uri(), e);
                    // Gib eine generische 500er Fehler-Response zurück
                    Response::builder()
                    .status(500)
                    .mimetype("text/plain") // Einfacher Text für die Fehlermeldung
                    .body(format!("Internal Server Error: {}", e).into_bytes()) // Body als Vec<u8>
                    .unwrap() // .body() kann hier nicht fehlschlagen
                }
            }
        }) */
        /* .setup(move |app| {
            // Der .setup Hook ist jetzt nur noch für andere Initialisierungen da
            // Der AppHandle ist hier nicht mehr nötig für die Protokoll-Registrierung
            println!("App Setup abgeschlossen.");
            Ok(())
        }) */
        .plugin(tauri_plugin_http::init())
        .manage(DbConnection(Mutex::new(None)))
        .manage(ExtensionState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        //.plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            database::create_encrypted_database,
            database::open_encrypted_database,
            database::sql_execute,
            database::sql_select,
            extension::database::extension_sql_execute,
            extension::database::extension_sql_select,
            extension::copy_directory //browser::create_tab
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
