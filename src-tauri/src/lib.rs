//mod browser;
mod database;
mod extension;
mod models;

use database::DbConnection;
use models::ExtensionState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let protocol_name = "haex-extension";

    tauri::Builder::default()
        .setup(|app| {
            // --- START DER ASSET-INVENTUR (Korrigierte Version) ---
            println!("\n[INVENTUR] App-Setup wird ausgeführt. Liste alle Assets im Bundle auf...");

            let mut found_assets_count = 0;

            // KORREKTE METHODE: Direkt über eine Referenz auf den AssetResolver iterieren
            for asset_name in app.asset_resolver().iter() {
                found_assets_count += 1;
                // Wir geben jeden gefundenen Asset-Namen aus
                println!("[INVENTUR] Gefundenes Asset: '{}'", asset_name.0);
            }

            if found_assets_count == 0 {
                println!("[INVENTUR] Es wurden KEINE Assets im Bundle gefunden!");
            } else {
                println!(
                    "[INVENTUR] Inventur abgeschlossen. {} Assets gefunden.",
                    found_assets_count
                );
            }

            println!("[INVENTUR] --- ENDE DER INVENTUR ---\n");
            // --- ENDE DER ASSET-INVENTUR ---
            // --- START DES DEFINITIVEN ASSET-TESTS ---
            println!("\n[DEBUG] App-Setup wird ausgeführt. Versuche, die Datenbank zu laden...");

            // BITTE SICHERSTELLEN: Dieser String muss EXAKT dem SCHLÜSSEL (KEY)
            // in deiner tauri.conf.json entsprechen!
            let asset_to_find = "database/vault.db";

            println!(
                "[DEBUG] Suche nach Asset mit dem Alias: '{}'",
                asset_to_find
            );

            match app.asset_resolver().get(asset_to_find.to_string()) {
                Some(asset) => {
                    // ERFOLG! Das Asset wurde gefunden.
                    println!("\n✅ ✅ ✅ ERFOLG! ✅ ✅ ✅");
                    println!(
                        "[DEBUG] Asset '{}' wurde im Bundle gefunden.",
                        asset_to_find
                    );
                    println!("[DEBUG] Größe der Datenbank: {} Bytes.", asset.bytes.len());
                }
                None => {
                    // FEHLER! Das Asset wurde nicht gefunden.
                    println!("\n❌ ❌ ❌ FEHLER! ❌ ❌ ❌");
                    println!(
                        "[DEBUG] Asset '{}' wurde NICHT im Bundle gefunden.",
                        asset_to_find
                    );
                    println!("[DEBUG] Mögliche Ursachen:");
                    println!("[DEBUG] 1. Der Alias-String im Code ist falsch (Tippfehler?).");
                    println!("[DEBUG] 2. Der Schlüssel in 'tauri.conf.json' ist anders.");
                    println!(
                        "[DEBUG] 3. Der Build-Cache ist veraltet (lösche 'src-tauri/target')."
                    );
                }
            }
            println!("[DEBUG] --- ENDE DES ASSET-TESTS ---\n");
            // --- ENDE DES DEFINITIVEN ASSET-TESTS ---

            // Hier kann dein restlicher Setup-Code stehen bleiben
            Ok(())
        })
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
        .manage(DbConnection(Mutex::new(None)))
        .manage(ExtensionState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
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
