// src-tauri/src/extension/core/manifest.rs

use crate::extension::crypto::ExtensionCrypto;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::types::{
    Action, DbConstraints, ExtensionPermission, FsConstraints, HttpConstraints,
    PermissionConstraints, PermissionStatus, ResourceType, ShellConstraints,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub entry: String,
    pub icon: Option<String>,
    pub public_key: String,
    pub signature: String,
    pub permissions: ExtensionManifestPermissions,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

impl ExtensionManifest {
    pub fn calculate_key_hash(&self) -> Result<String, ExtensionError> {
        ExtensionCrypto::calculate_key_hash(&self.public_key)
            .map_err(|e| ExtensionError::InvalidPublicKey { reason: e })
    }

    pub fn full_extension_id(&self) -> Result<String, ExtensionError> {
        let key_hash = self.calculate_key_hash()?;
        Ok(format!("{}-{}", key_hash, self.id))
    }

    pub fn to_editable_permissions(&self) -> EditablePermissions {
        let mut permissions = Vec::new();

        if let Some(db) = &self.permissions.database {
            for resource in &db.read {
                permissions.push(EditablePermission {
                    resource_type: "db".to_string(),
                    action: "read".to_string(),
                    target: resource.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
            for resource in &db.write {
                permissions.push(EditablePermission {
                    resource_type: "db".to_string(),
                    action: "write".to_string(),
                    target: resource.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        if let Some(fs) = &self.permissions.filesystem {
            for path in &fs.read {
                permissions.push(EditablePermission {
                    resource_type: "fs".to_string(),
                    action: "read".to_string(),
                    target: path.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
            for path in &fs.write {
                permissions.push(EditablePermission {
                    resource_type: "fs".to_string(),
                    action: "write".to_string(),
                    target: path.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        if let Some(http_list) = &self.permissions.http {
            for domain in http_list {
                permissions.push(EditablePermission {
                    resource_type: "http".to_string(),
                    action: "read".to_string(),
                    target: domain.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        if let Some(shell_list) = &self.permissions.shell {
            for command in shell_list {
                permissions.push(EditablePermission {
                    resource_type: "shell".to_string(),
                    action: "read".to_string(),
                    target: command.clone(),
                    constraints: None,
                    status: "granted".to_string(),
                });
            }
        }

        EditablePermissions { permissions }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtensionManifestPermissions {
    #[serde(default)]
    pub database: Option<DatabaseManifestPermissions>,
    #[serde(default)]
    pub filesystem: Option<FilesystemManifestPermissions>,
    #[serde(default)]
    pub http: Option<Vec<String>>,
    #[serde(default)]
    pub shell: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DatabaseManifestPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FilesystemManifestPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

// Editable Permissions für UI
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditablePermissions {
    pub permissions: Vec<EditablePermission>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditablePermission {
    pub resource_type: String,
    pub action: String,
    pub target: String,
    pub constraints: Option<serde_json::Value>,
    pub status: String,
}

impl EditablePermissions {
    pub fn to_internal_permissions(&self, extension_id: &str) -> Vec<ExtensionPermission> {
        self.permissions
            .iter()
            .map(|p| ExtensionPermission {
                id: uuid::Uuid::new_v4().to_string(),
                extension_id: extension_id.to_string(),
                resource_type: match p.resource_type.as_str() {
                    "fs" => ResourceType::Fs,
                    "http" => ResourceType::Http,
                    "db" => ResourceType::Db,
                    "shell" => ResourceType::Shell,
                    _ => ResourceType::Fs,
                },
                action: match p.action.as_str() {
                    "read" => Action::Read,
                    "write" => Action::Write,
                    _ => Action::Read,
                },
                target: p.target.clone(),
                constraints: p
                    .constraints
                    .as_ref()
                    .and_then(|c| Self::parse_constraints(&p.resource_type, c)),
                status: match p.status.as_str() {
                    "granted" => PermissionStatus::Granted,
                    "denied" => PermissionStatus::Denied,
                    "ask" => PermissionStatus::Ask,
                    _ => PermissionStatus::Denied,
                },
                haex_timestamp: None,
                haex_tombstone: None,
            })
            .collect()
    }

    fn parse_constraints(
        resource_type: &str,
        json_value: &serde_json::Value,
    ) -> Option<PermissionConstraints> {
        match resource_type {
            "db" => serde_json::from_value::<DbConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Database),
            "fs" => serde_json::from_value::<FsConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Filesystem),
            "http" => serde_json::from_value::<HttpConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Http),
            "shell" => serde_json::from_value::<ShellConstraints>(json_value.clone())
                .ok()
                .map(PermissionConstraints::Shell),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ExtensionPreview {
    pub manifest: ExtensionManifest,
    pub is_valid_signature: bool,
    pub key_hash: String,
    pub editable_permissions: EditablePermissions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionInfoResponse {
    pub key_hash: String,
    pub name: String,
    pub full_id: String,
    pub version: String,
    pub display_name: Option<String>,
    pub namespace: Option<String>,
    pub allowed_origin: String,
}

impl ExtensionInfoResponse {
    pub fn from_extension(
        extension: &crate::extension::core::types::Extension,
    ) -> Result<Self, ExtensionError> {
        use crate::extension::core::types::get_tauri_origin;

        let allowed_origin = get_tauri_origin();
        let key_hash = extension.manifest.calculate_key_hash()?;
        let full_id = extension.manifest.full_extension_id()?;

        Ok(Self {
            key_hash,
            name: extension.manifest.name.clone(),
            full_id,
            version: extension.manifest.version.clone(),
            display_name: Some(extension.manifest.name.clone()),
            namespace: extension.manifest.author.clone(),
            allowed_origin,
        })
    }
}
