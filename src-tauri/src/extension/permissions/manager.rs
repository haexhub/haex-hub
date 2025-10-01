use crate::AppState;
use crate::database::core::with_connection;
use crate::database::error::DatabaseError;
use crate::extension::database::executor::SqlExecutor;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::types::{Action, DbConstraints, ExtensionPermission, FsConstraints, HttpConstraints, PermissionConstraints, PermissionStatus, ResourceType, ShellConstraints};
use serde_json;
use serde_json::json;
use std::path::Path;
use tauri::State;
use url::Url;

pub struct PermissionManager;

impl PermissionManager {
    /// Speichert alle Permissions einer Extension
    pub async fn save_permissions(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        permissions: &[ExtensionPermission],
    ) -> Result<(), ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = app_state
                .hlc
                .lock()
                .map_err(|_| DatabaseError::MutexPoisoned {
                    reason: "Failed to lock HLC service".to_string(),
                })?;

            for perm in permissions {
                let resource_type_str = format!("{:?}", perm.resource_type).to_lowercase();
                let action_str = format!("{:?}", perm.action).to_lowercase();

                let constraints_json = perm
                    .constraints
                    .as_ref()
                    .map(|c| serde_json::to_string(c).ok())
                    .flatten();

                let sql = "INSERT INTO haex_extension_permissions 
                     (id, extension_id, resource_type, action, target, constraints, status) 
                     VALUES (?, ?, ?, ?, ?, ?, ?)";

                let params = vec![
                    json!(perm.id),
                    json!(extension_id),
                    json!(resource_type_str),
                    json!(action_str),
                    json!(perm.target),
                    json!(constraints_json),
                    json!(perm.status.as_str()),
                ];

                SqlExecutor::execute_internal(&tx, &hlc_service, sql, &params)?;
            }

            tx.commit().map_err(DatabaseError::from)?;
            Ok(())
        })
        .map_err(ExtensionError::from)
    }

    /// Aktualisiert eine Permission
    pub async fn update_permission(
        app_state: &State<'_, AppState>,
        permission: &ExtensionPermission,
    ) -> Result<(), ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = app_state
                .hlc
                .lock()
                .map_err(|_| DatabaseError::MutexPoisoned {
                    reason: "Failed to lock HLC service".to_string(),
                })?;

            let resource_type_str = format!("{:?}", permission.resource_type).to_lowercase();
            let action_str = format!("{:?}", permission.action).to_lowercase();

            let constraints_json = permission
                .constraints
                .as_ref()
                .map(|c| serde_json::to_string(c).ok())
                .flatten();

            let sql = "UPDATE haex_extension_permissions 
                 SET resource_type = ?, action = ?, target = ?, constraints = ?, status = ?
                 WHERE id = ?";

            let params = vec![
                json!(resource_type_str),
                json!(action_str),
                json!(permission.target),
                json!(constraints_json),
                json!(permission.status.as_str()),
                json!(permission.id),
            ];

            SqlExecutor::execute_internal(&tx, &hlc_service, sql, &params)?;

            tx.commit().map_err(DatabaseError::from)?;
            Ok(())
        })
        .map_err(ExtensionError::from)
    }

    /// Ändert den Status einer Permission
    pub async fn update_permission_status(
        app_state: &State<'_, AppState>,
        permission_id: &str,
        new_status: PermissionStatus,
    ) -> Result<(), ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;

            let hlc_service = app_state
                .hlc
                .lock()
                .map_err(|_| DatabaseError::MutexPoisoned {
                    reason: "Failed to lock HLC service".to_string(),
                })?;

            let sql = "UPDATE haex_extension_permissions 
                 SET status = ?
                 WHERE id = ?";

            let params = vec![json!(new_status.as_str()), json!(permission_id)];

            SqlExecutor::execute_internal(&tx, &hlc_service, sql, &params)?;

            tx.commit().map_err(DatabaseError::from)?;
            Ok(())
        })
        .map_err(ExtensionError::from)
    }

    /// Löscht alle Permissions einer Extension
   pub async fn delete_permission(
        app_state: &State<'_, AppState>,
        permission_id: &str,
    ) -> Result<(), ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;
            
            let hlc_service = app_state.hlc.lock()
                .map_err(|_| DatabaseError::MutexPoisoned {
                    reason: "Failed to lock HLC service".to_string(),
                })?;
            
            // Echtes DELETE - wird vom CrdtTransformer zu UPDATE umgewandelt
            let sql = "DELETE FROM haex_extension_permissions WHERE id = ?";
            
            let params = vec![json!(permission_id)];
            
            SqlExecutor::execute_internal(&tx, &hlc_service, sql, &params)?;
            
            tx.commit().map_err(DatabaseError::from)?;
            Ok(())
        }).map_err(ExtensionError::from)
    }
    
    /// Löscht alle Permissions einer Extension (Soft-Delete)
    pub async fn delete_permissions(
        app_state: &State<'_, AppState>,
        extension_id: &str,
    ) -> Result<(), ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let tx = conn.transaction().map_err(DatabaseError::from)?;
            
            let hlc_service = app_state.hlc.lock()
                .map_err(|_| DatabaseError::MutexPoisoned {
                    reason: "Failed to lock HLC service".to_string(),
                })?;
            
            // Echtes DELETE - wird vom CrdtTransformer zu UPDATE umgewandelt
            let sql = "DELETE FROM haex_extension_permissions WHERE extension_id = ?";
            
            let params = vec![json!(extension_id)];
            
            SqlExecutor::execute_internal(&tx, &hlc_service, sql, &params)?;
            
            tx.commit().map_err(DatabaseError::from)?;
            Ok(())
        }).map_err(ExtensionError::from)
    }
    /// Lädt alle Permissions einer Extension
    pub async fn get_permissions(
        app_state: &State<'_, AppState>,
        extension_id: &str,
    ) -> Result<Vec<ExtensionPermission>, ExtensionError> {
        with_connection(&app_state.db, |conn| {
            let sql = "SELECT id, extension_id, resource_type, action, target, constraints, status, haex_timestamp, haex_tombstone
                 FROM haex_extension_permissions 
                 WHERE extension_id = ?";
            
            let params = vec![json!(extension_id)];
            
            // SELECT nutzt select_internal
            let results = SqlExecutor::select_internal(conn, sql, &params)?;
            
            // Parse JSON results zu ExtensionPermission
            let permissions = results
                .into_iter()
                .map(|row| Self::parse_permission_from_json(row))
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(permissions)
        }).map_err(ExtensionError::from)
    }

    // Helper für JSON -> ExtensionPermission Konvertierung
    fn parse_permission_from_json(json: serde_json::Value) -> Result<ExtensionPermission, DatabaseError> {

      let obj = json.as_object().ok_or_else(|| DatabaseError::SerializationError {
          reason: "Expected JSON object".to_string(),
      })?;
      
      let resource_type = Self::parse_resource_type(
          obj.get("resource_type")
              .and_then(|v| v.as_str())
              .ok_or_else(|| DatabaseError::SerializationError {
                  reason: "Missing resource_type".to_string(),
              })?
      )?;
      
      let action = Self::parse_action(
        obj.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DatabaseError::SerializationError {
                reason: "Missing action".to_string(),
            })?
        )?;
        
        let status = PermissionStatus::from_str(
            obj.get("status")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DatabaseError::SerializationError {
                    reason: "Missing status".to_string(),
                })?
        )?; // Jetzt funktioniert das ?
        
        let constraints = obj.get("constraints")
            .and_then(|v| v.as_str())
            .map(|json_str| Self::parse_constraints(&resource_type, json_str))
            .transpose()?;
        
        Ok(ExtensionPermission {
            id: obj.get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DatabaseError::SerializationError {
                    reason: "Missing id".to_string(),
                })?
                .to_string(),
            extension_id: obj.get("extension_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DatabaseError::SerializationError {
                    reason: "Missing extension_id".to_string(),
                })?
                .to_string(),
            resource_type,
            action,
            target: obj.get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DatabaseError::SerializationError {
                    reason: "Missing target".to_string(),
                })?
                .to_string(),
            constraints,
            status,
            haex_timestamp: obj.get("haex_timestamp")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            haex_tombstone: obj.get("haex_tombstone")
                .and_then(|v| v.as_i64())
                .map(|i| i == 1),
        })
    }

    /// Prüft Datenbankberechtigungen
   pub async fn check_database_permission(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        action: Action,
        table_name: &str,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_permissions(app_state, extension_id).await?;

        let has_permission = permissions
            .iter()
            .filter(|perm| perm.status == PermissionStatus::Granted) // NUR granted!
            .filter(|perm| perm.resource_type == ResourceType::Db)
            .filter(|perm| perm.action == action) // action ist nicht mehr Option
            .any(|perm| {
                if perm.target != "*" && perm.target != table_name {
                    return false;
                }
                true
            });

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                &format!("{:?}", action),
                &format!("database table '{}'", table_name),
            ));
        }

        Ok(())
    }

    /// Prüft Dateisystem-Berechtigungen
    pub async fn check_filesystem_permission(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        action: Action,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_permissions(app_state, extension_id).await?;

        let file_path_str = file_path.to_string_lossy();

        let has_permission = permissions
            .iter()
            .filter(|perm| perm.status == PermissionStatus::Granted)
            .filter(|perm| perm.resource_type == ResourceType::Fs)
            .filter(|perm| perm.action == action)
            .any(|perm| {
                if !Self::matches_path_pattern(&perm.target, &file_path_str) {
                    return false;
                }

                if let Some(PermissionConstraints::Filesystem(constraints)) = &perm.constraints {
                    if let Some(allowed_ext) = &constraints.allowed_extensions {
                        if let Some(ext) = file_path.extension() {
                            let ext_str = format!(".{}", ext.to_string_lossy());
                            if !allowed_ext.contains(&ext_str) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }

                true
            });

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                &format!("{:?}", action),
                &format!("filesystem path '{}'", file_path_str),
            ));
        }

        Ok(())
    }

    /// Prüft HTTP-Berechtigungen
    pub async fn check_http_permission(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        method: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_permissions(app_state, extension_id).await?;

        let url_parsed = Url::parse(url).map_err(|e| ExtensionError::ValidationError {
            reason: format!("Invalid URL: {}", e),
        })?;

        let domain = url_parsed.host_str().unwrap_or("");

        let has_permission = permissions
            .iter()
            .filter(|perm| perm.status == PermissionStatus::Granted)
            .filter(|perm| perm.resource_type == ResourceType::Http)
            .any(|perm| {
                let domain_matches = perm.target == "*"
                    || perm.target == domain
                    || domain.ends_with(&format!(".{}", perm.target));

                if !domain_matches {
                    return false;
                }

                if let Some(PermissionConstraints::Http(constraints)) = &perm.constraints {
                    if let Some(methods) = &constraints.methods {
                        if !methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                            return false;
                        }
                    }
                }

                true
            });

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                method,
                &format!("HTTP request to '{}'", url),
            ));
        }

        Ok(())
    }

    /// Prüft Shell-Berechtigungen
    pub async fn check_shell_permission(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        command: &str,
        args: &[String],
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_permissions(app_state, extension_id).await?;

        let has_permission = permissions
            .iter()
            .filter(|perm| perm.status == PermissionStatus::Granted)
            .filter(|perm| perm.resource_type == ResourceType::Shell)
            .any(|perm| {
                if perm.target != command && perm.target != "*" {
                    return false;
                }

                if let Some(PermissionConstraints::Shell(constraints)) = &perm.constraints {
                    if let Some(allowed_subcommands) = &constraints.allowed_subcommands {
                        if !args.is_empty() {
                            if !allowed_subcommands.contains(&args[0])
                                && !allowed_subcommands.contains(&"*".to_string())
                            {
                                return false;
                            }
                        }
                    }

                    if let Some(forbidden) = &constraints.forbidden_args {
                        if args.iter().any(|arg| forbidden.contains(arg)) {
                            return false;
                        }
                    }

                    if let Some(allowed_flags) = &constraints.allowed_flags {
                        let user_flags: Vec<_> =
                            args.iter().filter(|arg| arg.starts_with('-')).collect();

                        for flag in user_flags {
                            if !allowed_flags.contains(flag)
                                && !allowed_flags.contains(&"*".to_string())
                            {
                                return false;
                            }
                        }
                    }
                }

                true
            });

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                "execute",
                &format!("shell command '{}' with args {:?}", command, args),
            ));
        }

        Ok(())
    }

    // Helper-Methoden - müssen DatabaseError statt ExtensionError zurückgeben
    fn parse_resource_type(s: &str) -> Result<ResourceType, DatabaseError> {
        match s {
            "fs" => Ok(ResourceType::Fs),
            "http" => Ok(ResourceType::Http),
            "db" => Ok(ResourceType::Db),
            "shell" => Ok(ResourceType::Shell),
            _ => Err(DatabaseError::SerializationError {
                reason: format!("Unknown resource type: {}", s),
            }),
        }
    }

    fn parse_action(s: &str) -> Result<Action, DatabaseError> {
        match s {
            "read" => Ok(Action::Read),
            "write" => Ok(Action::Write),
            _ => Err(DatabaseError::SerializationError {
                reason: format!("Unknown action: {}", s),
            }),
        }
    }

    fn parse_constraints(
        resource_type: &ResourceType,
        json: &str,
    ) -> Result<PermissionConstraints, DatabaseError> {
        match resource_type {
            ResourceType::Db => {
                let constraints: DbConstraints = serde_json::from_str(json)
                    .map_err(|e| DatabaseError::SerializationError {
                        reason: format!("Failed to parse DB constraints: {}", e),
                    })?;
                Ok(PermissionConstraints::Database(constraints))
            }
            ResourceType::Fs => {
                let constraints: FsConstraints = serde_json::from_str(json)
                    .map_err(|e| DatabaseError::SerializationError {
                        reason: format!("Failed to parse FS constraints: {}", e),
                    })?;
                Ok(PermissionConstraints::Filesystem(constraints))
            }
            ResourceType::Http => {
                let constraints: HttpConstraints = serde_json::from_str(json)
                    .map_err(|e| DatabaseError::SerializationError {
                        reason: format!("Failed to parse HTTP constraints: {}", e),
                    })?;
                Ok(PermissionConstraints::Http(constraints))
            }
            ResourceType::Shell => {
                let constraints: ShellConstraints = serde_json::from_str(json)
                    .map_err(|e| DatabaseError::SerializationError {
                        reason: format!("Failed to parse Shell constraints: {}", e),
                    })?;
                Ok(PermissionConstraints::Shell(constraints))
            }
        }
    }

    fn matches_path_pattern(pattern: &str, path: &str) -> bool {
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return path.starts_with(prefix);
        }

        if pattern.starts_with("*.") {
            let suffix = &pattern[1..];
            return path.ends_with(suffix);
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }

        pattern == path || pattern == "*"
    }

    

}

// Convenience-Funktionen für die verschiedenen Subsysteme
impl PermissionManager {
    // Convenience-Methoden
    pub async fn can_read_file(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        Self::check_filesystem_permission(app_state, extension_id, Action::Read, file_path).await
    }

    pub async fn can_write_file(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        Self::check_filesystem_permission(app_state, extension_id, Action::Write, file_path).await
    }

    pub async fn can_read_table(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        table_name: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_database_permission(app_state, extension_id, Action::Read, table_name).await
    }

    pub async fn can_write_table(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        table_name: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_database_permission(app_state, extension_id, Action::Write, table_name).await
    }

    pub async fn can_http_get(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_http_permission(app_state, extension_id, "GET", url).await
    }

    pub async fn can_http_post(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_http_permission(app_state, extension_id, "POST", url).await
    }

    pub async fn can_execute_command(
        app_state: &State<'_, AppState>,
        extension_id: &str,
        command: &str,
        args: &[String],
    ) -> Result<(), ExtensionError> {
        Self::check_shell_permission(app_state, extension_id, command, args).await
    }

    pub async fn grant_permission(
        app_state: &State<'_, AppState>,
        permission_id: &str,
    ) -> Result<(), ExtensionError> {
        Self::update_permission_status(app_state, permission_id, PermissionStatus::Granted).await
    }

    pub async fn deny_permission(
        app_state: &State<'_, AppState>,
        permission_id: &str,
    ) -> Result<(), ExtensionError> {
        Self::update_permission_status(app_state, permission_id, PermissionStatus::Denied).await
    }

    pub async fn ask_permission(
        app_state: &State<'_, AppState>,
        permission_id: &str,
    ) -> Result<(), ExtensionError> {
        Self::update_permission_status(app_state, permission_id, PermissionStatus::Ask).await
    }

    pub async fn get_ask_permissions(
        app_state: &State<'_, AppState>,
        extension_id: &str,
    ) -> Result<Vec<ExtensionPermission>, ExtensionError> {
        let all_permissions = Self::get_permissions(app_state, extension_id).await?;
        Ok(all_permissions
            .into_iter()
            .filter(|perm| perm.status == PermissionStatus::Ask)
            .collect())
    }
}
