// database/mod.rs
pub mod core;

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, State};

pub struct DbConnection(pub Mutex<Option<Connection>>);

// Öffentliche Funktionen für direkten Datenbankzugriff
#[tauri::command]
pub async fn sql_select(
    sql: String,
    params: Vec<String>,
    state: State<'_, DbConnection>,
) -> Result<Vec<Vec<String>>, String> {
    core::select(&sql, &params, &state).await
}

#[tauri::command]
pub async fn sql_execute(
    sql: String,
    params: Vec<String>,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    core::execute(&sql, &params, &state).await
}

/// Erstellt eine verschlüsselte Datenbank
#[tauri::command]
pub fn create_encrypted_database(
    app_handle: AppHandle,
    path: String,
    key: String,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    // Ressourcenpfad zur eingebundenen Datenbank auflösen

    let resource_path = app_handle
        .path()
        .resolve("resources/vault.db", BaseDirectory::Resource)
        .map_err(|e| format!("Fehler beim Auflösen des Ressourcenpfads: {}", e))?;

    // Prüfen, ob die Ressourcendatei existiert
    if !resource_path.exists() {
        return Err(format!(
            "Ressourcendatenbank wurde nicht gefunden: {}",
            resource_path.display()
        ));
    }

    // Sicherstellen, dass das Zielverzeichnis existiert
    if let Some(parent) = Path::new(&path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Fehler beim Erstellen des Zielverzeichnisses: {}", e))?;
        }
    }

    // Kopieren der Ressourcen-Datenbank zum Zielpfad
    core::copy_file(&resource_path, &path)?;

    // Öffnen der kopierten Datenbank ohne Verschlüsselung
    let conn = Connection::open(&path).map_err(|e| {
        format!(
            "Fehler beim Öffnen der kopierten Datenbank: {}",
            e.to_string()
        )
    })?;

    // Verschlüsseln der Datenbank mit dem angegebenen Schlüssel
    conn.pragma_update(None, "key", &key)
        .map_err(|e| format!("Fehler beim Verschlüsseln der Datenbank: {}", e.to_string()))?;

    // Schließen der Verbindung, um sicherzustellen, dass Änderungen gespeichert werden
    drop(conn);

    // Öffnen der verschlüsselten Datenbank mit dem Schlüssel
    let encrypted_conn = core::open_and_init_db(&path, &key, false)
        .map_err(|e| format!("Fehler beim Öffnen der verschlüsselten Datenbank: {}", e))?;

    // Überprüfen, ob die Datenbank korrekt verschlüsselt wurde, indem wir eine einfache Abfrage ausführen
    let validation_result: Result<i32, _> =
        encrypted_conn.query_row("SELECT 1", [], |row| row.get(0));

    if let Err(e) = validation_result {
        return Err(format!(
            "Fehler beim Testen der verschlüsselten Datenbank: {}",
            e.to_string()
        ));
    }
    // Aktualisieren der Datenbankverbindung im State
    let mut db = state
        .0
        .lock()
        .map_err(|e| format!("Mutex-Fehler: {}", e.to_string()))?;
    *db = Some(encrypted_conn);

    Ok(format!(
        "Verschlüsselte CRDT-Datenbank erstellt unter: {} (kopiert aus Ressource)",
        path
    ))
}

/// Öffnet eine verschlüsselte Datenbank
#[tauri::command]
pub fn open_encrypted_database(
    path: String,
    key: String,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    if !std::path::Path::new(&path).exists() {
        return Err("Datenbankdatei nicht gefunden".into());
    }

    let conn = core::open_and_init_db(&path, &key, false)?;
    let mut db = state.0.lock().map_err(|e| e.to_string())?;
    *db = Some(conn);

    Ok(format!("Verschlüsselte CRDT-Datenbank geöffnet: {}", path))
}
