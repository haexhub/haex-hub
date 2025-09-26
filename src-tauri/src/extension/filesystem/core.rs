use serde::{Deserialize, Serialize};

use crate::extension::error::ExtensionError;

/// Simple filesystem permissions using path patterns with environment-style variables
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilesystemPermissions {
    /// Read access to files and directories
    /// Examples: ["$DOCUMENT/**", "$PICTURE/*.jpg", "$APPDATA/my-extension/*"]
    pub read: Option<Vec<String>>,
    /// Write access to files and directories (includes create/delete)
    /// Examples: ["$APPDATA/my-extension/**", "$DOWNLOAD/*.pdf"]
    pub write: Option<Vec<String>>,
}

impl FilesystemPermissions {
    /// Helper to create common permission patterns
    pub fn new() -> Self {
        Self {
            read: None,
            write: None,
        }
    }

    /// Add read permission for a path pattern
    pub fn add_read(&mut self, pattern: &str) {
        match &mut self.read {
            Some(patterns) => patterns.push(pattern.to_string()),
            None => self.read = Some(vec![pattern.to_string()]),
        }
    }

    /// Add write permission for a path pattern
    pub fn add_write(&mut self, pattern: &str) {
        match &mut self.write {
            Some(patterns) => patterns.push(pattern.to_string()),
            None => self.write = Some(vec![pattern.to_string()]),
        }
    }

    /// Helper: Add extension's own data directory permissions
    pub fn extension_data(extension_id: &str) -> Self {
        Self {
            read: Some(vec![format!("$APPDATA/extensions/{}/**", extension_id)]),
            write: Some(vec![format!("$APPDATA/extensions/{}/**", extension_id)]),
        }
    }

    /// Helper: Add document access permissions
    pub fn documents_read_only() -> Self {
        Self {
            read: Some(vec!["$DOCUMENT/**".to_string()]),
            write: None,
        }
    }

    /// Helper: Add picture access permissions
    pub fn pictures_read_only() -> Self {
        Self {
            read: Some(vec!["$PICTURE/**".to_string()]),
            write: None,
        }
    }

    /// Validate all path patterns
    pub fn validate(&self) -> Result<(), ExtensionError> {
        if let Some(read_patterns) = &self.read {
            for pattern in read_patterns {
                validate_path_pattern(pattern)?;
            }
        }

        if let Some(write_patterns) = &self.write {
            for pattern in write_patterns {
                validate_path_pattern(pattern)?;
            }
        }

        Ok(())
    }
}

/// Validates a filesystem path pattern
fn validate_path_pattern(pattern: &str) -> Result<(), ExtensionError> {
    if pattern.is_empty() {
        return Err(ExtensionError::ValidationError {
            reason: "Path pattern cannot be empty".to_string(),
        });
    }

    // Check if pattern starts with valid base directory variable
    let valid_bases = [
        "$APPDATA",
        "$APPCACHE",
        "$APPCONFIG",
        "$APPLOCALDATA",
        "$APPLOG",
        "$AUDIO",
        "$CACHE",
        "$CONFIG",
        "$DATA",
        "$LOCALDATA",
        "$DESKTOP",
        "$DOCUMENT",
        "$DOWNLOAD",
        "$EXECUTABLE",
        "$FONT",
        "$HOME",
        "$PICTURE",
        "$PUBLIC",
        "$RUNTIME",
        "$TEMPLATE",
        "$VIDEO",
        "$RESOURCE",
        "$TEMP",
    ];

    let starts_with_valid_base = valid_bases.iter().any(|&base| {
        pattern.starts_with(base)
            && (pattern.len() == base.len() || pattern.chars().nth(base.len()) == Some('/'))
    });

    if !starts_with_valid_base {
        return Err(ExtensionError::ValidationError {
            reason: format!(
                "Path pattern '{}' must start with a valid base directory: {}",
                pattern,
                valid_bases.join(", ")
            ),
        });
    }

    // Check for path traversal attempts
    if pattern.contains("../") || pattern.contains("..\\") {
        return Err(ExtensionError::SecurityViolation {
            reason: format!("Path traversal detected in pattern: {}", pattern),
        });
    }

    Ok(())
}

/// Resolves a path pattern to actual filesystem paths using Tauri's BaseDirectory
pub fn resolve_path_pattern(
    pattern: &str,
    app_handle: &tauri::AppHandle,
) -> Result<(String, String), ExtensionError> {
    let (base_var, relative_path) = if let Some(slash_pos) = pattern.find('/') {
        (&pattern[..slash_pos], &pattern[slash_pos + 1..])
    } else {
        (pattern, "")
    };

    let base_directory = match base_var {
        "$APPDATA" => "AppData",
        "$APPCACHE" => "AppCache",
        "$APPCONFIG" => "AppConfig",
        "$APPLOCALDATA" => "AppLocalData",
        "$APPLOG" => "AppLog",
        "$AUDIO" => "Audio",
        "$CACHE" => "Cache",
        "$CONFIG" => "Config",
        "$DATA" => "Data",
        "$LOCALDATA" => "LocalData",
        "$DESKTOP" => "Desktop",
        "$DOCUMENT" => "Document",
        "$DOWNLOAD" => "Download",
        "$EXECUTABLE" => "Executable",
        "$FONT" => "Font",
        "$HOME" => "Home",
        "$PICTURE" => "Picture",
        "$PUBLIC" => "Public",
        "$RUNTIME" => "Runtime",
        "$TEMPLATE" => "Template",
        "$VIDEO" => "Video",
        "$RESOURCE" => "Resource",
        "$TEMP" => "Temp",
        _ => {
            return Err(ExtensionError::ValidationError {
                reason: format!("Unknown base directory variable: {}", base_var),
            });
        }
    };

    Ok((base_directory.to_string(), relative_path.to_string()))
}
