// src-tauri/src/extension/core/types.rs

use crate::extension::core::manifest::ExtensionManifest;
use std::path::PathBuf;
use std::time::SystemTime;

/// Extension source type (production vs development)
#[derive(Debug, Clone)]
pub enum ExtensionSource {
    Production {
        path: PathBuf,
        version: String,
    },
    Development {
        dev_server_url: String,
        manifest_path: PathBuf,
        auto_reload: bool,
    },
}

/// Complete extension data structure
#[derive(Debug, Clone)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub source: ExtensionSource,
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub last_accessed: SystemTime,
}

pub fn get_tauri_origin() -> String {
    #[cfg(target_os = "windows")]
    {
        "https://tauri.localhost".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "android")]
    {
        "tauri://localhost".to_string()
    }

    #[cfg(target_os = "ios")]
    {
        "tauri://localhost".to_string()
    }
}

pub fn copy_directory(
    source: String,
    destination: String,
) -> Result<(), crate::extension::error::ExtensionError> {
    use crate::extension::error::ExtensionError;
    use std::path::PathBuf;

    println!(
        "Kopiere Verzeichnis von '{}' nach '{}'",
        source, destination
    );

    let source_path = PathBuf::from(&source);
    let destination_path = PathBuf::from(&destination);

    if !source_path.exists() || !source_path.is_dir() {
        return Err(ExtensionError::Filesystem {
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory '{}' not found", source),
            ),
        });
    }

    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.copy_inside = true;
    options.buffer_size = 64000;

    fs_extra::dir::copy(&source_path, &destination_path, &options).map_err(|e| {
        ExtensionError::Filesystem {
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        }
    })?;
    Ok(())
}
