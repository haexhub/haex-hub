// src-tauri/src/database/init.rs
// Database initialization utilities (trigger setup, etc.)

use crate::crdt::trigger;
use crate::database::error::DatabaseError;
use crate::table_names::{
    TABLE_DESKTOP_ITEMS,
    TABLE_EXTENSIONS,
    TABLE_EXTENSION_PERMISSIONS,
    TABLE_NOTIFICATIONS,
    TABLE_SETTINGS,
    TABLE_WORKSPACES,
};
use rusqlite::{params, Connection};

/// Liste aller CRDT-Tabellen die Trigger benötigen (ohne Password-Tabellen - die kommen in Extension)
const CRDT_TABLES: &[&str] = &[
    TABLE_SETTINGS,
    TABLE_EXTENSIONS,
    TABLE_EXTENSION_PERMISSIONS,
    TABLE_NOTIFICATIONS,
    TABLE_WORKSPACES,
    TABLE_DESKTOP_ITEMS,
];

/// Prüft ob Trigger bereits initialisiert wurden und erstellt sie falls nötig
///
/// Diese Funktion wird beim ersten Öffnen einer Template-DB aufgerufen.
/// Sie erstellt alle CRDT-Trigger für die definierten Tabellen und markiert
/// die Initialisierung in haex_settings.
///
/// Bei Migrations (ALTER TABLE) werden Trigger automatisch neu erstellt,
/// daher ist kein Versioning nötig.
pub fn ensure_triggers_initialized(conn: &mut Connection) -> Result<bool, DatabaseError> {
    let tx = conn.transaction()?;

    // Check if triggers already initialized
    let check_sql = format!(
        "SELECT value FROM {TABLE_SETTINGS} WHERE key = ? AND type = ?"
    );
    let initialized: Option<String> = tx
        .query_row(
            &check_sql,
            params!["triggers_initialized", "system"],
            |row| row.get(0),
        )
        .ok();

    if initialized.is_some() {
        eprintln!("DEBUG: Triggers already initialized, skipping");
        tx.commit()?; // Wichtig: Transaktion trotzdem abschließen
        return Ok(true); // true = war schon initialisiert
    }

    eprintln!("INFO: Initializing CRDT triggers for database...");

    // Create triggers for all CRDT tables
    for table_name in CRDT_TABLES {
        eprintln!("  - Setting up triggers for: {table_name}");
        trigger::setup_triggers_for_table(&tx, table_name, false)?;
    }

    tx.commit()?;
    eprintln!("INFO: ✓ CRDT triggers created successfully (flag pending)");
    Ok(false) // false = wurde gerade initialisiert
}
