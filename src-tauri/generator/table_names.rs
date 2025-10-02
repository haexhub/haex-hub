// src-tarui/src/build/table_names.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Schema {
    haex: Haex,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Haex {
    settings: TableDefinition,
    extensions: TableDefinition,
    extension_permissions: TableDefinition,
    notifications: TableDefinition,
    crdt: Crdt,
}

#[derive(Debug, Deserialize)]
struct Crdt {
    logs: TableDefinition,
    snapshots: TableDefinition,
    configs: TableDefinition,
}

#[derive(Debug, Deserialize)]
struct TableDefinition {
    name: String,
    columns: HashMap<String, String>,
}

pub fn generate_table_names() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR ist nicht gesetzt.");
    println!("Generiere Tabellennamen nach {}", out_dir);
    let schema_path = Path::new("database/tableNames.json");
    let dest_path = Path::new(&out_dir).join("tableNames.rs");

    let file = File::open(&schema_path).expect("Konnte tableNames.json nicht öffnen");
    let reader = BufReader::new(file);
    let schema: Schema =
        serde_json::from_reader(reader).expect("Konnte tableNames.json nicht parsen");
    let haex = schema.haex;

    let code = format!(
        r#"
// ==================================================================
// HINWEIS: Diese Datei wurde automatisch von build.rs generiert.
// Manuelle Änderungen werden bei der nächsten Kompilierung überschrieben!
// ==================================================================

// --- Table: haex_settings ---
pub const TABLE_SETTINGS: &str = "{t_settings}";
pub const COL_SETTINGS_ID: &str = "{c_settings_id}";
pub const COL_SETTINGS_KEY: &str = "{c_settings_key}";
pub const COL_SETTINGS_TYPE: &str = "{c_settings_type}";
pub const COL_SETTINGS_VALUE: &str = "{c_settings_value}";
pub const COL_SETTINGS_HAEX_TOMBSTONE: &str = "{c_settings_tombstone}";
pub const COL_SETTINGS_HAEX_TIMESTAMP: &str = "{c_settings_timestamp}";

// --- Table: haex_extensions ---
pub const TABLE_EXTENSIONS: &str = "{t_extensions}";
pub const COL_EXTENSIONS_ID: &str = "{c_ext_id}";
pub const COL_EXTENSIONS_AUTHOR: &str = "{c_ext_author}";
pub const COL_EXTENSIONS_DESCRIPTION: &str = "{c_ext_description}";
pub const COL_EXTENSIONS_ENTRY: &str = "{c_ext_entry}";
pub const COL_EXTENSIONS_HOMEPAGE: &str = "{c_ext_homepage}";
pub const COL_EXTENSIONS_ENABLED: &str = "{c_ext_enabled}";
pub const COL_EXTENSIONS_ICON: &str = "{c_ext_icon}";
pub const COL_EXTENSIONS_NAME: &str = "{c_ext_name}";
pub const COL_EXTENSIONS_PUBLIC_KEY: &str = "{c_ext_public_key}";
pub const COL_EXTENSIONS_SIGNATURE: &str = "{c_ext_signature}";
pub const COL_EXTENSIONS_URL: &str = "{c_ext_url}";
pub const COL_EXTENSIONS_VERSION: &str = "{c_ext_version}";
pub const COL_EXTENSIONS_HAEX_TOMBSTONE: &str = "{c_ext_tombstone}";
pub const COL_EXTENSIONS_HAEX_TIMESTAMP: &str = "{c_ext_timestamp}";

// --- Table: haex_extension_permissions ---
pub const TABLE_EXTENSION_PERMISSIONS: &str = "{t_ext_perms}";
pub const COL_EXT_PERMS_ID: &str = "{c_extp_id}";
pub const COL_EXT_PERMS_EXTENSION_ID: &str = "{c_extp_extensionId}";
pub const COL_EXT_PERMS_RESOURCE_TYPE: &str = "{c_extp_resourceType}";
pub const COL_EXT_PERMS_ACTION: &str = "{c_extp_action}";
pub const COL_EXT_PERMS_TARGET: &str = "{c_extp_target}";
pub const COL_EXT_PERMS_CONSTRAINTS: &str = "{c_extp_constraints}";
pub const COL_EXT_PERMS_STATUS: &str = "{c_extp_status}";
pub const COL_EXT_PERMS_CREATED_AT: &str = "{c_extp_createdAt}";
pub const COL_EXT_PERMS_UPDATE_AT: &str = "{c_extp_updateAt}";
pub const COL_EXT_PERMS_HAEX_TOMBSTONE: &str = "{c_extp_tombstone}";
pub const COL_EXT_PERMS_HAEX_TIMESTAMP: &str = "{c_extp_timestamp}";

// --- Table: haex_notifications ---
pub const TABLE_NOTIFICATIONS: &str = "{t_notifications}";
pub const COL_NOTIFICATIONS_ID: &str = "{c_notif_id}";
pub const COL_NOTIFICATIONS_ALT: &str = "{c_notif_alt}";
pub const COL_NOTIFICATIONS_DATE: &str = "{c_notif_date}";
pub const COL_NOTIFICATIONS_ICON: &str = "{c_notif_icon}";
pub const COL_NOTIFICATIONS_IMAGE: &str = "{c_notif_image}";
pub const COL_NOTIFICATIONS_READ: &str = "{c_notif_read}";
pub const COL_NOTIFICATIONS_SOURCE: &str = "{c_notif_source}";
pub const COL_NOTIFICATIONS_TEXT: &str = "{c_notif_text}";
pub const COL_NOTIFICATIONS_TITLE: &str = "{c_notif_title}";
pub const COL_NOTIFICATIONS_TYPE: &str = "{c_notif_type}";
pub const COL_NOTIFICATIONS_HAEX_TOMBSTONE: &str = "{c_notif_tombstone}";

// --- Table: haex_crdt_logs ---
pub const TABLE_CRDT_LOGS: &str = "{t_crdt_logs}";
pub const COL_CRDT_LOGS_ID: &str = "{c_crdt_logs_id}";
pub const COL_CRDT_LOGS_HAEX_TIMESTAMP: &str = "{c_crdt_logs_timestamp}";
pub const COL_CRDT_LOGS_TABLE_NAME: &str = "{c_crdt_logs_tableName}";
pub const COL_CRDT_LOGS_ROW_PKS: &str = "{c_crdt_logs_rowPks}";
pub const COL_CRDT_LOGS_OP_TYPE: &str = "{c_crdt_logs_opType}";
pub const COL_CRDT_LOGS_COLUMN_NAME: &str = "{c_crdt_logs_columnName}";
pub const COL_CRDT_LOGS_NEW_VALUE: &str = "{c_crdt_logs_newValue}";
pub const COL_CRDT_LOGS_OLD_VALUE: &str = "{c_crdt_logs_oldValue}";

// --- Table: haex_crdt_snapshots ---
pub const TABLE_CRDT_SNAPSHOTS: &str = "{t_crdt_snapshots}";
pub const COL_CRDT_SNAPSHOTS_ID: &str = "{c_crdt_snap_id}";
pub const COL_CRDT_SNAPSHOTS_CREATED: &str = "{c_crdt_snap_created}";
pub const COL_CRDT_SNAPSHOTS_EPOCH_HLC: &str = "{c_crdt_snap_epoch}";
pub const COL_CRDT_SNAPSHOTS_LOCATION_URL: &str = "{c_crdt_snap_location}";
pub const COL_CRDT_SNAPSHOTS_FILE_SIZE: &str = "{c_crdt_snap_size}";

// --- Table: haex_crdt_configs ---
pub const TABLE_CRDT_CONFIGS: &str = "{t_crdt_configs}";
pub const COL_CRDT_CONFIGS_KEY: &str = "{c_crdt_configs_key}";
pub const COL_CRDT_CONFIGS_VALUE: &str = "{c_crdt_configs_value}";
"#,
        // Settings
        t_settings = haex.settings.name,
        c_settings_id = haex.settings.columns["id"],
        c_settings_key = haex.settings.columns["key"],
        c_settings_type = haex.settings.columns["type"],
        c_settings_value = haex.settings.columns["value"],
        c_settings_tombstone = haex.settings.columns["haexTombstone"],
        c_settings_timestamp = haex.settings.columns["haexTimestamp"],
        // Extensions
        t_extensions = haex.extensions.name,
        c_ext_id = haex.extensions.columns["id"],
        c_ext_author = haex.extensions.columns["author"],
        c_ext_description = haex.extensions.columns["description"],
        c_ext_entry = haex.extensions.columns["entry"],
        c_ext_homepage = haex.extensions.columns["homepage"],
        c_ext_enabled = haex.extensions.columns["enabled"],
        c_ext_icon = haex.extensions.columns["icon"],
        c_ext_name = haex.extensions.columns["name"],
        c_ext_public_key = haex.extensions.columns["public_key"],
        c_ext_signature = haex.extensions.columns["signature"],
        c_ext_url = haex.extensions.columns["url"],
        c_ext_version = haex.extensions.columns["version"],
        c_ext_tombstone = haex.extensions.columns["haexTombstone"],
        c_ext_timestamp = haex.extensions.columns["haexTimestamp"],
        // Extension Permissions
        t_ext_perms = haex.extension_permissions.name,
        c_extp_id = haex.extension_permissions.columns["id"],
        c_extp_extensionId = haex.extension_permissions.columns["extensionId"],
        c_extp_resourceType = haex.extension_permissions.columns["resourceType"],
        c_extp_action = haex.extension_permissions.columns["action"],
        c_extp_target = haex.extension_permissions.columns["target"],
        c_extp_constraints = haex.extension_permissions.columns["constraints"],
        c_extp_status = haex.extension_permissions.columns["status"],
        c_extp_createdAt = haex.extension_permissions.columns["createdAt"],
        c_extp_updateAt = haex.extension_permissions.columns["updateAt"],
        c_extp_tombstone = haex.extension_permissions.columns["haexTombstone"],
        c_extp_timestamp = haex.extension_permissions.columns["haexTimestamp"],
        // Notifications
        t_notifications = haex.notifications.name,
        c_notif_id = haex.notifications.columns["id"],
        c_notif_alt = haex.notifications.columns["alt"],
        c_notif_date = haex.notifications.columns["date"],
        c_notif_icon = haex.notifications.columns["icon"],
        c_notif_image = haex.notifications.columns["image"],
        c_notif_read = haex.notifications.columns["read"],
        c_notif_source = haex.notifications.columns["source"],
        c_notif_text = haex.notifications.columns["text"],
        c_notif_title = haex.notifications.columns["title"],
        c_notif_type = haex.notifications.columns["type"],
        c_notif_tombstone = haex.notifications.columns["haexTombstone"],
        // CRDT Logs
        t_crdt_logs = haex.crdt.logs.name,
        c_crdt_logs_id = haex.crdt.logs.columns["id"],
        c_crdt_logs_timestamp = haex.crdt.logs.columns["haexTimestamp"],
        c_crdt_logs_tableName = haex.crdt.logs.columns["tableName"],
        c_crdt_logs_rowPks = haex.crdt.logs.columns["rowPks"],
        c_crdt_logs_opType = haex.crdt.logs.columns["opType"],
        c_crdt_logs_columnName = haex.crdt.logs.columns["columnName"],
        c_crdt_logs_newValue = haex.crdt.logs.columns["newValue"],
        c_crdt_logs_oldValue = haex.crdt.logs.columns["oldValue"],
        // CRDT Snapshots
        t_crdt_snapshots = haex.crdt.snapshots.name,
        c_crdt_snap_id = haex.crdt.snapshots.columns["snapshotId"],
        c_crdt_snap_created = haex.crdt.snapshots.columns["created"],
        c_crdt_snap_epoch = haex.crdt.snapshots.columns["epochHlc"],
        c_crdt_snap_location = haex.crdt.snapshots.columns["locationUrl"],
        c_crdt_snap_size = haex.crdt.snapshots.columns["fileSizeBytes"],
        // CRDT Configs
        t_crdt_configs = haex.crdt.configs.name,
        c_crdt_configs_key = haex.crdt.configs.columns["key"],
        c_crdt_configs_value = haex.crdt.configs.columns["value"]
    );

    // --- Datei schreiben ---
    let mut f = File::create(&dest_path).expect("Konnte Zieldatei nicht erstellen");
    f.write_all(code.as_bytes())
        .expect("Konnte nicht in Zieldatei schreiben");

    println!("cargo:rerun-if-changed=database/tableNames.json");
}
