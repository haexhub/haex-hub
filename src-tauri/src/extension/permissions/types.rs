use crate::extension::error::ExtensionError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ts_rs::TS;

// --- Spezifische Aktionen ---

/// Definiert Aktionen, die auf eine Datenbank angewendet werden können.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DbAction {
    Read,
    ReadWrite,
    Create,
    Delete,
    AlterDrop,
}

impl DbAction {
    /// Prüft, ob diese Aktion Lesezugriff gewährt (implizites Recht).
    pub fn allows_read(&self) -> bool {
        matches!(self, DbAction::Read | DbAction::ReadWrite)
    }

    /// Prüft, ob diese Aktion Schreibzugriff gewährt.
    pub fn allows_write(&self) -> bool {
        matches!(
            self,
            DbAction::ReadWrite | DbAction::Create | DbAction::Delete
        )
    }
}

impl FromStr for DbAction {
    type Err = ExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(DbAction::Read),
            "readwrite" | "read_write" => Ok(DbAction::ReadWrite),
            "create" => Ok(DbAction::Create),
            "delete" => Ok(DbAction::Delete),
            "alterdrop" | "alter_drop" => Ok(DbAction::AlterDrop),
            _ => Err(ExtensionError::InvalidActionString {
                input: s.to_string(),
                resource_type: "database".to_string(),
            }),
        }
    }
}

/// Definiert Aktionen, die auf das Dateisystem angewendet werden können.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FsAction {
    Read,
    ReadWrite,
}

impl FsAction {
    /// Prüft, ob diese Aktion Lesezugriff gewährt (implizites Recht).
    pub fn allows_read(&self) -> bool {
        matches!(self, FsAction::Read | FsAction::ReadWrite)
    }

    /// Prüft, ob diese Aktion Schreibzugriff gewährt.
    pub fn allows_write(&self) -> bool {
        matches!(self, FsAction::ReadWrite)
    }
}

impl FromStr for FsAction {
    type Err = ExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(FsAction::Read),
            "readwrite" | "read_write" => Ok(FsAction::ReadWrite),
            _ => Err(ExtensionError::InvalidActionString {
                input: s.to_string(),
                resource_type: "filesystem".to_string(),
            }),
        }
    }
}

/// Definiert Aktionen (HTTP-Methoden), die auf HTTP-Anfragen angewendet werden können.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "UPPERCASE")]
#[ts(export)]
pub enum HttpAction {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    #[serde(rename = "*")]
    All,
}

impl FromStr for HttpAction {
    type Err = ExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpAction::Get),
            "POST" => Ok(HttpAction::Post),
            "PUT" => Ok(HttpAction::Put),
            "PATCH" => Ok(HttpAction::Patch),
            "DELETE" => Ok(HttpAction::Delete),
            "*" => Ok(HttpAction::All),
            _ => Err(ExtensionError::InvalidActionString {
                input: s.to_string(),
                resource_type: "http".to_string(),
            }),
        }
    }
}

/// Definiert Aktionen, die auf Shell-Befehle angewendet werden können.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ShellAction {
    Execute,
}

impl FromStr for ShellAction {
    type Err = ExtensionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "execute" => Ok(ShellAction::Execute),
            _ => Err(ExtensionError::InvalidActionString {
                input: s.to_string(),
                resource_type: "shell".to_string(),
            }),
        }
    }
}

// --- Haupt-Typen für Berechtigungen ---

/// Ein typsicherer Container, der die spezifische Aktion für einen Ressourcentyp enthält.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Action {
    Database(DbAction),
    Filesystem(FsAction),
    Http(HttpAction),
    Shell(ShellAction),
}

/// Die interne Repräsentation einer einzelnen, gewährten Berechtigung.
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haex_timestamp: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ResourceType {
    Fs,
    Http,
    Db,
    Shell,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PermissionStatus {
    Ask,
    Granted,
    Denied,
}

// --- Constraint-Typen (unverändert) ---

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum PermissionConstraints {
    Database(DbConstraints),
    Filesystem(FsConstraints),
    Http(HttpConstraints),
    Shell(ShellConstraints),
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct DbConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct FsConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_extensions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct HttpConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct RateLimit {
    pub requests: u32,
    pub per_minutes: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct ShellConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_subcommands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_flags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_args: Option<Vec<String>>,
}

// --- Konvertierungen zwischen ExtensionPermission und HaexExtensionPermissions ---

impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::Fs => "fs",
            ResourceType::Http => "http",
            ResourceType::Db => "db",
            ResourceType::Shell => "shell",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ExtensionError> {
        match s {
            "fs" => Ok(ResourceType::Fs),
            "http" => Ok(ResourceType::Http),
            "db" => Ok(ResourceType::Db),
            "shell" => Ok(ResourceType::Shell),
            _ => Err(ExtensionError::ValidationError {
                reason: format!("Unknown resource type: {}", s),
            }),
        }
    }
}

impl Action {
    pub fn as_str(&self) -> String {
        match self {
            Action::Database(action) => serde_json::to_string(action)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            Action::Filesystem(action) => serde_json::to_string(action)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            Action::Http(action) => serde_json::to_string(action)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            Action::Shell(action) => serde_json::to_string(action)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
        }
    }

    pub fn from_str(resource_type: &ResourceType, s: &str) -> Result<Self, ExtensionError> {
        match resource_type {
            ResourceType::Db => Ok(Action::Database(DbAction::from_str(s)?)),
            ResourceType::Fs => Ok(Action::Filesystem(FsAction::from_str(s)?)),
            ResourceType::Http => {
                let action: HttpAction =
                    serde_json::from_str(&format!("\"{}\"", s)).map_err(|_| {
                        ExtensionError::InvalidActionString {
                            input: s.to_string(),
                            resource_type: "http".to_string(),
                        }
                    })?;
                Ok(Action::Http(action))
            }
            ResourceType::Shell => Ok(Action::Shell(ShellAction::from_str(s)?)),
        }
    }
}

impl PermissionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PermissionStatus::Ask => "ask",
            PermissionStatus::Granted => "granted",
            PermissionStatus::Denied => "denied",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ExtensionError> {
        match s {
            "ask" => Ok(PermissionStatus::Ask),
            "granted" => Ok(PermissionStatus::Granted),
            "denied" => Ok(PermissionStatus::Denied),
            _ => Err(ExtensionError::ValidationError {
                reason: format!("Unknown permission status: {}", s),
            }),
        }
    }
}

impl From<&ExtensionPermission> for crate::database::generated::HaexExtensionPermissions {
    fn from(perm: &ExtensionPermission) -> Self {
        Self {
            id: perm.id.clone(),
            extension_id: perm.extension_id.clone(),
            resource_type: Some(perm.resource_type.as_str().to_string()),
            action: Some(perm.action.as_str().to_string()),
            target: Some(perm.target.clone()),
            constraints: perm
                .constraints
                .as_ref()
                .and_then(|c| serde_json::to_string(c).ok()),
            status: perm.status.as_str().to_string(),
            created_at: None,
            updated_at: None,
            haex_timestamp: perm.haex_timestamp.clone(),
        }
    }
}

impl From<crate::database::generated::HaexExtensionPermissions> for ExtensionPermission {
    fn from(db_perm: crate::database::generated::HaexExtensionPermissions) -> Self {
        let resource_type = db_perm
            .resource_type
            .as_deref()
            .and_then(|s| ResourceType::from_str(s).ok())
            .unwrap_or(ResourceType::Db);

        let action = db_perm
            .action
            .as_deref()
            .and_then(|s| Action::from_str(&resource_type, s).ok())
            .unwrap_or(Action::Database(DbAction::Read));

        let status =
            PermissionStatus::from_str(db_perm.status.as_str()).unwrap_or(PermissionStatus::Denied);

        let constraints = db_perm
            .constraints
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());

        Self {
            id: db_perm.id,
            extension_id: db_perm.extension_id,
            resource_type,
            action,
            target: db_perm.target.unwrap_or_default(),
            constraints,
            status,
            haex_timestamp: db_perm.haex_timestamp,
        }
    }
}
