// src-tauri/src/extension/permissions/types.rs

use std::str::FromStr;

use crate::{
    database::{error::DatabaseError, generated::HaexExtensionPermissions},
    extension::permissions::manager::PermissionManager,
};
use serde::{Deserialize, Serialize};

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

impl From<HaexExtensionPermissions> for ExtensionPermission {
    fn from(db_perm: HaexExtensionPermissions) -> Self {
        let resource_type = ResourceType::from_str(&db_perm.resource_type.unwrap_or_default())
            .unwrap_or(ResourceType::Db); // Fallback

        let constraints = db_perm
            .constraints
            .and_then(|json_str| parse_constraints(&resource_type, &json_str).ok());

        ExtensionPermission {
            id: db_perm.id,
            extension_id: db_perm.extension_id.unwrap_or_default(),
            resource_type,
            action: Action::from_str(&db_perm.action.unwrap_or_default()).unwrap_or(Action::Read),
            target: db_perm.target.unwrap_or_default(),
            status: PermissionStatus::from_str(&db_perm.status).unwrap_or(PermissionStatus::Ask),
            constraints,
            haex_timestamp: db_perm.haex_timestamp,
            haex_tombstone: db_perm.haex_tombstone,
        }
    }
}

impl From<&ExtensionPermission> for HaexExtensionPermissions {
    fn from(perm: &ExtensionPermission) -> Self {
        let constraints_json = perm
            .constraints
            .as_ref()
            .and_then(|c| serde_json::to_string(c).ok());

        HaexExtensionPermissions {
            id: perm.id.clone(),
            extension_id: Some(perm.extension_id.clone()),
            resource_type: Some(format!("{:?}", perm.resource_type).to_lowercase()),
            action: Some(format!("{:?}", perm.action).to_lowercase()),
            target: Some(perm.target.clone()),
            constraints: constraints_json,
            status: perm.status.as_str().to_string(),
            created_at: None, // Wird von der DB gesetzt
            updated_at: None, // Wird von der DB gesetzt
            haex_timestamp: perm.haex_timestamp.clone(),
            haex_tombstone: perm.haex_tombstone,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Fs,
    Http,
    Db,
    Shell,
}

impl FromStr for ResourceType {
    type Err = DatabaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fs" => Ok(ResourceType::Fs),
            "http" => Ok(ResourceType::Http),
            "db" => Ok(ResourceType::Db),
            "shell" => Ok(ResourceType::Shell),
            _ => Err(DatabaseError::SerializationError {
                reason: format!("Unbekannter Ressourcentyp: {}", s),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Read,
    Write,
}

impl FromStr for Action {
    type Err = DatabaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Action::Read),
            "write" => Ok(Action::Write),
            _ => Err(DatabaseError::SerializationError {
                reason: format!("Unbekannte Aktion: {}", s),
            }),
        }
    }
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
/* impl EditablePermissions {
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
} */

pub fn parse_constraints(
    resource_type: &ResourceType,
    json: &str,
) -> Result<PermissionConstraints, DatabaseError> {
    match resource_type {
        ResourceType::Db => {
            let constraints: DbConstraints =
                serde_json::from_str(json).map_err(|e| DatabaseError::SerializationError {
                    reason: format!("Failed to parse DB constraints: {}", e),
                })?;
            Ok(PermissionConstraints::Database(constraints))
        }
        ResourceType::Fs => {
            let constraints: FsConstraints =
                serde_json::from_str(json).map_err(|e| DatabaseError::SerializationError {
                    reason: format!("Failed to parse FS constraints: {}", e),
                })?;
            Ok(PermissionConstraints::Filesystem(constraints))
        }
        ResourceType::Http => {
            let constraints: HttpConstraints =
                serde_json::from_str(json).map_err(|e| DatabaseError::SerializationError {
                    reason: format!("Failed to parse HTTP constraints: {}", e),
                })?;
            Ok(PermissionConstraints::Http(constraints))
        }
        ResourceType::Shell => {
            let constraints: ShellConstraints =
                serde_json::from_str(json).map_err(|e| DatabaseError::SerializationError {
                    reason: format!("Failed to parse Shell constraints: {}", e),
                })?;
            Ok(PermissionConstraints::Shell(constraints))
        }
    }
}
