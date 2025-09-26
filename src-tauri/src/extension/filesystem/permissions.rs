use serde::{Deserialize, Serialize};

use crate::extension::error::ExtensionError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilesystemPermissions {
    /// Read access to files and directories
    pub read: Option<Vec<FilesystemPath>>,
    /// Write access to files and directories (includes create/delete)
    pub write: Option<Vec<FilesystemPath>>,
}

/// Cross-platform filesystem path specification
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilesystemPath {
    /// The type of path (determines base directory)
    pub path_type: FilesystemPathType,
    /// Relative path from the base directory
    pub relative_path: String,
    /// Whether subdirectories are included (recursive)
    pub recursive: bool,
}

/// Platform-agnostic path types that map to appropriate directories on each OS
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FilesystemPathType {
    /// App's data directory ($APPDATA on Windows, ~/.local/share on Linux, etc.)
    AppData,
    /// App's cache directory
    AppCache,
    /// App's configuration directory
    AppConfig,
    /// User's documents directory
    Documents,
    /// User's pictures directory
    Pictures,
    /// User's downloads directory
    Downloads,
    /// Temporary directory
    Temp,
    /// Extension's own private directory (always allowed)
    ExtensionData,
    /// Shared data between extensions (requires special permission)
    SharedData,
}

impl FilesystemPath {
    /// Creates a new filesystem path specification
    pub fn new(path_type: FilesystemPathType, relative_path: &str, recursive: bool) -> Self {
        Self {
            path_type,
            relative_path: relative_path.to_string(),
            recursive,
        }
    }

    /// Resolves the path to an actual system path
    /// This would be implemented in your Tauri backend
    pub fn resolve_system_path(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> Result<String, ExtensionError> {
        /* let base_dir = match self.path_type {
            FilesystemPathType::AppData => app_handle.path().app_data_dir(),
            FilesystemPathType::AppCache => app_handle.path().app_cache_dir(),
            FilesystemPathType::AppConfig => app_handle.path().app_config_dir(),
            FilesystemPathType::Documents => app_handle.path().document_dir(),
            FilesystemPathType::Pictures => app_handle.path().picture_dir(),
            FilesystemPathType::Downloads => app_handle.path().download_dir(),
            FilesystemPathType::Temp => app_handle.path().temp_dir(),
            FilesystemPathType::ExtensionData => app_handle
                .path()
                .app_data_dir()
                .map(|p| p.join("extensions")),
            FilesystemPathType::SharedData => {
                app_handle.path().app_data_dir().map(|p| p.join("shared"))
            }
        }
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to resolve base directory: {}", e),
        })?;

        let final_path = base_dir.join(&self.relative_path);

        // Security check - ensure the resolved path is still within the base directory
        if let (Ok(canonical_final), Ok(canonical_base)) =
            (final_path.canonicalize(), base_dir.canonicalize())
        {
            if !canonical_final.starts_with(&canonical_base) {
                return Err(ExtensionError::SecurityViolation {
                    reason: format!(
                        "Path traversal detected: {} escapes base directory",
                        self.relative_path
                    ),
                });
            }
        } */

        Ok("".to_string())
    }
}
