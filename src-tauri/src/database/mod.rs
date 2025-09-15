// database/mod.rs
pub mod core;

use rusqlite::Connection;
use serde_json::Value as JsonValue;

use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};

use crate::database::core::open_and_init_db;
pub struct HlcService(pub Mutex<uhlc::HLC>);
pub struct DbConnection(pub Arc<Mutex<Option<Connection>>>);

#[tauri::command]
pub async fn sql_select(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, DbConnection>,
) -> Result<Vec<Vec<JsonValue>>, String> {
    core::select(sql, params, &state).await
}

#[tauri::command]
pub async fn sql_execute(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, DbConnection>,
) -> Result<usize, String> {
    core::execute(sql, params, &state).await
}

#[tauri::command]
pub fn test(app_handle: AppHandle) -> Result<String, String> {
    let resource_path = app_handle
        .path()
        .resolve("database/vault.db", BaseDirectory::Resource)
        .map_err(|e| format!("Fehler {}", e));
    //let file = app_handle.fs().open(resource_path, {}).unwrap().read();
    Ok(String::from(resource_path.unwrap().to_string_lossy()))
    /* std::fs::exists(String::from(resource_path.unwrap().to_string_lossy()))
    .map_err(|e| format!("Fehler: {}", e)) */
}

#[tauri::command]
pub fn create_encrypted_database(
    app_handle: AppHandle,
    path: String,
    key: String,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    // Ressourcenpfad zur eingebundenen Datenbank auflösen

    println!("Arbeitsverzeichnis: {:?}", std::env::current_dir());
    println!(
        "Ressourcenverzeichnis: {:?}",
        app_handle.path().resource_dir()
    );

    /* let resource_path = app_handle
    .path()
    .resolve("database/vault.db", BaseDirectory::Resource)
    .map_err(|e| format!("Fehler beim Auflösen des Ressourcenpfads: {}", e))?; */

    let resource_path = app_handle
        .path()
        .resolve("temp_vault.db", BaseDirectory::AppLocalData)
        .map_err(|e| format!("Fehler beim Auflösen des Ressourcenpfads: {}", e))?;

    // Prüfen, ob die Ressourcendatei existiert
    if !resource_path.exists() {
        return Err(format!(
            "Ressourcendatenbank wurde nicht gefunden: {}",
            resource_path.display()
        ));
    }

    // Sicherstellen, dass das Zielverzeichnis existiert
    /* if let Some(parent) = Path::new(&path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Fehler beim Erstellen des Zielverzeichnisses: {}\n mit Fehler {}",
                    path, e
                )
            })?;
        }
    } */

    let target = Path::new(&path);
    if target.exists() & target.is_file() {
        println!(
            "Datei '{}' existiert bereits. Sie wird gelöscht.",
            target.display()
        );

        fs::remove_file(target)
            .map_err(|e| format!("Kann Vault {} nicht löschen. \n {}", target.display(), e))?;
    } else {
        println!("Datei '{}' existiert nicht.", target.display());
    }

    println!(
        "Öffne unverschlüsselte Datenbank: {}",
        resource_path.as_path().display()
    );

    let conn = Connection::open(&resource_path).map_err(|e| {
        format!(
            "Fehler beim Öffnen der kopierten Datenbank: {}",
            e.to_string()
        )
    })?;

    println!("Hänge neue, verschlüsselte Datenbank an unter '{}'", &path);
    // ATTACH DATABASE 'Dateiname' AS Alias KEY 'Passwort';
    conn.execute("ATTACH DATABASE ?1 AS encrypted KEY ?2;", [&path, &key])
        .map_err(|e| format!("Fehler bei ATTACH DATABASE: {}", e.to_string()))?;

    println!(
        "Exportiere Daten von 'main' nach 'encrypted' mit password {} ...",
        &key
    );

    match conn.query_row("SELECT sqlcipher_export('encrypted');", [], |_row| Ok(())) {
        Ok(_) => {
            println!(">>> sqlcipher_export erfolgreich ausgeführt (Rückgabewert ignoriert).");
        }
        Err(e) => {
            eprintln!("!!! FEHLER während sqlcipher_export: {}", e);
            conn.execute("DETACH DATABASE encrypted;", []).ok(); // Versuche zu detachen
            return Err(e.to_string()); // Gib den Fehler zurück
        }
    }

    println!("Löse die verschlüsselte Datenbank vom Handle...");
    conn.execute("DETACH DATABASE encrypted;", [])
        .map_err(|e| format!("Fehler bei DETACH DATABASE: {}", e.to_string()))?;

    println!("Datenbank erfolgreich nach '{}' verschlüsselt.", &path);
    println!(
        "Die Originaldatei '{}' ist unverändert.",
        resource_path.as_path().display()
    );

    // 2. VERSUCHEN, EINE SQLCIPHER-SPEZIFISCHE OPERATION AUSZUFÜHREN
    println!("Prüfe SQLCipher-Aktivität mit 'PRAGMA cipher_version;'...");
    match conn.query_row("PRAGMA cipher_version;", [], |row| {
        let version: String = row.get(0)?;
        Ok(version)
    }) {
        Ok(version) => {
            println!("SQLCipher ist aktiv! Version: {}", version);
        }
        Err(e) => {
            eprintln!("FEHLER: SQLCipher scheint NICHT aktiv zu sein!");
            eprintln!("Der Befehl 'PRAGMA cipher_version;' schlug fehl: {}", e);
            eprintln!("Die Datenbank wurde wahrscheinlich NICHT verschlüsselt.");
        }
    }

    println!("resource_path: {}", resource_path.display());

    conn.close().unwrap();

    let new_conn = open_and_init_db(&path, &key, false)?;

    // Aktualisieren der Datenbankverbindung im State
    let mut db = state
        .0
        .lock()
        .map_err(|e| format!("Mutex-Fehler: {}", e.to_string()))?;
    *db = Some(new_conn);

    Ok(format!("Verschlüsselte CRDT-Datenbank erstellt",))
}

#[tauri::command]
pub fn open_encrypted_database(
    app_handle: AppHandle,
    path: String,
    key: String,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    /* let vault_path = app_handle
    .path()
    .resolve(format!("vaults/{}", path), BaseDirectory::AppLocalData)
    .map_err(|e| format!("Fehler {}", e))?
    .into_os_string()
    .into_string()
    .unwrap(); */
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found {}", path).into());
    }

    let conn =
        core::open_and_init_db(&path, &key, false).map_err(|e| format!("Error during open: {}", e));

    let mut db = state.0.lock().map_err(|e| e.to_string())?;
    *db = Some(conn.unwrap());

    Ok(format!("success"))
}

fn get_target_triple() -> Result<String, String> {
    let target_triple = if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-unknown-linux-gnu".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64-unknown-linux-gnu".to_string()
        } else {
            return Err(format!(
                "Unbekannte Linux-Architektur: {}",
                std::env::consts::ARCH
            ));
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin".to_string()
        } else {
            return Err(format!(
                "Unbekannte macOS-Architektur: {}",
                std::env::consts::ARCH
            ));
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(target_arch = "x86") {
            "i686-pc-windows-msvc".to_string()
        } else {
            return Err(format!(
                "Unbekannte Windows-Architektur: {}",
                std::env::consts::ARCH
            ));
        }
    } else if cfg!(target_os = "android") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-linux-android".to_string()
        } else {
            return Err(format!(
                "Unbekannte Android-Architektur: {}",
                std::env::consts::ARCH
            ));
        }
    } else if cfg!(target_os = "ios") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-ios".to_string()
        } else {
            return Err(format!(
                "Unbekannte iOS-Architektur: {}",
                std::env::consts::ARCH
            ));
        }
    } else {
        return Err("Unbekanntes Zielsystem".to_string());
    };
    Ok(target_triple)
}

pub fn get_hlc_timestamp(state: tauri::State<HlcService>) -> String {
    let hlc = state.0.lock().unwrap();
    hlc.new_timestamp().to_string()
}

#[tauri::command]
pub fn update_hlc_from_remote(
    remote_timestamp_str: String,
    state: tauri::State<HlcService>,
) -> Result<(), String> {
    let remote_ts =
        uhlc::Timestamp::from_str(&remote_timestamp_str).map_err(|e| e.cause.to_string())?;

    let hlc = state.0.lock().unwrap();
    hlc.update_with_timestamp(&remote_ts)
        .map_err(|e| format!("HLC update failed: {:?}", e))
}

#[tauri::command]
pub async fn create_crdt_trigger_for_table(
    state: &State<'_, DbConnection>,
    table_name: String,
) -> Result<Vec<Vec<JsonValue>>, String> {
    let stmt = format!(
        "SELECT cid, name, type, notnull, dflt_value, pk from pragma_table_info('{}')",
        table_name
    );

    let table_info = core::select(stmt, vec![], state).await;
    Ok(table_info.unwrap())
}
