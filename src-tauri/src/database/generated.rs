// Auto-generated from Drizzle schema
// DO NOT EDIT MANUALLY
// Run 'pnpm generate:rust-types' to regenerate

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexSettings {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_tombstone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
}

impl HaexSettings {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            key: row.get(1)?,
            r#type: row.get(2)?,
            value: row.get(3)?,
            haex_tombstone: row.get(4)?,
            haex_timestamp: row.get(5)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexExtensions {
    pub id: String,
    pub public_key: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub entry: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_tombstone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
}

impl HaexExtensions {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            public_key: row.get(1)?,
            name: row.get(2)?,
            version: row.get(3)?,
            author: row.get(4)?,
            description: row.get(5)?,
            entry: row.get(6)?,
            homepage: row.get(7)?,
            enabled: row.get(8)?,
            icon: row.get(9)?,
            signature: row.get(10)?,
            haex_tombstone: row.get(11)?,
            haex_timestamp: row.get(12)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexExtensionPermissions {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_tombstone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
}

impl HaexExtensionPermissions {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            extension_id: row.get(1)?,
            resource_type: row.get(2)?,
            action: row.get(3)?,
            target: row.get(4)?,
            constraints: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            haex_tombstone: row.get(9)?,
            haex_timestamp: row.get(10)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexCrdtLogs {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_pks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
}

impl HaexCrdtLogs {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            haex_timestamp: row.get(1)?,
            table_name: row.get(2)?,
            row_pks: row.get(3)?,
            op_type: row.get(4)?,
            column_name: row.get(5)?,
            new_value: row.get(6)?,
            old_value: row.get(7)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexCrdtSnapshots {
    pub snapshot_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch_hlc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_bytes: Option<i64>,
}

impl HaexCrdtSnapshots {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            snapshot_id: row.get(0)?,
            created: row.get(1)?,
            epoch_hlc: row.get(2)?,
            location_url: row.get(3)?,
            file_size_bytes: row.get(4)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaexCrdtConfigs {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl HaexCrdtConfigs {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    }
}

