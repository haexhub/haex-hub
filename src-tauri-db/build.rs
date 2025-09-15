use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Schema {
    haex: Haex,
}

#[derive(Debug, Deserialize)]
struct Haex {
    settings: String,
    extensions: String,
    extension_permissions: String,
    notifications: String,
    passwords: Passwords,
    crdt: Crdt,
}

#[derive(Debug, Deserialize)]
struct Passwords {
    groups: String,
    group_items: String,
    item_details: String,
    item_key_values: String,
    item_histories: String,
}

#[derive(Debug, Deserialize)]
struct Crdt {
    logs: String,
    snapshots: String,
    configs: String,
}

fn main() {
    // Pfad zur Eingabe-JSON und zur Ausgabe-Rust-Datei festlegen.
    // `OUT_DIR` ist ein spezielles Verzeichnis, das Cargo für generierte Dateien bereitstellt.
    let schema_path = Path::new("database/tableNames.json");
    let out_dir =
        env::var("OUT_DIR").expect("OUT_DIR ist nicht gesetzt. Führen Sie dies mit Cargo aus.");
    let dest_path = Path::new(&out_dir).join("tableNames.rs");

    // --- 2. JSON-Datei lesen und mit serde parsen ---
    let file = File::open(&schema_path).expect("Konnte tableNames.json nicht öffnen");
    let reader = BufReader::new(file);
    let schema: Schema =
        serde_json::from_reader(reader).expect("Konnte tableNames.json nicht parsen");
    let haex = schema.haex;

    // --- 3. Den zu generierenden Rust-Code als String erstellen ---
    // Wir verwenden das `format!`-Makro, um die Werte aus den geparsten Structs
    // in einen vordefinierten Code-Template-String einzufügen.
    // Das `r#""#`-Format erlaubt uns, mehrzeilige Strings mit Anführungszeichen zu verwenden.
    let code = format!(
        r#"
        // HINWEIS: Diese Datei wurde automatisch von build.rs generiert.
        // Manuelle Änderungen werden bei der nächsten Kompilierung überschrieben!

        pub const TABLE_SETTINGS: &str = "{settings}";
        pub const TABLE_EXTENSIONS: &str = "{extensions}";
        pub const TABLE_EXTENSION_PERMISSIONS: &str = "{extension_permissions}";
        pub const TABLE_NOTIFICATIONS: &str = "{notifications}";

        // Passwords
        pub const TABLE_PASSWORDS_GROUPS: &str = "{pw_groups}";
        pub const TABLE_PASSWORDS_GROUP_ITEMS: &str = "{pw_group_items}";
        pub const TABLE_PASSWORDS_ITEM_DETAILS: &str = "{pw_item_details}";
        pub const TABLE_PASSWORDS_ITEM_KEY_VALUES: &str = "{pw_item_key_values}";
        pub const TABLE_PASSWORDS_ITEM_HISTORIES: &str = "{pw_item_histories}";

        // CRDT
        pub const TABLE_CRDT_LOGS: &str = "{crdt_logs}";
        pub const TABLE_CRDT_SNAPSHOTS: &str = "{crdt_snapshots}";
        pub const TABLE_CRDT_CONFIGS: &str = "{crdt_configs}";

        "#,
        // Hier werden die Werte aus dem `haex`-Struct in die Platzhalter oben eingesetzt.
        settings = haex.settings,
        extensions = haex.extensions,
        extension_permissions = haex.extension_permissions,
        notifications = haex.notifications,
        pw_groups = haex.passwords.groups,
        pw_group_items = haex.passwords.group_items,
        pw_item_details = haex.passwords.item_details,
        pw_item_key_values = haex.passwords.item_key_values,
        pw_item_histories = haex.passwords.item_histories,
        crdt_logs = haex.crdt.logs,
        crdt_snapshots = haex.crdt.snapshots,
        crdt_configs = haex.crdt.configs
    );

    // --- 4. Den generierten Code in die Zieldatei schreiben ---
    let mut f = File::create(&dest_path).expect("Konnte die Zieldatei nicht erstellen");
    f.write_all(code.as_bytes())
        .expect("Konnte nicht in die Zieldatei schreiben");

    // --- 5. Cargo anweisen, das Skript erneut auszuführen, wenn sich die JSON-Datei ändert ---
    // Diese Zeile ist extrem wichtig für eine reibungslose Entwicklung! Ohne sie
    // würde Cargo Änderungen an der JSON-Datei nicht bemerken.
    println!("cargo:rerun-if-changed=database/tableNames.json");

    tauri_build::build()
}
