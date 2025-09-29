// src-tauri/src/database/mod.rs

pub mod core;
pub mod error;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use std::{fs, sync::Arc};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use tauri_plugin_fs::FsExt;
use thiserror::Error;
use ts_rs::TS;

use crate::crdt::hlc::HlcService;
use crate::database::error::DatabaseError;
use crate::table_names::TABLE_CRDT_CONFIGS;
use crate::AppState;
pub struct DbConnection(pub Arc<Mutex<Option<Connection>>>);

const VAULT_EXTENSION: &str = ".db";
const VAULT_DIRECTORY: &str = "vaults";

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

/// Resolves a database name to the full vault path
fn get_vault_path(app_handle: &AppHandle, vault_name: &str) -> Result<String, DatabaseError> {
    // Sicherstellen, dass der Name eine .db Endung hat
    let vault_file_name = if vault_name.ends_with(VAULT_EXTENSION) {
        vault_name.to_string()
    } else {
        format!("{}{VAULT_EXTENSION}", vault_name)
    };

    let vault_directory = get_vaults_directory(app_handle)?;

    let vault_path = app_handle
        .path()
        .resolve(
            format!("{vault_directory}/{}", vault_file_name),
            BaseDirectory::AppLocalData,
        )
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: format!(
                "Failed to resolve vault path for '{}': {}",
                vault_file_name, e
            ),
        })?;

    // Sicherstellen, dass das vaults-Verzeichnis existiert
    if let Some(parent) = vault_path.parent() {
        fs::create_dir_all(parent).map_err(|e| DatabaseError::IoError {
            path: parent.display().to_string(),
            reason: format!("Failed to create vaults directory: {}", e),
        })?;
    }

    Ok(vault_path.to_string_lossy().to_string())
}

/// Returns the vaults directory path
#[tauri::command]
pub fn get_vaults_directory(app_handle: &AppHandle) -> Result<String, DatabaseError> {
    let vaults_dir = app_handle
        .path()
        .resolve(VAULT_DIRECTORY, BaseDirectory::AppLocalData)
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: e.to_string(),
        })?;

    Ok(vaults_dir.to_string_lossy().to_string())
}

//#[serde(tag = "type", content = "details")]
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct VaultInfo {
    name: String,
    last_access: u64,
    path: String,
}

/// Lists all vault databases in the vaults directory
#[tauri::command]
pub fn list_vaults(app_handle: AppHandle) -> Result<Vec<VaultInfo>, DatabaseError> {
    let vaults_dir_str = get_vaults_directory(&app_handle)?;
    let vaults_dir = Path::new(&vaults_dir_str);

    println!("Suche vaults in {}", vaults_dir.display());

    let mut vaults: Vec<VaultInfo> = vec![];

    if !vaults_dir.exists() {
        println!("Vaults-Verzeichnis existiert nicht, gebe leere Liste zurück.");
        return Ok(vec![]);
    }

    for entry in fs::read_dir(vaults_dir).map_err(|e| DatabaseError::IoError {
        path: "vaults directory".to_string(),
        reason: e.to_string(),
    })? {
        let entry = entry.map_err(|e| DatabaseError::IoError {
            path: "vaults directory entry".to_string(),
            reason: e.to_string(),
        })?;

        println!("Suche entry {}", entry.path().to_string_lossy());
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(VAULT_EXTENSION) {
                    // Entferne .db Endung für die Rückgabe
                    println!("Vault gefunden {}", filename.to_string());

                    let metadata = fs::metadata(&path).map_err(|e| DatabaseError::IoError {
                        path: path.to_string_lossy().to_string(),
                        reason: format!("Metadaten konnten nicht gelesen werden: {}", e),
                    })?;

                    let last_access_timestamp = metadata
                        .accessed()
                        .map_err(|e| DatabaseError::IoError {
                            path: path.to_string_lossy().to_string(),
                            reason: format!("Zugriffszeit konnte nicht gelesen werden: {}", e),
                        })?
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default() // Fallback für den seltenen Fall einer Zeit vor 1970
                        .as_secs();

                    let vault_name = filename.trim_end_matches(VAULT_EXTENSION).to_string();

                    vaults.push(VaultInfo {
                        name: vault_name,
                        last_access: last_access_timestamp,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    Ok(vaults)
}

/// Checks if a vault with the given name exists
#[tauri::command]
pub fn vault_exists(app_handle: AppHandle, db_name: String) -> Result<bool, DatabaseError> {
    let vault_path = get_vault_path(&app_handle, &db_name)?;
    Ok(Path::new(&vault_path).exists())
}

/// Deletes a vault database file
#[tauri::command]
pub fn delete_vault(app_handle: AppHandle, db_name: String) -> Result<String, DatabaseError> {
    let vault_path = get_vault_path(&app_handle, &db_name)?;

    if !Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path,
            reason: "Vault does not exist".to_string(),
        });
    }

    fs::remove_file(&vault_path).map_err(|e| DatabaseError::IoError {
        path: vault_path.clone(),
        reason: format!("Failed to delete vault: {}", e),
    })?;

    Ok(format!("Vault '{}' successfully deleted", db_name))
}

#[tauri::command]
pub fn create_encrypted_database(
    app_handle: AppHandle,
    vault_name: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    println!("Creating encrypted vault with name: {}", vault_name);

    let vault_path = get_vault_path(&app_handle, &vault_name)?;
    println!("Resolved vault path: {}", vault_path);

    // Prüfen, ob bereits eine Vault mit diesem Namen existiert
    if Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path,
            reason: format!("A vault with the name '{}' already exists", vault_name),
        });
    }
    /* let resource_path = app_handle
    .path()
    .resolve("database/vault.db", BaseDirectory::Resource)
    .map_err(|e| format!("Fehler beim Auflösen des Ressourcenpfads: {}", e))?; */

    let template_path = app_handle
        .path()
        .resolve("database/vault.db", BaseDirectory::Resource)
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: format!("Failed to resolve template database: {}", e),
        })?;

    let template_content =
        app_handle
            .fs()
            .read(&template_path)
            .map_err(|e| DatabaseError::IoError {
                path: template_path.display().to_string(),
                reason: format!("Failed to read template database from resources: {}", e),
            })?;

    let temp_path = app_handle
        .path()
        .resolve("temp_vault.db", BaseDirectory::AppLocalData)
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: format!("Failed to resolve temp database: {}", e),
        })?;

    let temp_path_clone = temp_path.to_owned();
    fs::write(temp_path, template_content).map_err(|e| DatabaseError::IoError {
        path: vault_path.to_string(),
        reason: format!("Failed to write temporary template database: {}", e),
    })?;
    /* if !template_path.exists() {
        return Err(DatabaseError::IoError {
            path: template_path.display().to_string(),
            reason: "Template database not found in resources".to_string(),
        });
    } */

    println!("Öffne Temp-Datenbank direkt: {}", temp_path_clone.display());
    let conn = Connection::open(&temp_path_clone).map_err(|e| DatabaseError::ConnectionFailed {
        path: temp_path_clone.display().to_string(),
        reason: format!(
            "Fehler beim Öffnen der unverschlüsselten Quelldatenbank: {}",
            e
        ),
    })?;

    println!(
        "Hänge neue, verschlüsselte Datenbank an unter '{}'",
        &vault_path
    );
    // ATTACH DATABASE 'Dateiname' AS Alias KEY 'Passwort';
    conn.execute(
        "ATTACH DATABASE ?1 AS encrypted KEY ?2;",
        [&vault_path, &key],
    )
    .map_err(|e| DatabaseError::ExecutionError {
        sql: "ATTACH DATABASE ...".to_string(),
        reason: e.to_string(),
        table: None,
    })?;

    println!("Exportiere Daten von 'main' nach 'encrypted' ...");

    if let Err(e) = conn.query_row("SELECT sqlcipher_export('encrypted');", [], |_| Ok(())) {
        // Versuche aufzuräumen, ignoriere Fehler dabei
        let _ = conn.execute("DETACH DATABASE encrypted;", []);
        // Lösche auch die eventuell teilweise erstellte Datei
        let _ = fs::remove_file(&vault_path);
        let _ = fs::remove_file(&temp_path_clone);
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

    println!(
        "Datenbank erfolgreich nach '{}' verschlüsselt.",
        &vault_path
    );

    // SQLCipher-Verifizierung
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

    conn.close()
        .map_err(|(_, e)| DatabaseError::ConnectionFailed {
            path: template_path.display().to_string(),
            reason: format!("Fehler beim Schließen der Quelldatenbank: {}", e),
        })?;

    let _ = fs::remove_file(&temp_path_clone);

    initialize_session(&app_handle, &vault_path, &key, &state)?;

    Ok(vault_path)
}

#[tauri::command]
pub fn open_encrypted_database(
    app_handle: AppHandle,
    vault_path: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    println!("Opening encrypted database vault_path: {}", vault_path);

    // Vault-Pfad aus dem Namen ableiten
    //let vault_path = get_vault_path(&app_handle, &vault_name)?;
    println!("Resolved vault path: {}", vault_path);

    if !Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path.to_string(),
            reason: format!("Vault '{}' does not exist", vault_path),
        });
    }

    initialize_session(&app_handle, &vault_path, &key, &state)?;

    Ok(format!("Vault '{}' opened successfully", vault_path))
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
