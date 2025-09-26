/// src-tauri/src/extension/error.rs
use thiserror::Error;

use crate::database::error::DatabaseError;

/// Comprehensive error type for extension operations
#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Extension not found: {id}")]
    NotFound { id: String },

    #[error("Permission denied: {extension_id} cannot {operation} on {resource}")]
    PermissionDenied {
        extension_id: String,
        operation: String,
        resource: String,
    },

    #[error("Database operation failed: {source}")]
    Database {
        #[from]
        source: DatabaseError,
    },

    #[error("Filesystem operation failed: {source}")]
    Filesystem {
        #[from]
        source: std::io::Error,
        // oder: source: FilesystemError,
    },

    #[error("HTTP request failed: {reason}")]
    Http {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Shell command failed: {reason}")]
    Shell {
        reason: String,
        exit_code: Option<i32>,
    },

    /* #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    }, */
    #[error("Manifest error: {reason}")]
    ManifestError { reason: String },

    #[error("Validation error: {reason}")]
    ValidationError { reason: String },

    #[error("Dev server error: {reason}")]
    DevServerError { reason: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Configuration error: {reason}")]
    ConfigError { reason: String },
}

impl ExtensionError {
    /// Convenience constructor for permission denied errors
    pub fn permission_denied(extension_id: &str, operation: &str, resource: &str) -> Self {
        Self::PermissionDenied {
            extension_id: extension_id.to_string(),
            operation: operation.to_string(),
            resource: resource.to_string(),
        }
    }

    /// Convenience constructor for HTTP errors
    pub fn http_error(reason: &str) -> Self {
        Self::Http {
            reason: reason.to_string(),
            source: None,
        }
    }

    /// Convenience constructor for HTTP errors with source
    pub fn http_error_with_source(
        reason: &str,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Http {
            reason: reason.to_string(),
            source: Some(source),
        }
    }

    /// Convenience constructor for shell errors
    pub fn shell_error(reason: &str, exit_code: Option<i32>) -> Self {
        Self::Shell {
            reason: reason.to_string(),
            exit_code,
        }
    }

    /// Check if this error is related to permissions
    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            ExtensionError::PermissionDenied { .. } | ExtensionError::SecurityViolation { .. }
        )
    }

    /// Extract extension ID if available
    pub fn extension_id(&self) -> Option<&str> {
        match self {
            ExtensionError::PermissionDenied { extension_id, .. } => Some(extension_id),
            ExtensionError::Database { source } => source.extension_id(),
            _ => None,
        }
    }
}

impl serde::Serialize for ExtensionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ExtensionError", 3)?;

        // Error type as discriminator
        let error_type = match self {
            ExtensionError::SecurityViolation { .. } => "SecurityViolation",
            ExtensionError::NotFound { .. } => "NotFound",
            ExtensionError::PermissionDenied { .. } => "PermissionDenied",
            ExtensionError::Database { .. } => "Database",
            ExtensionError::Filesystem { .. } => "Filesystem",
            ExtensionError::Http { .. } => "Http",
            ExtensionError::Shell { .. } => "Shell",
            //ExtensionError::Io { .. } => "Io",
            ExtensionError::ManifestError { .. } => "ManifestError",
            ExtensionError::ValidationError { .. } => "ValidationError",
            ExtensionError::DevServerError { .. } => "DevServerError",
            ExtensionError::SerializationError { .. } => "SerializationError",
            ExtensionError::ConfigError { .. } => "ConfigError",
        };

        state.serialize_field("type", error_type)?;
        state.serialize_field("message", &self.to_string())?;

        // Add extension_id if available
        if let Some(ext_id) = self.extension_id() {
            state.serialize_field("extension_id", ext_id)?;
        } else {
            state.serialize_field("extension_id", &Option::<String>::None)?;
        }

        state.end()
    }
}

// For Tauri command serialization
impl From<serde_json::Error> for ExtensionError {
    fn from(err: serde_json::Error) -> Self {
        ExtensionError::SerializationError {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::error::DatabaseError;

    /* #[test]
    fn test_database_error_conversion() {
        let db_error = DatabaseError::access_denied("ext1", "read", "users", "no permission");
        let ext_error: ExtensionError = db_error.into();

        assert!(ext_error.is_permission_error());
        assert_eq!(ext_error.extension_id(), Some("ext1"));
    } */

    #[test]
    fn test_permission_denied_constructor() {
        let error = ExtensionError::permission_denied("ext1", "write", "config.json");

        match error {
            ExtensionError::PermissionDenied {
                extension_id,
                operation,
                resource,
            } => {
                assert_eq!(extension_id, "ext1");
                assert_eq!(operation, "write");
                assert_eq!(resource, "config.json");
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }

    #[test]
    fn test_serialization() {
        let error = ExtensionError::permission_denied("ext1", "read", "database");
        let serialized = serde_json::to_string(&error).unwrap();

        // Basic check that it serializes properly
        assert!(serialized.contains("PermissionDenied"));
        assert!(serialized.contains("ext1"));
    }
}
