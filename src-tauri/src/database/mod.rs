// src-tauri/src/database/mod.rs

pub mod core;
pub mod error;

use rusqlite::Connection;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::{fs, sync::Arc};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};

use crate::crdt::hlc::HlcService;
use crate::database::error::DatabaseError;
use crate::table_names::TABLE_CRDT_CONFIGS;
use crate::AppState;
pub struct DbConnection(pub Arc<Mutex<Option<Connection>>>);

#[tauri::command]
pub fn sql_select(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<HashMap<String, JsonValue>>, DatabaseError> {
    core::select(sql, params, &state.db)
}

#[tauri::command]
pub fn sql_execute(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<usize, DatabaseError> {
    core::execute(sql, params, &state.db)
}

#[tauri::command]
pub fn create_encrypted_database(
    app_handle: AppHandle,
    path: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
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
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: e.to_string(),
        })?;

    // Prüfen, ob die Ressourcendatei existiert
    if !resource_path.exists() {
        return Err(DatabaseError::IoError {
            path: resource_path.display().to_string(),
            reason: "Ressourcendatenbank wurde nicht gefunden.".to_string(),
        });
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
        fs::remove_file(target).map_err(|e| DatabaseError::IoError {
            path: target.display().to_string(),
            reason: format!("Bestehende Zieldatei konnte nicht gelöscht werden: {}", e),
        })?;
    }

    println!(
        "Öffne unverschlüsselte Datenbank: {}",
        resource_path.as_path().display()
    );

    let conn = Connection::open(&resource_path).map_err(|e| DatabaseError::ConnectionFailed {
        path: resource_path.display().to_string(),
        reason: format!(
            "Fehler beim Öffnen der unverschlüsselten Quelldatenbank: {}",
            e
        ),
    })?;

    println!("Hänge neue, verschlüsselte Datenbank an unter '{}'", &path);
    // ATTACH DATABASE 'Dateiname' AS Alias KEY 'Passwort';
    conn.execute("ATTACH DATABASE ?1 AS encrypted KEY ?2;", [&path, &key])
        .map_err(|e| DatabaseError::ExecutionError {
            sql: "ATTACH DATABASE ...".to_string(),
            reason: e.to_string(),
            table: None,
        })?;

    println!(
        "Exportiere Daten von 'main' nach 'encrypted' mit password {} ...",
        &key
    );

    if let Err(e) = conn.query_row("SELECT sqlcipher_export('encrypted');", [], |_| Ok(())) {
        // Versuche aufzuräumen, ignoriere Fehler dabei
        let _ = conn.execute("DETACH DATABASE encrypted;", []);
        return Err(DatabaseError::QueryError {
            reason: format!("Fehler während sqlcipher_export: {}", e),
        });
    }

    println!("Löse die verschlüsselte Datenbank vom Handle...");
    conn.execute("DETACH DATABASE encrypted;", [])
        .map_err(|e| DatabaseError::ExecutionError {
            sql: "DETACH DATABASE ...".to_string(),
            reason: e.to_string(),
            table: None,
        })?;

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

    conn.close()
        .map_err(|(_, e)| DatabaseError::ConnectionFailed {
            path: resource_path.display().to_string(),
            reason: format!("Fehler beim Schließen der Quelldatenbank: {}", e),
        })?;

    initialize_session(&app_handle, &path, &key, &state)?;

    /* let new_conn = core::open_and_init_db(&path, &key, false)?;

    // Aktualisieren der Datenbankverbindung im State
    let mut db = state.db.0.lock().map_err(|e| DatabaseError::LockError {
        reason: e.to_string(),
    })?;

    *db = Some(new_conn); */

    Ok(format!("Verschlüsselte CRDT-Datenbank erstellt",))
}

#[tauri::command]
pub fn open_encrypted_database(
    app_handle: AppHandle,
    path: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    if !Path::new(&path).exists() {
        return Err(DatabaseError::IoError {
            path,
            reason: "Database file not found.".to_string(),
        });
    }
    /* let vault_path = app_handle
    .path()
    .resolve(format!("vaults/{}", path), BaseDirectory::AppLocalData)
    .map_err(|e| format!("Fehler {}", e))?
    .into_os_string()
    .into_string()
    .unwrap(); */
    /* if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found {}", path).into());
    } */

    /* let conn = core::open_and_init_db(&path, &key, false)
        .map_err(|e| format!("Error during open: {}", e))?;

    let mut db = state.db.0.lock().map_err(|e| e.to_string())?;

    *db = Some(conn); */

    initialize_session(&app_handle, &path, &key, &state)?;

    Ok(format!("success"))
}

/// Opens the DB, initializes the HLC service, and stores both in the AppState.
fn initialize_session(
    app_handle: &AppHandle,
    path: &str,
    key: &str,
    state: &State<'_, AppState>,
) -> Result<(), DatabaseError> {
    // 1. Establish the raw database connection
    let conn = core::open_and_init_db(path, key, false)?;

    // 2. Initialize the HLC service
    let hlc_service = HlcService::try_initialize(&conn, app_handle).map_err(|e| {
        // We convert the HlcError into a DatabaseError
        DatabaseError::ExecutionError {
            sql: "HLC Initialization".to_string(),
            reason: e.to_string(),
            table: Some(TABLE_CRDT_CONFIGS.to_string()),
        }
    })?;

    // 3. Store everything in the global AppState
    let mut db_guard = state.db.0.lock().map_err(|e| DatabaseError::LockError {
        reason: e.to_string(),
    })?;
    *db_guard = Some(conn);

    let mut hlc_guard = state.hlc.lock().map_err(|e| DatabaseError::LockError {
        reason: e.to_string(),
    })?;
    *hlc_guard = hlc_service;

    Ok(())
}
