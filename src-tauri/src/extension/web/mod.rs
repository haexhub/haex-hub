// src-tauri/src/extension/web/mod.rs

use crate::extension::error::ExtensionError;
use crate::AppState;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;
use tauri_plugin_http::reqwest;

/// Request structure matching the SDK's WebRequestOptions
#[derive(Debug, Deserialize)]
pub struct WebFetchRequest {
    pub url: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<String>, // Base64 encoded
    #[serde(default)]
    pub timeout: Option<u64>, // milliseconds
}

/// Response structure matching the SDK's WebResponse
#[derive(Debug, Serialize)]
pub struct WebFetchResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String, // Base64 encoded
    pub url: String,
}

#[tauri::command]
pub async fn extension_web_open(
    url: String,
    public_key: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), ExtensionError> {
    // Get extension to validate it exists
    let extension = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
        .ok_or_else(|| ExtensionError::NotFound {
            public_key: public_key.clone(),
            name: name.clone(),
        })?;

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
    crate::extension::permissions::manager::PermissionManager::check_web_permission(
        &state,
        &extension.id,
        &url,
    )
    .await?;

    // Open URL in default browser using tauri-plugin-opener
    tauri_plugin_opener::open_url(&url, None::<&str>).map_err(|e| ExtensionError::WebError {
        reason: format!("Failed to open URL in browser: {}", e),
    })?;

    Ok(())
}

#[tauri::command]
pub async fn extension_web_fetch(
    url: String,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    timeout: Option<u64>,
    public_key: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<WebFetchResponse, ExtensionError> {
    // Get extension to validate it exists
    let extension = state
        .extension_manager
        .get_extension_by_public_key_and_name(&public_key, &name)?
        .ok_or_else(|| ExtensionError::NotFound {
            public_key: public_key.clone(),
            name: name.clone(),
        })?;

    let method_str = method.as_deref().unwrap_or("GET");

    // Check web permissions before making request
    crate::extension::permissions::manager::PermissionManager::check_web_permission(
        &state,
        &extension.id,
        &url,
    )
    .await?;

    let request = WebFetchRequest {
        url,
        method: Some(method_str.to_string()),
        headers,
        body,
        timeout,
    };

    fetch_web_request(request).await
}

/// Performs the actual HTTP request without CORS restrictions
async fn fetch_web_request(request: WebFetchRequest) -> Result<WebFetchResponse, ExtensionError> {
    let method_str = request.method.as_deref().unwrap_or("GET");
    let timeout_ms = request.timeout.unwrap_or(30000);

    // Build reqwest client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| ExtensionError::WebError {
            reason: format!("Failed to create HTTP client: {}", e),
        })?;

    // Build request
    let mut req_builder = match method_str.to_uppercase().as_str() {
        "GET" => client.get(&request.url),
        "POST" => client.post(&request.url),
        "PUT" => client.put(&request.url),
        "DELETE" => client.delete(&request.url),
        "PATCH" => client.patch(&request.url),
        "HEAD" => client.head(&request.url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &request.url),
        _ => {
            return Err(ExtensionError::WebError {
                reason: format!("Unsupported HTTP method: {}", method_str),
            })
        }
    };

    // Add headers
    if let Some(headers) = request.headers {
        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }
    }

    // Add body if present (decode from base64)
    if let Some(body_base64) = request.body {
        let body_bytes = STANDARD.decode(&body_base64).map_err(|e| {
            ExtensionError::WebError {
                reason: format!("Failed to decode request body from base64: {}", e),
            }
        })?;
        req_builder = req_builder.body(body_bytes);
    }

    // Execute request
    let response = req_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            ExtensionError::WebError {
                reason: format!("Request timeout after {}ms", timeout_ms),
            }
        } else {
            ExtensionError::WebError {
                reason: format!("Request failed: {}", e),
            }
        }
    })?;

    // Extract response data
    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("").to_string();
    let final_url = response.url().to_string();

    // Extract headers
    let mut response_headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(key.to_string(), value_str.to_string());
        }
    }

    // Read body and encode to base64
    let body_bytes = response.bytes().await.map_err(|e| ExtensionError::WebError {
        reason: format!("Failed to read response body: {}", e),
    })?;

    let body_base64 = STANDARD.encode(&body_bytes);

    Ok(WebFetchResponse {
        status,
        status_text,
        headers: response_headers,
        body: body_base64,
        url: final_url,
    })
}
