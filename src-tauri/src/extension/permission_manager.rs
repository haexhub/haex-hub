/// src-tauri/src/extension/permission_manager.rs

use crate::extension::error::ExtensionError;
use crate::database::DbConnection;
use crate::extension::database::permissions::DbExtensionPermission;
use serde::{Deserialize, Serialize};
use tauri::Url;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionPermissions {
    pub database: Vec<DbExtensionPermission>,
    pub filesystem: Vec<FilesystemPermission>,
    pub http: Vec<HttpPermission>,
    pub shell: Vec<ShellPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesystemPermission {
    pub extension_id: String,
    pub operation: String, // read, write, create, delete
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpPermission {
    pub extension_id: String,
    pub operation: String, // get, post, put, delete
    pub domain: String,
    pub path_pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShellPermission {
    pub extension_id: String,
    pub command: String,
    pub arguments: Vec<String>,
}

/// Zentraler Permission Manager
pub struct PermissionManager;

impl PermissionManager {
    /// Prüft Datenbankberechtigungen
    pub async fn check_database_permission(
        connection: &DbConnection,
        extension_id: &str,
        operation: &str,
        table_name: &str,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_database_permissions(connection, extension_id, operation).await?;
        
        let has_permission = permissions
            .iter()
            .any(|perm| perm.path.contains(table_name));

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                operation,
                &format!("database table '{}'", table_name),
            ));
        }

        Ok(())
    }

    /// Prüft Dateisystem-Berechtigungen
    pub async fn check_filesystem_permission(
        connection: &DbConnection,
        extension_id: &str,
        operation: &str,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_filesystem_permissions(connection, extension_id, operation).await?;
        
        let file_path_str = file_path.to_string_lossy();
        let has_permission = permissions.iter().any(|perm| {
            // Prüfe, ob der Pfad mit einem erlaubten Pfad beginnt oder übereinstimmt
            file_path_str.starts_with(&perm.path) || 
            // Oder ob es ein Wildcard-Match gibt
            Self::matches_path_pattern(&perm.path, &file_path_str)
        });

        if !has_permission {
            return Err(ExtensionError::permission_denied(
                extension_id,
                operation,
                &format!("filesystem path '{}'", file_path_str),
            ));
        }

        Ok(())
    }

    /// Prüft HTTP-Berechtigungen
    pub async fn check_http_permission(
        connection: &DbConnection,
        extension_id: &str,
        method: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_http_permissions(connection, extension_id, method).await?;
        
        let url_parsed = Url::parse(url).map_err(|e| {
            ExtensionError::ValidationError {
                reason: format!("Invalid URL: {}", e),
            }
        })?;

        let domain = url_parsed.host_str().unwrap_or("");
        let path = url_parsed.path();

        let has_permission = permissions.iter().any(|perm| {
            // Prüfe Domain
            let domain_matches = perm.domain == "*" || 
                               perm.domain == domain || 
                               domain.ends_with(&format!(".{}", perm.domain));

            // Prüfe Pfad (falls spezifiziert)
            let path_matches = perm.path_pattern.as_ref()
                .map(|pattern| Self::matches_path_pattern(pattern, path))
                .unwrap_or(true);

            domain_matches && path_matches
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
        connection: &DbConnection,
        extension_id: &str,
        command: &str,
        args: &[String],
    ) -> Result<(), ExtensionError> {
        let permissions = Self::get_shell_permissions(connection, extension_id).await?;
        
        let has_permission = permissions.iter().any(|perm| {
            // Prüfe Command
            if perm.command != command && perm.command != "*" {
                return false;
            }

            // Prüfe Arguments (falls spezifiziert)
            if !perm.arguments.is_empty() {
                // Alle erforderlichen Args müssen vorhanden sein
                perm.arguments.iter().all(|required_arg| {
                    args.iter().any(|actual_arg| {
                        required_arg == actual_arg || required_arg == "*"
                    })
                })
            } else {
                true
            }
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

    // Private Helper-Methoden

    async fn get_database_permissions(
        connection: &DbConnection,
        extension_id: &str,
        operation: &str,
    ) -> Result<Vec<DbExtensionPermission>, ExtensionError> {
        // Verwende die bestehende Funktion aus dem permissions.rs
        crate::extension::database::permissions::get_extension_permissions(
            connection, 
            extension_id, 
            "database", 
            operation
        ).await.map_err(ExtensionError::from)
    }

    async fn get_filesystem_permissions(
        connection: &DbConnection,
        extension_id: &str,
        operation: &str,
    ) -> Result<Vec<FilesystemPermission>, ExtensionError> {
        // Implementierung für Filesystem-Permissions
        // Ähnlich wie get_database_permissions, aber für filesystem Tabelle
        todo!("Implementiere Filesystem-Permission-Loading")
    }

    async fn get_http_permissions(
        connection: &DbConnection,
        extension_id: &str,
        method: &str,
    ) -> Result<Vec<HttpPermission>, ExtensionError> {
        // Implementierung für HTTP-Permissions
        todo!("Implementiere HTTP-Permission-Loading")
    }

    async fn get_shell_permissions(
        connection: &DbConnection,
        extension_id: &str,
    ) -> Result<Vec<ShellPermission>, ExtensionError> {
        // Implementierung für Shell-Permissions
        todo!("Implementiere Shell-Permission-Loading")
    }

    fn matches_path_pattern(pattern: &str, path: &str) -> bool {
        // Einfache Wildcard-Implementierung
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            path.starts_with(prefix)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            path.ends_with(suffix)
        } else {
            pattern == path
        }
    }
}

// Convenience-Funktionen für die verschiedenen Subsysteme
impl PermissionManager {
    /// Convenience für Datei lesen
    pub async fn can_read_file(
        connection: &DbConnection,
        extension_id: &str,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        Self::check_filesystem_permission(connection, extension_id, "read", file_path).await
    }

    /// Convenience für Datei schreiben
    pub async fn can_write_file(
        connection: &DbConnection,
        extension_id: &str,
        file_path: &Path,
    ) -> Result<(), ExtensionError> {
        Self::check_filesystem_permission(connection, extension_id, "write", file_path).await
    }

    /// Convenience für HTTP GET
    pub async fn can_http_get(
        connection: &DbConnection,
        extension_id: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_http_permission(connection, extension_id, "GET", url).await
    }

    /// Convenience für HTTP POST
    pub async fn can_http_post(
        connection: &DbConnection,
        extension_id: &str,
        url: &str,
    ) -> Result<(), ExtensionError> {
        Self::check_http_permission(connection, extension_id, "POST", url).await
    }

    /// Convenience für Shell-Befehl
    pub async fn can_execute_command(
        connection: &DbConnection,
        extension_id: &str,
        command: &str,
        args: &[String],
    ) -> Result<(), ExtensionError> {
        Self::check_shell_permission(connection, extension_id, command, args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_pattern_matching() {
        assert!(PermissionManager::matches_path_pattern("/home/user/*", "/home/user/documents/file.txt"));
        assert!(PermissionManager::matches_path_pattern("*.txt", "/path/to/file.txt"));
        assert!(PermissionManager::matches_path_pattern("/exact/path", "/exact/path"));
        
        assert!(!PermissionManager::matches_path_pattern("/home/user/*", "/etc/passwd"));
        assert!(!PermissionManager::matches_path_pattern("*.txt", "/path/to/file.pdf"));
    }
}