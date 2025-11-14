use crate::extension::core::protocol::ExtensionInfo;
use crate::extension::error::ExtensionError;
use crate::extension::permissions::manager::PermissionManager;
use crate::extension::permissions::types::{Action, DbAction, FsAction};
use crate::AppState;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};
use tauri_plugin_http::reqwest;

use super::helpers::{get_extension_id, get_extension_info_from_window};

// ============================================================================
// Types for SDK communication
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationContext {
    pub theme: String,
    pub locale: String,
    pub platform: String,
}

// ============================================================================
// Extension Info Command
// ============================================================================

#[tauri::command]
pub fn webview_extension_get_info(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<ExtensionInfo, ExtensionError> {
    get_extension_info_from_window(&window, &state)
}

// ============================================================================
// Context API Commands
// ============================================================================

#[tauri::command]
pub fn webview_extension_context_get(
    state: State<'_, AppState>,
) -> Result<ApplicationContext, ExtensionError> {
    let context = state.context.lock().map_err(|e| ExtensionError::ValidationError {
        reason: format!("Failed to lock context: {}", e),
    })?;
    Ok(context.clone())
}

#[tauri::command]
pub fn webview_extension_context_set(
    state: State<'_, AppState>,
    context: ApplicationContext,
) -> Result<(), ExtensionError> {
    let mut current_context = state.context.lock().map_err(|e| ExtensionError::ValidationError {
        reason: format!("Failed to lock context: {}", e),
    })?;
    *current_context = context;
    Ok(())
}

// ============================================================================
// Permission API Commands
// ============================================================================

#[tauri::command]
pub async fn webview_extension_check_web_permission(
    window: WebviewWindow,
    state: State<'_, AppState>,
    url: String,
) -> Result<bool, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    match PermissionManager::check_web_permission(&state, &extension_id, &url).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn webview_extension_check_database_permission(
    window: WebviewWindow,
    state: State<'_, AppState>,
    resource: String,
    operation: String,
) -> Result<bool, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    let action = match operation.as_str() {
        "read" => Action::Database(DbAction::Read),
        "write" => Action::Database(DbAction::ReadWrite),
        _ => return Ok(false),
    };

    match PermissionManager::check_database_permission(&state, &extension_id, action, &resource).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn webview_extension_check_filesystem_permission(
    window: WebviewWindow,
    state: State<'_, AppState>,
    path: String,
    action_str: String,
) -> Result<bool, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    let action = match action_str.as_str() {
        "read" => Action::Filesystem(FsAction::Read),
        "write" => Action::Filesystem(FsAction::ReadWrite),
        _ => return Ok(false),
    };

    let path_buf = std::path::Path::new(&path);
    match PermissionManager::check_filesystem_permission(&state, &extension_id, action, path_buf).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ============================================================================
// Web API Commands
// ============================================================================

#[tauri::command]
pub async fn webview_extension_web_open(
    window: WebviewWindow,
    state: State<'_, AppState>,
    url: String,
) -> Result<(), ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    // Validate URL format
    let parsed_url = url::Url::parse(&url).map_err(|e| ExtensionError::WebError {
        reason: format!("Invalid URL: {}", e),
    })?;

    // Only allow http and https URLs
    let scheme = parsed_url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ExtensionError::WebError {
            reason: format!("Unsupported URL scheme: {}. Only http and https are allowed.", scheme),
        });
    }

    // Check web permissions
    PermissionManager::check_web_permission(&state, &extension_id, &url).await?;

    // Open URL in default browser using tauri-plugin-opener
    tauri_plugin_opener::open_url(&url, None::<&str>).map_err(|e| ExtensionError::WebError {
        reason: format!("Failed to open URL in browser: {}", e),
    })?;

    Ok(())
}

#[tauri::command]
pub async fn webview_extension_web_request(
    window: WebviewWindow,
    state: State<'_, AppState>,
    url: String,
    method: Option<String>,
    headers: Option<serde_json::Value>,
    body: Option<String>,
) -> Result<serde_json::Value, ExtensionError> {
    let extension_id = get_extension_id(&window, &state)?;

    // Check permission first
    PermissionManager::check_web_permission(&state, &extension_id, &url).await?;

    // Build request
    let method = method.unwrap_or_else(|| "GET".to_string());

    let client = reqwest::Client::new();
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => {
            return Err(ExtensionError::ValidationError {
                reason: format!("Unsupported HTTP method: {}", method),
            })
        }
    };

    // Add headers
    if let Some(headers) = headers {
        if let Some(headers_obj) = headers.as_object() {
            for (key, value) in headers_obj {
                if let Some(value_str) = value.as_str() {
                    request = request.header(key, value_str);
                }
            }
        }
    }

    // Add body
    if let Some(body) = body {
        request = request.body(body);
    }

    // Execute request
    let response = request
        .send()
        .await
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("HTTP request failed: {}", e),
        })?;

    let status = response.status().as_u16();
    let headers_map = response.headers().clone();

    // Get response body as bytes
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| ExtensionError::ValidationError {
            reason: format!("Failed to read response body: {}", e),
        })?;

    // Encode body as base64
    let body_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &body_bytes);

    // Convert headers to JSON
    let mut headers_json = serde_json::Map::new();
    for (key, value) in headers_map.iter() {
        if let Ok(value_str) = value.to_str() {
            headers_json.insert(
                key.to_string(),
                serde_json::Value::String(value_str.to_string()),
            );
        }
    }

    Ok(serde_json::json!({
        "status": status,
        "headers": headers_json,
        "body": body_base64,
        "ok": status >= 200 && status < 300
    }))
}
