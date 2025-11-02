// src-tauri/src/database/mod.rs

pub mod core;
pub mod error;
pub mod generated;
pub mod init;

use crate::crdt::hlc::HlcService;
use crate::database::core::execute_with_crdt;
use crate::database::error::DatabaseError;
use crate::extension::database::executor::SqlExecutor;
use crate::table_names::{TABLE_CRDT_CONFIGS, TABLE_SETTINGS};
use crate::AppState;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use std::{fs, sync::Arc};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use tauri_plugin_fs::FsExt;
#[cfg(not(target_os = "android"))]
use trash;
use ts_rs::TS;

pub struct DbConnection(pub Arc<Mutex<Option<Connection>>>);

const VAULT_EXTENSION: &str = ".db";
const VAULT_DIRECTORY: &str = "vaults";

#[tauri::command]
pub fn sql_select(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
    core::select(sql, params, &state.db)
}

#[tauri::command]
pub fn sql_execute(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
    core::execute(sql, params, &state.db)
}

#[tauri::command]
pub fn sql_select_with_crdt(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
    core::select_with_crdt(sql, params, &state.db)
}

#[tauri::command]
pub fn sql_execute_with_crdt(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
    let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
        reason: "Failed to lock HLC service".to_string(),
    })?;
    core::execute_with_crdt(sql, params, &state.db, &hlc_service)
}

#[tauri::command]
pub fn sql_query_with_crdt(
    sql: String,
    params: Vec<JsonValue>,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<JsonValue>>, DatabaseError> {
    let hlc_service = state.hlc.lock().map_err(|_| DatabaseError::MutexPoisoned {
        reason: "Failed to lock HLC service".to_string(),
    })?;

    core::with_connection(&state.db, |conn| {
        let tx = conn.transaction().map_err(DatabaseError::from)?;
        let (_modified_tables, result) =
            SqlExecutor::query_internal(&tx, &hlc_service, &sql, &params)?;
        tx.commit().map_err(DatabaseError::from)?;
        Ok(result)
    })
}

/// Resolves a database name to the full vault path
fn get_vault_path(app_handle: &AppHandle, vault_name: &str) -> Result<String, DatabaseError> {
    // Sicherstellen, dass der Name eine .db Endung hat
    let vault_file_name = if vault_name.ends_with(VAULT_EXTENSION) {
        vault_name.to_string()
    } else {
        format!("{vault_name}{VAULT_EXTENSION}")
    };

    let vault_directory = get_vaults_directory(app_handle)?;

    let vault_path = app_handle
        .path()
        .resolve(
            format!("{vault_directory}/{vault_file_name}"),
            BaseDirectory::AppLocalData,
        )
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: format!(
                "Failed to resolve vault path for '{vault_file_name}': {e}"
            ),
        })?;

    // Sicherstellen, dass das vaults-Verzeichnis existiert
    if let Some(parent) = vault_path.parent() {
        fs::create_dir_all(parent).map_err(|e| DatabaseError::IoError {
            path: parent.display().to_string(),
            reason: format!("Failed to create vaults directory: {e}"),
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
                    println!("Vault gefunden {filename}");

                    let metadata = fs::metadata(&path).map_err(|e| DatabaseError::IoError {
                        path: path.to_string_lossy().to_string(),
                        reason: format!("Metadaten konnten nicht gelesen werden: {e}"),
                    })?;

                    let last_access_timestamp = metadata
                        .accessed()
                        .map_err(|e| DatabaseError::IoError {
                            path: path.to_string_lossy().to_string(),
                            reason: format!("Zugriffszeit konnte nicht gelesen werden: {e}"),
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
pub fn vault_exists(app_handle: AppHandle, vault_name: String) -> Result<bool, DatabaseError> {
    let vault_path = get_vault_path(&app_handle, &vault_name)?;
    Ok(Path::new(&vault_path).exists())
}

/// Moves a vault database file to trash (or deletes permanently if trash is unavailable)
#[tauri::command]
pub fn move_vault_to_trash(
    app_handle: AppHandle,
    vault_name: String,
) -> Result<String, DatabaseError> {
    // On Android, trash is not available, so delete permanently
    #[cfg(target_os = "android")]
    {
        println!(
            "Android platform detected, permanently deleting vault '{}'",
            vault_name
        );
        return delete_vault(app_handle, vault_name);
    }

    // On non-Android platforms, try to use trash
    #[cfg(not(target_os = "android"))]
    {
        let vault_path = get_vault_path(&app_handle, &vault_name)?;
        let vault_shm_path = format!("{vault_path}-shm");
        let vault_wal_path = format!("{vault_path}-wal");

        if !Path::new(&vault_path).exists() {
            return Err(DatabaseError::IoError {
                path: vault_path,
                reason: "Vault does not exist".to_string(),
            });
        }

        // Try to move to trash first (works on desktop systems)
        let moved_to_trash = trash::delete(&vault_path).is_ok();

        if moved_to_trash {
            // Also try to move auxiliary files to trash (ignore errors as they might not exist)
            let _ = trash::delete(&vault_shm_path);
            let _ = trash::delete(&vault_wal_path);

            Ok(format!(
                "Vault '{vault_name}' successfully moved to trash"
            ))
        } else {
            // Fallback: Permanent deletion if trash fails
            println!(
                "Trash not available, falling back to permanent deletion for vault '{vault_name}'"
            );
            delete_vault(app_handle, vault_name)
        }
    }
}

/// Deletes a vault database file permanently (bypasses trash)
#[tauri::command]
pub fn delete_vault(app_handle: AppHandle, vault_name: String) -> Result<String, DatabaseError> {
    let vault_path = get_vault_path(&app_handle, &vault_name)?;
    let vault_shm_path = format!("{vault_path}-shm");
    let vault_wal_path = format!("{vault_path}-wal");

    if !Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path,
            reason: "Vault does not exist".to_string(),
        });
    }

    if Path::new(&vault_shm_path).exists() {
        fs::remove_file(&vault_shm_path).map_err(|e| DatabaseError::IoError {
            path: vault_shm_path.clone(),
            reason: format!("Failed to delete vault: {e}"),
        })?;
    }

    if Path::new(&vault_wal_path).exists() {
        fs::remove_file(&vault_wal_path).map_err(|e| DatabaseError::IoError {
            path: vault_wal_path.clone(),
            reason: format!("Failed to delete vault: {e}"),
        })?;
    }

    fs::remove_file(&vault_path).map_err(|e| DatabaseError::IoError {
        path: vault_path.clone(),
        reason: format!("Failed to delete vault: {e}"),
    })?;

    Ok(format!("Vault '{vault_name}' successfully deleted"))
}

#[tauri::command]
pub fn create_encrypted_database(
    app_handle: AppHandle,
    vault_name: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    println!("Creating encrypted vault with name: {vault_name}");

    let vault_path = get_vault_path(&app_handle, &vault_name)?;
    println!("Resolved vault path: {vault_path}");

    // Prüfen, ob bereits eine Vault mit diesem Namen existiert
    if Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path,
            reason: format!("A vault with the name '{vault_name}' already exists"),
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
            reason: format!("Failed to resolve template database: {e}"),
        })?;

    let template_content =
        app_handle
            .fs()
            .read(&template_path)
            .map_err(|e| DatabaseError::IoError {
                path: template_path.display().to_string(),
                reason: format!("Failed to read template database from resources: {e}"),
            })?;

    let temp_path = app_handle
        .path()
        .resolve("temp_vault.db", BaseDirectory::AppLocalData)
        .map_err(|e| DatabaseError::PathResolutionError {
            reason: format!("Failed to resolve temp database: {e}"),
        })?;

    let temp_path_clone = temp_path.to_owned();
    fs::write(temp_path, template_content).map_err(|e| DatabaseError::IoError {
        path: vault_path.to_string(),
        reason: format!("Failed to write temporary template database: {e}"),
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
            "Fehler beim Öffnen der unverschlüsselten Quelldatenbank: {e}"
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
            reason: format!("Fehler während sqlcipher_export: {e}"),
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
            println!("SQLCipher ist aktiv! Version: {version}");
        }
        Err(e) => {
            eprintln!("FEHLER: SQLCipher scheint NICHT aktiv zu sein!");
            eprintln!("Der Befehl 'PRAGMA cipher_version;' schlug fehl: {e}");
            eprintln!("Die Datenbank wurde wahrscheinlich NICHT verschlüsselt.");
        }
    }

    conn.close()
        .map_err(|(_, e)| DatabaseError::ConnectionFailed {
            path: template_path.display().to_string(),
            reason: format!("Fehler beim Schließen der Quelldatenbank: {e}"),
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
    println!("Opening encrypted database vault_path: {vault_path}");
    println!("Resolved vault path: {vault_path}");

    if !Path::new(&vault_path).exists() {
        return Err(DatabaseError::IoError {
            path: vault_path.to_string(),
            reason: format!("Vault '{vault_path}' does not exist"),
        });
    }

    initialize_session(&app_handle, &vault_path, &key, &state)?;

    Ok(format!("Vault '{vault_path}' opened successfully"))
}

/// Opens the DB, initializes the HLC service, and stores both in the AppState.
fn initialize_session(
    app_handle: &AppHandle,
    path: &str,
    key: &str,
    state: &State<'_, AppState>,
) -> Result<(), DatabaseError> {
    // 1. Establish the raw database connection
    let mut conn = core::open_and_init_db(path, key, false)?;

    // 2. Ensure CRDT triggers are initialized (for template DB)
    let triggers_were_already_initialized = init::ensure_triggers_initialized(&mut conn)?;

    // 3. Initialize the HLC service
    let hlc_service = HlcService::try_initialize(&conn, app_handle).map_err(|e| {
        // We convert the HlcError into a DatabaseError
        DatabaseError::ExecutionError {
            sql: "HLC Initialization".to_string(),
            reason: e.to_string(),
            table: Some(TABLE_CRDT_CONFIGS.to_string()),
        }
    })?;

    // 4. Store everything in the global AppState
    let mut db_guard = state.db.0.lock().map_err(|e| DatabaseError::LockError {
        reason: e.to_string(),
    })?;
    // Wichtig: Wir brauchen den db_guard gleich nicht mehr,
    // da 'execute_with_crdt' 'with_connection' aufruft, was
    // 'state.db' selbst locken muss.
    // Wir müssen den Guard freigeben, *bevor* wir 'execute_with_crdt' rufen,
    // um einen Deadlock zu verhindern.
    // Aber wir müssen die 'conn' erst hineinbewegen.
    *db_guard = Some(conn);
    drop(db_guard);

    let mut hlc_guard = state.hlc.lock().map_err(|e| DatabaseError::LockError {
        reason: e.to_string(),
    })?;
    *hlc_guard = hlc_service;

    // WICHTIG: hlc_guard *nicht* freigeben, da 'execute_with_crdt'
    // eine Referenz auf die Guard erwartet.

    // 5. NEUER SCHRITT: Setze das Flag via CRDT, falls nötig
    if !triggers_were_already_initialized {
        eprintln!("INFO: Setting 'triggers_initialized' flag via CRDT...");

        let insert_sql = format!(
            "INSERT INTO {TABLE_SETTINGS} (id, key, type, value) VALUES (?, ?, ?, ?)"
        );

        // execute_with_crdt erwartet Vec<JsonValue>, kein params!-Makro
        let params_vec: Vec<JsonValue> = vec![
            JsonValue::String(uuid::Uuid::new_v4().to_string()),
            JsonValue::String("triggers_initialized".to_string()),
            JsonValue::String("system".to_string()),
            JsonValue::String("1".to_string()),
        ];

        // Jetzt können wir 'execute_with_crdt' sicher aufrufen,
        // da der AppState initialisiert ist.
        execute_with_crdt(
            insert_sql, params_vec, &state.db,  // Das &DbConnection (der Mutex)
            &hlc_guard, // Die gehaltene MutexGuard
        )?;

        eprintln!("INFO: ✓ 'triggers_initialized' flag set.");
    }

    Ok(())
}
