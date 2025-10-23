use crate::extension::error::ExtensionError;
use crate::extension::permissions::types::{
    Action, DbAction, ExtensionPermission, FsAction, HttpAction, PermissionConstraints,
    PermissionStatus, ResourceType, ShellAction,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ts_rs::TS;

/// Repräsentiert einen einzelnen Berechtigungseintrag im Manifest und im UI-Modell.
#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct PermissionEntry {
    pub target: String,

    /// Die auszuführende Aktion (z.B. "read", "read_write", "GET", "execute").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,

    /// Optionale, spezifische Einschränkungen für diese Berechtigung.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(type = "Record<string, unknown>")]
    pub constraints: Option<serde_json::Value>,

    /// Der Status der Berechtigung (wird nur im UI-Modell verwendet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<PermissionStatus>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExtensionPreview {
    pub manifest: ExtensionManifest,
    pub is_valid_signature: bool,
    pub editable_permissions: EditablePermissions,
}
/// Definiert die einheitliche Struktur für alle Berechtigungsarten im Manifest und UI.
#[derive(Serialize, Deserialize, Clone, Debug, Default, TS)]
#[ts(export)]
pub struct ExtensionPermissions {
    #[serde(default)]
    pub database: Option<Vec<PermissionEntry>>,
    #[serde(default)]
    pub filesystem: Option<Vec<PermissionEntry>>,
    #[serde(default)]
    pub http: Option<Vec<PermissionEntry>>,
    #[serde(default)]
    pub shell: Option<Vec<PermissionEntry>>,
}

/// Typ-Alias für bessere Lesbarkeit, wenn die Struktur als UI-Modell verwendet wird.
pub type EditablePermissions = ExtensionPermissions;

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub entry: String,
    pub icon: Option<String>,
    pub public_key: String,
    pub signature: String,
    pub permissions: ExtensionPermissions,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

impl ExtensionManifest {
    /// Konvertiert die Manifest-Berechtigungen in das bearbeitbare UI-Modell,
    /// indem der Standardstatus `Granted` gesetzt wird.
    pub fn to_editable_permissions(&self) -> EditablePermissions {
        let mut editable = self.permissions.clone();

        let set_status_for_list = |list: Option<&mut Vec<PermissionEntry>>| {
            if let Some(entries) = list {
                for entry in entries.iter_mut() {
                    entry.status = Some(PermissionStatus::Granted);
                }
            }
        };

        set_status_for_list(editable.database.as_mut());
        set_status_for_list(editable.filesystem.as_mut());
        set_status_for_list(editable.http.as_mut());
        set_status_for_list(editable.shell.as_mut());

        editable
    }
}

impl ExtensionPermissions {
    /// Konvertiert das UI-Modell in die flache Liste von internen `ExtensionPermission`-Objekten.
    pub fn to_internal_permissions(&self, extension_id: &str) -> Vec<ExtensionPermission> {
        let mut permissions = Vec::new();

        if let Some(entries) = &self.database {
            for p in entries {
                if let Some(perm) = Self::create_internal(extension_id, ResourceType::Db, p) {
                    permissions.push(perm);
                }
            }
        }
        if let Some(entries) = &self.filesystem {
            for p in entries {
                if let Some(perm) = Self::create_internal(extension_id, ResourceType::Fs, p) {
                    permissions.push(perm);
                }
            }
        }
        if let Some(entries) = &self.http {
            for p in entries {
                if let Some(perm) = Self::create_internal(extension_id, ResourceType::Http, p) {
                    permissions.push(perm);
                }
            }
        }
        if let Some(entries) = &self.shell {
            for p in entries {
                if let Some(perm) = Self::create_internal(extension_id, ResourceType::Shell, p) {
                    permissions.push(perm);
                }
            }
        }

        permissions
    }

    /// Parst einen einzelnen `PermissionEntry` und wandelt ihn in die interne, typsichere `ExtensionPermission`-Struktur um.
    fn create_internal(
        extension_id: &str,
        resource_type: ResourceType,
        p: &PermissionEntry,
    ) -> Option<ExtensionPermission> {
        let operation_str = p.operation.as_deref().unwrap_or_default();

        let action = match resource_type {
            ResourceType::Db => DbAction::from_str(operation_str).ok().map(Action::Database),
            ResourceType::Fs => FsAction::from_str(operation_str)
                .ok()
                .map(Action::Filesystem),
            ResourceType::Http => HttpAction::from_str(operation_str).ok().map(Action::Http),
            ResourceType::Shell => ShellAction::from_str(operation_str).ok().map(Action::Shell),
        };

        action.map(|act| ExtensionPermission {
            id: uuid::Uuid::new_v4().to_string(),
            extension_id: extension_id.to_string(),
            resource_type: resource_type.clone(),
            action: act,
            target: p.target.clone(),
            constraints: p
                .constraints
                .as_ref()
                .and_then(|c| serde_json::from_value::<PermissionConstraints>(c.clone()).ok()),
            status: p.status.clone().unwrap_or(PermissionStatus::Ask),
            haex_timestamp: None,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfoResponse {
    pub id: String,
    pub public_key: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub enabled: bool,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_server_url: Option<String>,
}

impl ExtensionInfoResponse {
    pub fn from_extension(
        extension: &crate::extension::core::types::Extension,
    ) -> Result<Self, ExtensionError> {
        use crate::extension::core::types::ExtensionSource;

        let dev_server_url = match &extension.source {
            ExtensionSource::Development { dev_server_url, .. } => Some(dev_server_url.clone()),
            ExtensionSource::Production { .. } => None,
        };

        Ok(Self {
            id: extension.id.clone(),
            public_key: extension.manifest.public_key.clone(),
            name: extension.manifest.name.clone(),
            version: extension.manifest.version.clone(),
            author: extension.manifest.author.clone(),
            enabled: extension.enabled,
            description: extension.manifest.description.clone(),
            homepage: extension.manifest.homepage.clone(),
            icon: extension.manifest.icon.clone(),
            dev_server_url,
        })
    }
}
