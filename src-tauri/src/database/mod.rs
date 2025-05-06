// database/mod.rs
pub mod core;

use rusqlite::Connection;
use serde_json::Value as JsonValue;
use std::path::Path;
use std::sync::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
pub struct DbConnection(pub Mutex<Option<Connection>>);

// Öffentliche Funktionen für direkten Datenbankzugriff
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

    //core::copy_file(&resource_path, &path)?;

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

    //let conn = Connection::open(&resource_path)?;

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
    // sqlcipher_export('Alias') kopiert Schema und Daten von 'main' zur Alias-DB
    /* conn.execute("SELECT sqlcipher_export('encrypted');", [])
    .map_err(|e| {
        format!(
            "Fehler bei SELECT sqlcipher_export('encrypted'): {}",
            e.to_string()
        )
    })?; */

    println!("Löse die verschlüsselte Datenbank vom Handle...");
    conn.execute("DETACH DATABASE encrypted;", [])
        .map_err(|e| format!("Fehler bei DETACH DATABASE: {}", e.to_string()))?;

    println!("Datenbank erfolgreich nach '{}' verschlüsselt.", &path);
    println!(
        "Die Originaldatei '{}' ist unverändert.",
        resource_path.as_path().display()
    );

    /* // Neue Datenbank erstellen
    let conn = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .map_err(|e| format!("Fehler beim Erstellen der Datenbank: {}", e.to_string()))?;

    // Datenbank mit dem angegebenen Passwort verschlüsseln
    conn.pragma_update(None, "key", &key)
        .map_err(|e| format!("Fehler beim Verschlüsseln der Datenbank: {}", e.to_string()))?;

    println!("Datenbank verschlüsselt mit key {}", &key);
    // Überprüfen, ob die Datenbank korrekt verschlüsselt wurde
    let validation_result: Result<i32, _> = conn.query_row("SELECT 1", [], |row| row.get(0));
    if let Err(e) = validation_result {
        return Err(format!(
            "Fehler beim Testen der verschlüsselten Datenbank: {}",
            e.to_string()
        ));
    } */

    // 2. VERSUCHEN, EINE SQLCIPHER-SPEZIFISCHE OPERATION AUSZUFÜHREN
    println!("Prüfe SQLCipher-Aktivität mit 'PRAGMA cipher_version;'...");
    match conn.query_row("PRAGMA cipher_version;", [], |row| {
        let version: String = row.get(0)?;
        Ok(version)
    }) {
        Ok(version) => {
            println!("SQLCipher ist aktiv! Version: {}", version);

            /* // Fahre mit normalen Operationen fort
            println!("Erstelle Tabelle 'benutzer'...");
            conn.execute(
                "CREATE TABLE benutzer (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
                [],
            )
            .map_err(|e| format!("Fehler beim Verschlüsseln der Datenbank: {}", e.to_string()))?;
            println!("Füge Benutzer 'Bob' hinzu...");
            conn.execute("INSERT INTO benutzer (name) VALUES ('Bob')", [])
                .map_err(|e| {
                    format!("Fehler beim Verschlüsseln der Datenbank: {}", e.to_string())
                })?;
            println!("Benutzer hinzugefügt."); */
        }
        Err(e) => {
            eprintln!("FEHLER: SQLCipher scheint NICHT aktiv zu sein!");
            eprintln!("Der Befehl 'PRAGMA cipher_version;' schlug fehl: {}", e);
            eprintln!("Die Datenbank wurde wahrscheinlich NICHT verschlüsselt.");
            // Optional: Hier die Verbindung schließen oder weitere Aktionen unterlassen
            // return Err(e); // Beende das Programm mit dem Fehler
        }
    }

    /* // Kopieren der Ressourcen-Datenbank zum Zielpfad
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
    */
    // Aktualisieren der Datenbankverbindung im State
    let mut db = state
        .0
        .lock()
        .map_err(|e| format!("Mutex-Fehler: {}", e.to_string()))?;
    *db = Some(conn);

    Ok(format!(
        "Verschlüsselte CRDT-Datenbank erstellt unter: {} and password",
        key
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
