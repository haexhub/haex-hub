// database/mod.rs
pub mod core;

use rusqlite::Connection;
use serde_json::Value as JsonValue;

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{path::BaseDirectory, AppHandle, Manager, State, Wry};

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

use tauri_plugin_dialog::{Dialog, DialogExt, MessageDialogKind};
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

fn prepare_temporary_asset_db(
    app_handle: &AppHandle<Wry>,
    asset_name: &str,
    temp_base_dir: BaseDirectory,
) -> Result<PathBuf, String> {
    println!("Lade Asset '{}' aus dem App-Bundle...", asset_name);

    //.resolve("vault.db", BaseDirectory::Resource)
    let asset_bytes = app_handle
        .asset_resolver()
        .get(asset_name.to_owned())
        .ok_or_else(|| format!("Asset '{}' wurde nicht im Bundle gefunden.", asset_name))?
        .bytes()
        .to_vec();

    println!(
        "Asset '{}' erfolgreich geladen ({} bytes).",
        asset_name,
        asset_bytes.len()
    );

    let temp_db_filename = format!("temp_unencrypted_{}", asset_name);
    let temp_db_path = app_handle
        .path()
        .resolve(&temp_db_filename, temp_base_dir)
        .map_err(|e| {
            format!(
                "Fehler beim Auflösen des Pfads für die temporäre DB '{}': {}",
                temp_db_filename, e
            )
        })?;

    println!(
        "Temporärer Pfad für unverschlüsselte DB: {}",
        temp_db_path.display()
    );

    if let Some(parent) = temp_db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Fehler beim Erstellen des temporären Verzeichnisses '{}': {}",
                    parent.display(),
                    e
                )
            })?;
            println!("Temporäres Verzeichnis '{}' erstellt.", parent.display());
        }
    }

    if temp_db_path.exists() {
        fs::remove_file(&temp_db_path).map_err(|e| {
            format!(
                "Fehler beim Löschen der alten temporären DB '{}': {}",
                temp_db_path.display(),
                e
            )
        })?;
        println!("Alte temporäre DB '{}' gelöscht.", temp_db_path.display());
    }

    fs::write(&temp_db_path, &asset_bytes).map_err(|e| {
        format!(
            "Fehler beim Schreiben der Asset-DB nach '{}': {}",
            temp_db_path.display(),
            e
        )
    })?;
    println!(
        "Asset-DB erfolgreich nach '{}' geschrieben.",
        temp_db_path.display()
    );

    Ok(temp_db_path)
}

/// Hilfsfunktion: Verschlüsselt eine Quelldatenbank in eine Zieldatenbank.
fn encrypt_database_from_source(
    unencrypted_source_path: &Path,
    target_encrypted_path_str: &str,
    key: &str,
) -> Result<(), String> {
    println!(
        "Öffne temporäre Quelldatenbank '{}'...",
        unencrypted_source_path.display()
    );
    let source_conn = Connection::open(unencrypted_source_path).map_err(|e| {
        format!(
            "Fehler beim Öffnen der Quelldatenbank '{}': {}",
            unencrypted_source_path.display(),
            e
        )
    })?;
    println!(
        "Verbindung zur Quelldatenbank '{}' geöffnet.",
        unencrypted_source_path.display()
    );

    let final_encrypted_db_path = PathBuf::from(target_encrypted_path_str);
    println!(
        "Zielpfad für verschlüsselte DB: {}",
        final_encrypted_db_path.display()
    );

    if let Some(parent) = final_encrypted_db_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Fehler beim Erstellen des Zielverzeichnisses '{}': {}",
                    parent.display(),
                    e
                )
            })?;
            println!("Zielverzeichnis '{}' erstellt.", parent.display());
        }
    }
    if final_encrypted_db_path.exists() {
        fs::remove_file(&final_encrypted_db_path).map_err(|e| {
            format!(
                "Fehler beim Löschen der alten verschlüsselten DB '{}': {}",
                final_encrypted_db_path.display(),
                e
            )
        })?;
        println!(
            "Alte verschlüsselte DB '{}' gelöscht.",
            final_encrypted_db_path.display()
        );
    }

    let attach_path_str = final_encrypted_db_path.to_str().ok_or_else(|| {
        format!(
            "Ungültiger UTF-8 Pfad für ATTACH: {}",
            final_encrypted_db_path.display()
        )
    })?;

    println!(
        "Hänge neue verschlüsselte DB an: '{}' mit KEY '{}'",
        attach_path_str, key
    );
    source_conn
        .execute(
            "ATTACH DATABASE ?1 AS encrypted_vault KEY ?2;",
            &[attach_path_str, key],
        )
        .map_err(|e| format!("Fehler bei ATTACH DATABASE an '{}': {}", attach_path_str, e))?;
    println!("Verschlüsselte DB 'encrypted_vault' erfolgreich angehängt.");

    println!("Exportiere Daten von 'main' (Quelle) nach 'encrypted_vault'...");
    if let Err(e) = source_conn.execute("SELECT sqlcipher_export('encrypted_vault');", []) {
        eprintln!("!!! FEHLER während sqlcipher_export: {}", e);
        source_conn
            .execute("DETACH DATABASE encrypted_vault;", [])
            .ok(); // Best-effort cleanup
        return Err(format!("Fehler bei sqlcipher_export: {}", e));
    }
    println!("SQLCipher Export nach 'encrypted_vault' erfolgreich.");

    println!("Löse 'encrypted_vault'...");
    source_conn
        .execute("DETACH DATABASE encrypted_vault;", [])
        .map_err(|e| format!("Fehler bei DETACH DATABASE 'encrypted_vault': {}", e))?;
    println!("'encrypted_vault' erfolgreich gelöst.");

    // Verbindung zur Quelldatenbank wird hier durch drop(source_conn) geschlossen.
    Ok(())
}

/// Hilfsfunktion: Öffnet eine verschlüsselte Datenbank und verifiziert sie.
/// Gibt die geöffnete und verifizierte Verbindung zurück.
fn open_and_verify_encrypted_db(db_path: &Path, key: &str) -> Result<Connection, String> {
    println!(
        "Öffne verschlüsselte DB '{}' zur Überprüfung...",
        db_path.display()
    );
    let conn = Connection::open(db_path).map_err(|e| {
        format!(
            "Fehler beim Öffnen der verschlüsselten DB '{}' für Check: {}",
            db_path.display(),
            e
        )
    })?;

    conn.pragma_update(None, "key", key).map_err(|e| {
        format!(
            "Fehler beim Setzen des PRAGMA key für DB '{}': {}",
            db_path.display(),
            e
        )
    })?;
    println!("PRAGMA key für DB '{}' gesetzt.", db_path.display());

    println!("Prüfe SQLCipher-Version auf DB '{}'...", db_path.display());
    match conn.query_row("PRAGMA cipher_version;", [], |row| row.get::<_, String>(0)) {
        Ok(version) => {
            println!(
                "SQLCipher ist aktiv auf DB '{}'! Version: {}",
                db_path.display(),
                version
            );
            match conn.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table';",
                [],
                |row| row.get::<_, i32>(0),
            ) {
                Ok(count) => println!(
                    "Testabfrage erfolgreich: {} Tabelle(n) in DB '{}' gefunden.",
                    count,
                    db_path.display()
                ),
                Err(e) => {
                    eprintln!(
                        "Fehler bei Testabfrage auf verschlüsselter DB '{}': {}",
                        db_path.display(),
                        e
                    );
                    return Err(format!(
                        "Testabfrage auf verschlüsselter DB '{}' fehlgeschlagen: {}",
                        db_path.display(),
                        e
                    ));
                }
            }
        }
        Err(e) => {
            eprintln!(
                "FEHLER: SQLCipher scheint NICHT aktiv zu sein auf DB '{}'!",
                db_path.display()
            );
            eprintln!("'PRAGMA cipher_version;' schlug fehl: {}", e);
            return Err(format!(
                "SQLCipher Aktivitätscheck für DB '{}' fehlgeschlagen: {}",
                db_path.display(),
                e
            ));
        }
    }
    Ok(conn)
}

/// Hauptfunktion: Erstellt eine verschlüsselte Datenbank aus einem gebündelten Asset.
#[tauri::command]
pub fn create_encrypted_database_new(
    app_handle: AppHandle<Wry>,
    path: String,
    key: String,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    let asset_name = "database/vault.db";
    let temp_db_path: PathBuf; // Muss deklariert werden, um im Fehlerfall aufgeräumt werden zu können

    // Schritt 1: Asset vorbereiten
    match prepare_temporary_asset_db(&app_handle, &asset_name, BaseDirectory::Resource) {
        Ok(path) => temp_db_path = path,
        Err(e) => return Err(e),
    }

    // Schritt 2: Datenbank verschlüsseln
    // Wir geben einen String-Slice für path, da die Funktion das erwartet.
    if let Err(e) = encrypt_database_from_source(&temp_db_path, &path, &key) {
        // Versuche, die temporäre Datei auch im Fehlerfall zu löschen
        let _ = fs::remove_file(&temp_db_path); // Fehler beim Löschen hier ignorieren
        return Err(e);
    }

    // Schritt 3: Temporäre Datei aufräumen
    if let Err(e) = fs::remove_file(&temp_db_path) {
        // Logge den Fehler, aber fahre fort, da die verschlüsselte DB erstellt wurde
        eprintln!("Warnung: Fehler beim Löschen der temporären DB '{}': {}. Die verschlüsselte DB wurde jedoch erstellt.", temp_db_path.display(), e);
    } else {
        println!(
            "Temporäre DB '{}' erfolgreich gelöscht.",
            temp_db_path.display()
        );
    }
    println!("Datenbank erfolgreich nach '{}' verschlüsselt.", path);

    // Schritt 4: Neu erstellte verschlüsselte Datenbank öffnen und verifizieren
    let final_encrypted_db_path = PathBuf::from(path.clone()); // Klonen, da String für Pfad benötigt wird
    let final_conn = match open_and_verify_encrypted_db(&final_encrypted_db_path, &key) {
        Ok(conn) => conn,
        Err(e) => {
            // Wenn das Öffnen/Verifizieren fehlschlägt, existiert die Datei vielleicht, ist aber unbrauchbar.
            // Je nach Strategie könnte man hier die `final_encrypted_db_path` löschen.
            return Err(e);
        }
    };

    // Schritt 5: Datenbankverbindung im State aktualisieren
    println!(
        "Aktualisiere DB-Verbindung im State mit '{}'",
        final_encrypted_db_path.display()
    );
    let mut db_state_lock = state
        .0
        .lock()
        .map_err(|e| format!("Mutex-Fehler beim Sperren des DB-Status: {}", e.to_string()))?;
    *db_state_lock = Some(final_conn);

    Ok(format!(
        "Verschlüsselte Datenbank erfolgreich erstellt, geprüft und im State gespeichert unter: {}",
        final_encrypted_db_path.display()
    ))
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

fn get_crsqlite_path(app_handle: AppHandle) -> Result<PathBuf, String> {
    // Laden der cr-sqlite Erweiterung
    let target_triple = get_target_triple()?;

    println!("target_triple: {}", target_triple);

    let crsqlite_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Fehler beim Ermitteln des Ressourcenverzeichnisses: {}", e))?
        .join(format!("crsqlite-{}", target_triple));

    println!("crsqlite_path: {}", crsqlite_path.display());
    Ok(crsqlite_path)
}

#[tauri::command]
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

#[derive(Debug, Clone)]
struct SqlTableInfo {
    cid: u32,
    name: String,
    r#type: String,
    notnull: bool,
    dflt_value: Option<String>,
    pk: u8,
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
