use crate::extension::database::{extension_sql_execute, extension_sql_select};
use crate::extension::error::ExtensionError;
use crate::AppState;
use tauri::{State, WebviewWindow};

use super::helpers::get_extension_id;

#[tauri::command]
pub async fn webview_extension_db_query(
    window: WebviewWindow,
    state: State<'_, AppState>,
    query: String,
    params: Vec<serde_json::Value>,
) -> Result<serde_json::Value, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    // Get extension to retrieve public_key and name for existing database functions
    let extension = state
        .extension_manager
        .get_extension(&extension_id)
        .ok_or_else(|| ExtensionError::ValidationError {
            reason: format!("Extension with ID {} not found", extension_id),
        })?;

    let rows = extension_sql_select(&query, params, extension.manifest.public_key.clone(), extension.manifest.name.clone(), state)
        .await
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Database query failed: {}", e),
        })?;

    Ok(serde_json::json!({
        "rows": rows,
        "rowsAffected": 0,
        "lastInsertId": null
    }))
}

#[tauri::command]
pub async fn webview_extension_db_execute(
    window: WebviewWindow,
    state: State<'_, AppState>,
    query: String,
    params: Vec<serde_json::Value>,
) -> Result<serde_json::Value, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    // Get extension to retrieve public_key and name for existing database functions
    let extension = state
        .extension_manager
        .get_extension(&extension_id)
        .ok_or_else(|| ExtensionError::ValidationError {
            reason: format!("Extension with ID {} not found", extension_id),
        })?;

    let rows = extension_sql_execute(&query, params, extension.manifest.public_key.clone(), extension.manifest.name.clone(), state)
        .await
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Database execute failed: {}", e),
        })?;

    Ok(serde_json::json!({
        "rows": rows,
        "rowsAffected": rows.len(),
        "lastInsertId": null
    }))
}
