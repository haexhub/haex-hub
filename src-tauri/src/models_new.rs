// models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Arc};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub entry: String,
    pub icon: Option<String>,
    pub permissions: ExtensionPermissions,
    pub homepage: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtensionPermissions {
    pub database: Option<DatabasePermissions>,
    pub http: Option<Vec<String>>,
    pub filesystem: Option<FilesystemPermissions>,
    pub shell: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabasePermissions {
    pub read: Option<Vec<String>>,
    pub write: Option<Vec<String>>,
    pub create: Option<Vec<String>>,
}

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
    pub fn resolve_system_path(&self, app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, ExtensionError> {
        let base_dir = match self.path_type {
            FilesystemPathType::AppData => app_handle.path().app_data_dir(),
            FilesystemPathType::AppCache => app_handle.path().app_cache_dir(),
            FilesystemPathType::AppConfig => app_handle.path().app_config_dir(),
            FilesystemPathType::Documents => app_handle.path().document_dir(),
            FilesystemPathType::Pictures => app_handle.path().picture_dir(),
            FilesystemPathType::Downloads => app_handle.path().download_dir(),
            FilesystemPathType::Temp => app_handle.path().temp_dir(),
            FilesystemPathType::ExtensionData => {
                app_handle.path().app_data_dir().map(|p| p.join("extensions"))
            },
            FilesystemPathType::SharedData => {
                app_handle.path().app_data_dir().map(|p| p.join("shared"))
            },
        }.map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to resolve base directory: {}", e),
        })?;
        
        let final_path = base_dir.join(&self.relative_path);
        
        // Security check - ensure the resolved path is still within the base directory
        if let (Ok(canonical_final), Ok(canonical_base)) = (final_path.canonicalize(), base_dir.canonicalize()) {
            if !canonical_final.starts_with(&canonical_base) {
                return Err(ExtensionError::SecurityViolation {
                    reason: format!("Path traversal detected: {} escapes base directory", self.relative_path),
                });
            }
        }
        
        Ok(final_path)
    }
}

/// Enum to represent the source of an extension
#[derive(Debug, Clone)]
pub enum ExtensionSource {
    /// Production extension installed in app data
    Production { 
        path: PathBuf,
        version: String,
    },
    /// Development mode extension with live reloading
    Development { 
        dev_server_url: String,
        manifest_path: PathBuf,
        auto_reload: bool,
    },
}

/// Complete extension data including source and runtime status
#[derive(Debug, Clone)]
pub struct Extension {
    /// Unique extension ID
    pub id: String,
    /// Extension display name
    pub name: String,
    /// Source information (production or dev)
    pub source: ExtensionSource,
    /// Complete manifest data
    pub manifest: ExtensionManifest,
    /// Enabled status
    pub enabled: bool,
    /// Last access timestamp
    pub last_accessed: SystemTime,
}

/// Cached permission data to avoid frequent database lookups
#[derive(Debug, Clone)]
pub struct CachedPermission {
    /// The permissions that were fetched
    pub permissions: Vec<DbExtensionPermission>,
    /// When this cache entry was created
    pub cached_at: SystemTime,
    /// How long this cache entry is valid
    pub ttl: Duration,
}

/// Enhanced extension manager with production/dev support and caching
#[derive(Default)]
pub struct ExtensionManager {
    /// Production extensions loaded from app data directory
    pub production_extensions: Mutex<HashMap<String, Extension>>,
    /// Development mode extensions for live-reloading during development
    pub dev_extensions: Mutex<HashMap<String, Extension>>,
    /// Cache for extension permissions to improve performance
    pub permission_cache: Mutex<HashMap<String, CachedPermission>>,
}

impl ExtensionManager {
    /// Creates a new extension manager
    pub fn new() -> Self {
        Self {
            production_extensions: Mutex::new(HashMap::new()),
            dev_extensions: Mutex::new(HashMap::new()),
            permission_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Adds a production extension to the manager
    pub fn add_production_extension(&self, extension: Extension) -> Result<(), ExtensionError> {
        if extension.id.is_empty() {
            return Err(ExtensionError::ValidationError {
                reason: "Extension ID cannot be empty".to_string(),
            });
        }
        
        match &extension.source {
            ExtensionSource::Production { .. } => {
                let mut extensions = self.production_extensions.lock().unwrap();
                extensions.insert(extension.id.clone(), extension);
                Ok(())
            },
            _ => Err(ExtensionError::ValidationError {
                reason: "Expected Production source for production extension".to_string(),
            })
        }
    }

    /// Adds a development mode extension to the manager
    pub fn add_dev_extension(&self, extension: Extension) -> Result<(), ExtensionError> {
        if extension.id.is_empty() {
            return Err(ExtensionError::ValidationError {
                reason: "Extension ID cannot be empty".to_string(),
            });
        }
        
        match &extension.source {
            ExtensionSource::Development { .. } => {
                let mut extensions = self.dev_extensions.lock().unwrap();
                extensions.insert(extension.id.clone(), extension);
                Ok(())
            },
            _ => Err(ExtensionError::ValidationError {
                reason: "Expected Development source for dev extension".to_string(),
            })
        }
    }

    /// Gets an extension by its ID
    pub fn get_extension(&self, extension_id: &str) -> Option<Extension> {
        // First check development extensions (they take priority)
        let dev_extensions = self.dev_extensions.lock().unwrap();
        if let Some(extension) = dev_extensions.get(extension_id) {
            return Some(extension.clone());
        }
        
        // Then check production extensions
        let prod_extensions = self.production_extensions.lock().unwrap();
        prod_extensions.get(extension_id).cloned()
    }

    /// Removes an extension from the manager
    pub fn remove_extension(&self, extension_id: &str) -> Result<(), ExtensionError> {
        // Check dev extensions first
        {
            let mut dev_extensions = self.dev_extensions.lock().unwrap();
            if dev_extensions.remove(extension_id).is_some() {
                return Ok(());
            }
        }
        
        // Then check production extensions
        {
            let mut prod_extensions = self.production_extensions.lock().unwrap();
            if prod_extensions.remove(extension_id).is_some() {
                return Ok(());
            }
        }
        
        Err(ExtensionError::NotFound { 
            id: extension_id.to_string(),
        })
    }

    /// Gets cached permissions or indicates they need to be loaded
    pub fn get_cached_permissions(
        &self,
        extension_id: &str,
        resource: &str,
        operation: &str,
    ) -> Option<Vec<DbExtensionPermission>> {
        let cache = self.permission_cache.lock().unwrap();
        let cache_key = format!("{}-{}-{}", extension_id, resource, operation);
        
        if let Some(cached) = cache.get(&cache_key) {
            let now = SystemTime::now();
            if now.duration_since(cached.cached_at).unwrap_or(Duration::from_secs(0)) < cached.ttl {
                return Some(cached.permissions.clone());
            }
        }
        
        None
    }

    /// Updates the permission cache
    pub fn update_permission_cache(
        &self,
        extension_id: &str,
        resource: &str,
        operation: &str,
        permissions: Vec<DbExtensionPermission>,
    ) {
        let mut cache = self.permission_cache.lock().unwrap();
        let cache_key = format!("{}-{}-{}", extension_id, resource, operation);
        
        cache.insert(cache_key, CachedPermission {
            permissions,
            cached_at: SystemTime::now(),
            ttl: Duration::from_secs(60), // Cache for 60 seconds
        });
    }

    /// Validates a manifest for security concerns
    pub fn validate_manifest_security(&self, manifest: &ExtensionManifest) -> Result<(), ExtensionError> {
        // Check for suspicious permission combinations
        let has_filesystem = manifest.permissions.filesystem.is_some();
        let has_database = manifest.permissions.database.is_some();
        let has_shell = manifest.permissions.shell.is_some();
        
        if has_filesystem && has_database && has_shell {
            // This is a powerful combination, warn or check user confirmation elsewhere
        }
        
        // Validate ID format
        if !manifest.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(ExtensionError::ValidationError { 
                reason: "Invalid extension ID format. Must contain only alphanumeric characters, dash or underscore.".to_string() 
            });
        }
        
        Ok(())
    }

    /// Lists all enabled extensions (both dev and production)
    pub fn list_enabled_extensions(&self) -> Vec<Extension> {
        let mut extensions = Vec::new();
        
        // Add enabled dev extensions first (higher priority)
        {
            let dev_extensions = self.dev_extensions.lock().unwrap();
            extensions.extend(
                dev_extensions
                    .values()
                    .filter(|ext| ext.enabled)
                    .cloned()
            );
        }
        
        // Add enabled production extensions (avoiding duplicates)
        {
            let prod_extensions = self.production_extensions.lock().unwrap();
            let dev_ids: std::collections::HashSet<String> = extensions.iter().map(|e| e.id.clone()).collect();
            
            extensions.extend(
                prod_extensions
                    .values()
                    .filter(|ext| ext.enabled && !dev_ids.contains(&ext.id))
                    .cloned()
            );
        }
        
        extensions
    }
}

// For backward compatibility - will be deprecated
#[derive(Default)]
pub struct ExtensionState {
    pub extensions: Mutex<HashMap<String, ExtensionManifest>>,
}

impl ExtensionState {
    pub fn add_extension(&self, path: String, manifest: ExtensionManifest) {
        let mut extensions = self.extensions.lock().unwrap();
        extensions.insert(path, manifest);
    }

    pub fn get_extension(&self, addon_id: &str) -> Option<ExtensionManifest> {
        let extensions = self.extensions.lock().unwrap();
        extensions.values().find(|p| p.name == addon_id).cloned()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbExtensionPermission {
    pub id: String,
    pub extension_id: String,
    pub resource: String,
    pub operation: String,
    pub path: String,
}

/// Comprehensive error type for all extension-related operations
#[derive(Error, Debug)]
pub enum ExtensionError {
    /// Security violation detected
    #[error("Security violation: {reason}")]
    SecurityViolation { 
        reason: String 
    },
    
    /// Extension not found
    #[error("Extension not found: {id}")]
    NotFound { 
        id: String 
    },
    
    /// Permission denied
    #[error("Permission denied: {extension_id} cannot {operation} on {resource}")]
    PermissionDenied { 
        extension_id: String, 
        operation: String, 
        resource: String 
    },
    
    /// IO error during extension operations
    #[error("IO error: {source}")]
    Io { 
        #[from]
        source: std::io::Error 
    },
    
    /// Error during extension manifest parsing
    #[error("Manifest error: {reason}")]
    ManifestError { 
        reason: String 
    },
    
    /// Input validation error
    #[error("Validation error: {reason}")]
    ValidationError { 
        reason: String 
    },
    
    /// Development server error
    #[error("Dev server error: {reason}")]
    DevServerError { 
        reason: String 
    },
}

// For Tauri Command Serialization
impl serde::Serialize for ExtensionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}