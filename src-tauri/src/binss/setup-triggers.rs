use crdt::trigger::TriggerManager;
use rusqlite::{Connection, Result};
// anpassen an dein Crate-Modul

fn main() -> Result<()> {
    // Vault-Datenbank öffnen
    let conn = Connection::open("vault.db")?;

    println!("🔄 Setup CRDT triggers...");

    // Tabellen aus der DB holen
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'haex_%' AND NOT LIKE 'haex_crdt_%';")?;
    let table_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

    for table_name in table_iter {
        let table_name = table_name?;
        println!("➡️  Processing table: {}", table_name);

        // Trigger für die Tabelle neu anlegen
        match TriggerManager::setup_triggers_for_table(&conn, &table_name) {
            Ok(_) => println!("   ✅ Triggers created for {}", table_name),
            Err(e) => println!(
                "   ⚠️ Could not create triggers for {}: {:?}",
                table_name, e
            ),
        }
    }

    println!("✨ Done setting up CRDT triggers.");
    Ok(())
}
