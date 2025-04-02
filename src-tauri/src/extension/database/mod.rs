mod permissions;
use crate::database;
use crate::database::DbConnection;
//use crate::models::ExtensionState;
use tauri::{AppHandle, State};

// Extension-bezogene Funktionen mit extension_-Präfix
/// Lädt eine Extension aus einer Manifest-Datei
/* #[tauri::command]
pub fn extension_load(
    manifest_path: String,
    app: AppHandle,
) -> Result<crate::models::ExtensionManifest, String> {
    let manifest_content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let manifest: crate::models::ExtensionManifest =
        serde_json::from_str(&manifest_content).map_err(|e| e.to_string())?;
    app.state::<ExtensionState>()
        .add_extension(manifest_path.clone(), manifest.clone());
    Ok(manifest)
} */

/// Führt SQL-Leseoperationen mit Berechtigungsprüfung aus
#[tauri::command]
pub async fn extension_sql_select(
    app: AppHandle,
    extension_id: String,
    sql: String,
    params: Vec<String>,
    state: State<'_, DbConnection>,
) -> Result<Vec<Vec<String>>, String> {
    permissions::check_read_permission(&app, &extension_id, &sql).await?;
    database::core::select(&sql, &params, &state).await
}

/// Führt SQL-Schreiboperationen mit Berechtigungsprüfung aus
#[tauri::command]
pub async fn extension_sql_execute(
    app: AppHandle,
    extension_id: String,
    sql: String,
    params: Vec<String>,
    state: State<'_, DbConnection>,
) -> Result<String, String> {
    permissions::check_write_permission(&app, &extension_id, &sql).await?;
    database::core::execute(&sql, &params, &state).await
}
