// src-tauri/src/extension/permissions/check.rs

use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::AppState;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub async fn check_web_permission(
    extension_id: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    PermissionManager::check_web_permission(&state, &extension_id, &url).await
}

#[tauri::command]
pub async fn check_database_permission(
    extension_id: String,
    resource: String,
    operation: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    let action = match operation.as_str() {
        "read" => crate::extension::permissions::types::Action::Database(
            crate::extension::permissions::types::DbAction::Read,
        ),
        "write" => crate::extension::permissions::types::Action::Database(
            crate::extension::permissions::types::DbAction::ReadWrite,
        ),
        _ => {
            return Err(ExtensionError::ValidationError {
                reason: format!("Invalid database operation: {}", operation),
            })
        }
    };

    PermissionManager::check_database_permission(&state, &extension_id, action, &resource).await
}

#[tauri::command]
pub async fn check_filesystem_permission(
    extension_id: String,
    path: String,
    operation: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    let action = match operation.as_str() {
        "read" => crate::extension::permissions::types::Action::Filesystem(
            crate::extension::permissions::types::FsAction::Read,
        ),
        "write" => crate::extension::permissions::types::Action::Filesystem(
            crate::extension::permissions::types::FsAction::ReadWrite,
        ),
        _ => {
            return Err(ExtensionError::ValidationError {
                reason: format!("Invalid filesystem operation: {}", operation),
            })
        }
    };

    let file_path = Path::new(&path);
    PermissionManager::check_filesystem_permission(&state, &extension_id, action, file_path).await
}
