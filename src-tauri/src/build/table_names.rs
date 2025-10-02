// build/table_names.rs
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub haex: Haex,
}

#[derive(Debug, Deserialize)]
pub struct Haex {
    pub settings: String,
    pub extensions: String,
    pub extension_permissions: String,
    pub crdt: Crdt,
}

#[derive(Debug, Deserialize)]
pub struct Crdt {
    pub logs: String,
    pub snapshots: String,
    pub configs: String,
}

pub fn generate_table_names(out_dir: &str) {
    let schema_path = Path::new("database/tableNames.json");
    let dest_path = Path::new(out_dir).join("tableNames.rs");

    let file = File::open(&schema_path).expect("Konnte tableNames.json nicht öffnen");
    let reader = BufReader::new(file);
    let schema: Schema =
        serde_json::from_reader(reader).expect("Konnte tableNames.json nicht parsen");
    let haex = schema.haex;

    let code = format!(
        r#"
        // Auto-generated - DO NOT EDIT

        // Core Tables
        pub const TABLE_SETTINGS: &str = "{settings}";
        pub const TABLE_EXTENSIONS: &str = "{extensions}";
        pub const TABLE_EXTENSION_PERMISSIONS: &str = "{extension_permissions}";

        // CRDT Tables
        pub const TABLE_CRDT_LOGS: &str = "{crdt_logs}";
        pub const TABLE_CRDT_SNAPSHOTS: &str = "{crdt_snapshots}";
        pub const TABLE_CRDT_CONFIGS: &str = "{crdt_configs}";
        "#,
        settings = haex.settings,
        extensions = haex.extensions,
        extension_permissions = haex.extension_permissions,
        crdt_logs = haex.crdt.logs,
        crdt_snapshots = haex.crdt.snapshots,
        crdt_configs = haex.crdt.configs
    );

    let mut f = File::create(&dest_path).expect("Konnte Zieldatei nicht erstellen");
    f.write_all(code.as_bytes())
        .expect("Konnte nicht in Zieldatei schreiben");

    println!("cargo:rerun-if-changed=database/tableNames.json");
}
