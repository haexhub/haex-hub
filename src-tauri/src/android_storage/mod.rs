#[cfg(target_os = "android")]
#[tauri::command]
pub async fn request_storage_permission(app_handle: tauri::AppHandle) -> Result<String, String> {
    Ok("Settings opened - Enable 'Allow management of all files'".to_string())
    /* use tauri_plugin_opener::OpenerExt;

    // Korrekte Android Settings Intent
    let intent_uri = "android.settings.MANAGE_ALL_FILES_ACCESS_PERMISSION";

    match app.opener().open_url(intent_uri, None::<&str>) {
        Ok(_) => Ok("Settings opened - Enable 'Allow management of all files'".to_string()),
        Err(_) => {
            // Fallback: App-spezifische Settings
            let app_settings = format!(
                "android.settings.APPLICATION_DETAILS_SETTINGS?package={}",
                app.config().identifier
            );
            match app.opener().open_url(&app_settings, None::<&str>) {
                Ok(_) => Ok("App settings opened - Go to Permissions > Files and media".to_string()),
                Err(_) => Ok("Manually go to: Settings > Apps > Special app access > All files access > HaexHub > Allow".to_string())
            }
        }
    }*/
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn has_storage_permission() -> Result<bool, String> {
    use std::path::Path;

    // Teste Schreibzugriff auf externen Speicher
    let test_paths = [
        "/storage/emulated/0/Android",
        "/sdcard/Android",
        "/storage/emulated/0",
    ];

    for path in &test_paths {
        if Path::new(path).exists() {
            // Versuche Testdatei zu erstellen
            let test_file = format!("{}/haex_test.tmp", path);
            match std::fs::write(&test_file, "test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    return Ok(true);
                }
                Err(_) => continue,
            }
        }
    }

    Ok(false)
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn get_external_storage_paths() -> Result<Vec<String>, String> {
    let mut paths = Vec::new();

    let common_paths = [
        "/storage/emulated/0",
        "/sdcard",
        "/storage/emulated/0/Download",
        "/storage/emulated/0/Documents",
        "/storage/emulated/0/Pictures",
        "/storage/emulated/0/DCIM",
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            paths.push(path.to_string());
        }
    }

    Ok(paths)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn request_storage_permission(_app: tauri::AppHandle) -> Result<String, String> {
    Ok("aaaaaaaa".to_string())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn has_storage_permission() -> Result<bool, String> {
    Ok(true)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn get_external_storage_paths() -> Result<Vec<String>, String> {
    Ok(vec![])
}
