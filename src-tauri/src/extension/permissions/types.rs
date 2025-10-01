use serde::{Deserialize, Serialize};

use crate::database::error::DatabaseError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionPermission {
    pub id: String,
    pub extension_id: String,
    pub resource_type: ResourceType,
    pub action: Action,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<PermissionConstraints>,
    pub status: PermissionStatus,

    // CRDT Felder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_tombstone: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Fs,
    Http,
    Db,
    Shell,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Read,
    Write,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionStatus {
    Ask,
    Granted,
    Denied,
}

impl PermissionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PermissionStatus::Ask => "ask",
            PermissionStatus::Granted => "granted",
            PermissionStatus::Denied => "denied",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DatabaseError> {
        match s {
            "ask" => Ok(PermissionStatus::Ask),
            "granted" => Ok(PermissionStatus::Granted),
            "denied" => Ok(PermissionStatus::Denied),
            _ => Err(DatabaseError::SerializationError {
                reason: format!("Unknown permission status: {}", s),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PermissionConstraints {
    Database(DbConstraints),
    Filesystem(FsConstraints),
    Http(HttpConstraints),
    Shell(ShellConstraints),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FsConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateLimit {
    pub requests: u32,
    pub per_minutes: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShellConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_subcommands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_flags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_args: Option<Vec<String>>,
}

// Wenn du weiterhin gruppierte Permissions brauchst:
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditablePermissions {
    pub permissions: Vec<ExtensionPermission>,
}

// Oder gruppiert nach Typ:
impl EditablePermissions {
    pub fn database_permissions(&self) -> Vec<&ExtensionPermission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == ResourceType::Db)
            .collect()
    }

    pub fn filesystem_permissions(&self) -> Vec<&ExtensionPermission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == ResourceType::Fs)
            .collect()
    }

    pub fn http_permissions(&self) -> Vec<&ExtensionPermission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == ResourceType::Http)
            .collect()
    }

    pub fn shell_permissions(&self) -> Vec<&ExtensionPermission> {
        self.permissions
            .iter()
            .filter(|p| p.resource_type == ResourceType::Shell)
            .collect()
    }
}
