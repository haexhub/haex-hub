use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

mod manager;

#[derive(Serialize, Deserialize)]
pub struct TabInfo {
    id: String,
    title: String,
    url: String,
    is_loading: bool,
    is_active: bool,
}

// Einfache Kommandos für die Tab-Verwaltung
#[tauri::command]
pub fn create_tab(app_handle: tauri::AppHandle, tab_id: String, url: String) -> Result<(), String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or("Hauptfenster nicht gefunden")?;
    let window_size = main_window.inner_size().map_err(|e| e.to_string())?;

    // Erstelle eine neue Webview als eigenständiges Fenster
    let webview = tauri::WebviewWindowBuilder::new(
        &app_handle,
        tab_id.clone(),
        tauri::WebviewUrl::External(url.parse::<tauri::Url>().map_err(|e| e.to_string())?),
        //tauri::WebviewUrl::External("http://google.de"),
    )
    .title(format!("Tab: {}", tab_id))
    .inner_size(window_size.width as f64, window_size.height as f64 - 50.0)
    .position(0.0, 50.0)
    .build()
    .map_err(|e| e.to_string())?;

    // Sende die Tab-ID zurück an das Hauptfenster
    /* main_window
    .emit("tab-created", tab_id)
    .map_err(|e| e.to_string())?; */

    Ok(())
}

#[tauri::command]
pub fn show_tab(app_handle: tauri::AppHandle, tab_id: String) -> Result<(), String> {
    // Hole alle Webview-Fenster
    let windows = app_handle.webview_windows();

    // Zeige das ausgewählte Tab und verstecke die anderen
    for (id, window) in windows {
        if id != "main" {
            // Hauptfenster nicht verstecken
            if id == tab_id {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
            } else {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn close_tab(app_handle: tauri::AppHandle, tab_id: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&tab_id) {
        window.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

/* #[tauri::command]
pub fn create_tab(app: AppHandle, url: String) -> Result<TabInfo, String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.create_tab(&app, &url) {
        Ok(tab) => {
            let active_tab_id = browser_manager.get_active_tab_id();
            let is_active = active_tab_id.as_ref().map_or(false, |id| id == &tab.id);

            let main = app.get_webview_window("main");

            //main.unwrap().
            // Sende Event an Frontend
            /* app.emit_all(
                "tab-created",
                TabInfo {
                    id: tab.id.clone(),
                    title: tab.title.clone(),
                    url: tab.url.clone(),
                    is_loading: tab.is_loading,
                    is_active,
                },
            )
            .unwrap(); */

            Ok(TabInfo {
                id: tab.id,
                title: tab.title,
                url: tab.url,
                is_loading: tab.is_loading,
                is_active: true,
            })
        }
        Err(e) => Err(format!("Fehler beim Erstellen des Tabs: {}", e)),
    }
} */

/* #[tauri::command]
pub fn close_tab(app: AppHandle, tab_id: String) -> Result<(), String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.close_tab(&app, &tab_id) {
        Ok(_) => {
            // Sende Event an Frontend
            //app.emit_all("tab-closed", tab_id).unwrap();
            Ok(())
        }
        Err(e) => Err(format!("Fehler beim Schließen des Tabs: {}", e)),
    }
} */

#[tauri::command]
pub fn navigate_to_url(app: AppHandle, tab_id: String, url: String) -> Result<(), String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.navigate_to_url(&app, &tab_id, &url) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Fehler bei der Navigation: {}", e)),
    }
}

#[tauri::command]
pub fn get_current_url(app: AppHandle, tab_id: String) -> Result<String, String> {
    let browser_manager = app.state::<manager::BrowserManager>();
    let tabs = browser_manager.get_all_tabs();

    for tab in tabs {
        if tab.id == tab_id {
            return Ok(tab.url);
        }
    }

    Err("Tab nicht gefunden".to_string())
}

#[tauri::command]
pub fn go_back(app: AppHandle, tab_id: String) -> Result<(), String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.go_back(&app, &tab_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Fehler beim Zurückgehen: {}", e)),
    }
}

#[tauri::command]
pub fn go_forward(app: AppHandle, tab_id: String) -> Result<(), String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.go_forward(&app, &tab_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Fehler beim Vorwärtsgehen: {}", e)),
    }
}

#[tauri::command]
pub fn block_resource_request(url: String, resource_type: String) -> bool {
    // Diese Funktion wird vom Frontend aufgerufen, um zu prüfen, ob eine Ressource blockiert werden soll
    // Die eigentliche Logik wird im JavaScript-Erweiterungssystem implementiert
    // Hier könnten Sie zusätzliche Rust-seitige Prüfungen durchführen
    println!("Prüfe Ressourcenanfrage: {} (Typ: {})", url, resource_type);

    // Einfache Prüfung für Beispielzwecke
    url.contains("ads") || url.contains("analytics") || url.contains("tracker")
}

#[tauri::command]
pub fn inject_content_script(app: AppHandle, tab_id: String, script: String) -> Result<(), String> {
    let browser_manager = app.state::<manager::BrowserManager>();

    match browser_manager.inject_content_script(&app, &tab_id, &script) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Fehler beim Injizieren des Scripts: {}", e)),
    }
}
