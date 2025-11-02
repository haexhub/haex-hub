// src-tarui/src/build/table_names.rs
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Schema {
    haex: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct TableDefinition {
    name: String,
    columns: HashMap<String, String>,
}

pub fn generate_table_names() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR ist nicht gesetzt.");
    println!("Generiere Tabellennamen nach {out_dir}");
    let schema_path = Path::new("../src/database/tableNames.json");
    let dest_path = Path::new(&out_dir).join("tableNames.rs");

    let file = File::open(schema_path).expect("Konnte tableNames.json nicht öffnen");
    let reader = BufReader::new(file);
    let schema: Schema =
        serde_json::from_reader(reader).expect("Konnte tableNames.json nicht parsen");

    let mut code = String::from(
        r#"
// ==================================================================
// HINWEIS: Diese Datei wurde automatisch von build.rs generiert.
// Manuelle Änderungen werden bei der nächsten Kompilierung überschrieben!
// ==================================================================

"#,
    );

    // Dynamisch über alle Einträge in haex iterieren
    for (key, value) in &schema.haex {
        // Spezialbehandlung für nested structures wie "crdt"
        if key == "crdt" {
            if let Some(crdt_obj) = value.as_object() {
                for (crdt_key, crdt_value) in crdt_obj {
                    if let Ok(table) = serde_json::from_value::<TableDefinition>(crdt_value.clone())
                    {
                        let const_prefix = format!("CRDT_{}", to_screaming_snake_case(crdt_key));
                        code.push_str(&generate_table_constants(&table, &const_prefix));
                    }
                }
            }
        } else {
            // Normale Tabelle (settings, extensions, notifications, workspaces, desktop_items, etc.)
            if let Ok(table) = serde_json::from_value::<TableDefinition>(value.clone()) {
                let const_prefix = to_screaming_snake_case(key);
                code.push_str(&generate_table_constants(&table, &const_prefix));
            }
        }
    }

    // --- Datei schreiben ---
    let mut f = File::create(&dest_path).expect("Konnte Zieldatei nicht erstellen");
    f.write_all(code.as_bytes())
        .expect("Konnte nicht in Zieldatei schreiben");

    println!("cargo:rerun-if-changed=../src/database/tableNames.json");
}

/// Konvertiert einen String zu SCREAMING_SNAKE_CASE
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, ch) in s.chars().enumerate() {
        if ch == '_' {
            result.push('_');
            prev_is_lower = false;
        } else if ch.is_uppercase() {
            if i > 0 && prev_is_lower {
                result.push('_');
            }
            result.push(ch);
            prev_is_lower = false;
        } else {
            result.push(ch.to_ascii_uppercase());
            prev_is_lower = true;
        }
    }

    result
}

/// Generiert die Konstanten für eine Tabelle
fn generate_table_constants(table: &TableDefinition, const_prefix: &str) -> String {
    let mut code = String::new();

    // Tabellenname
    code.push_str(&format!("// --- Table: {} ---\n", table.name));
    code.push_str(&format!(
        "pub const TABLE_{}: &str = \"{}\";\n",
        const_prefix, table.name
    ));

    // Spalten
    for (col_key, col_value) in &table.columns {
        let col_const_name = format!("COL_{}_{}", const_prefix, to_screaming_snake_case(col_key));
        code.push_str(&format!(
            "pub const {col_const_name}: &str = \"{col_value}\";\n"
        ));
    }

    code.push('\n');
    code
}
