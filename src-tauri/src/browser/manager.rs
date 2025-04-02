//mod middleware;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{webview, AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, Window};
//use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: String,
    pub webview_label: String,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub is_visible: bool,
}

pub struct BrowserManager {
    tabs: Arc<Mutex<HashMap<String, Tab>>>,
    active_tab_id: Arc<Mutex<Option<String>>>,
    //middleware: Arc<RoutingMiddleware>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            tabs: Arc::new(Mutex::new(HashMap::new())),
            active_tab_id: Arc::new(Mutex::new(None)),
            //middleware: Arc::new(RoutingMiddleware::new()),
        }
    }

    /* pub async fn create_window(app: tauri::AppHandle) -> Result<tauri::WebviewWindow, _> {
        let webview_window = tauri::WebviewWindowBuilder::new(
            &app,
            "label",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .build()
        .unwrap();
        Ok(webview_window);
    } */
    pub fn create_tab(&self, app: AppHandle, url: &str) {
        // Generiere eine eindeutige ID für den Tab
        /* let tab_id = Uuid::new_v4().to_string();
        let webview_label = format!("webview-{}", tab_id); */

        // Überprüfe URL mit Middleware
        //let processed_url = self.middleware.process_url(url);

        // Hole das Hauptfenster
        let main_window = app.get_webview_window("main").unwrap();

        // Berechne die Position und Größe für den Webview
        // Hier nehmen wir an, dass wir einen Header-Bereich von 100 Pixeln haben
        /* let window_size = main_window.inner_size()?;
               let header_height = 100.0;
               let webview_position = LogicalPosition::new(0.0, header_height);
               let webview_size = LogicalSize::new(window_size.width, window_size.height - header_height);
        */
        /* let webview = tauri::WebviewWindowBuilder::new(
            &app,
            "label",
            //WebviewUrl::External(processed_url.parse().unwrap()),
            WebviewUrl::External(url),
        )
        .build()
        .unwrap() */
        /* .on_navigation(move |url| {
            // Middleware für Navigation anwenden
            self.middleware.process_navigation(url.as_str())
        })
        .on_web_resource_request(move |request, response| {
            // Middleware für HTTP-Anfragen anwenden
            self.middleware.process_request(request, response)
        }); */

        // Erstelle Tab-Objekt
        /* let tab = Tab {
            id: tab_id.clone(),
            webview_label: webview_label.clone(),
            title: "Neuer Tab".to_string(),
            url: processed_url.to_string(),
            is_loading: true,
            is_visible: false,
        }; */

        // Speichere Tab
        /* {
            let mut tabs = self.tabs.lock().unwrap();
            tabs.insert(tab_id.clone(), tab.clone());
        } */

        // Setze als aktiven Tab
        //self.activate_tab(app, &tab_id)?;

        // Injiziere die Webview-Bridge
        /* let script = include_str!("../assets/webview-bridge.js");
        webview.evaluate_script(script)?; */

        // Registriere Event-Handler für Titeländerungen
        let tab_manager = self.clone();
        //let tab_id_clone = tab_id.clone();
        /* webview.listen("tauri://title-changed", move |event| {
            if let Some(title) = event.payload().and_then(|p| p.as_str()) {
                tab_manager.update_tab_title(&tab_id_clone, title);
            }
        }); */

        // Registriere Event-Handler für Ladestatus
        let tab_manager = self.clone();
        //let tab_id_clone = tab_id.clone();
        /* webview.listen("tauri://load-changed", move |event| {
            if let Some(status) = event.payload().and_then(|p| p.as_str()) {
                let is_loading = status == "loading";
                tab_manager.update_tab_loading_status(&tab_id_clone, is_loading);
            }
        }); */

        //Ok()
    }

    pub fn close_tab(&self, app: &AppHandle, tab_id: &str) -> Result<(), tauri::Error> {
        // Hole das Hauptfenster
        let main_window = app.get_webview_window("main").unwrap();

        // Entferne Tab aus der Verwaltung
        let webview_label = {
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(tab) = tabs.remove(tab_id) {
                tab.webview_label
            } else {
                return Ok(());
            }
        };

        // Entferne den Webview
        //main_window.remove_child(&webview_label)?;

        // Aktualisiere aktiven Tab, falls nötig
        {
            let mut active_tab_id = self.active_tab_id.lock().unwrap();
            if active_tab_id.as_ref().map_or(false, |id| id == tab_id) {
                // Wähle einen anderen Tab als aktiv
                let tabs = self.tabs.lock().unwrap();
                *active_tab_id = tabs.keys().next().cloned();

                // Aktiviere den neuen Tab, falls vorhanden
                if let Some(new_active_id) = active_tab_id.clone() {
                    drop(active_tab_id); // Mutex freigeben vor dem rekursiven Aufruf
                    self.activate_tab(app, &new_active_id)?;
                }
            }
        }

        Ok(())
    }

    pub fn activate_tab(&self, app: &AppHandle, tab_id: &str) -> Result<(), tauri::Error> {
        // Hole das Hauptfenster
        let main_window = app.get_webview_window("main").unwrap();

        // Setze Tab als aktiv
        {
            let mut active_tab_id = self.active_tab_id.lock().unwrap();
            *active_tab_id = Some(tab_id.to_string());
        }

        // Verstecke alle anderen Tabs und zeige den aktiven
        let mut tabs = self.tabs.lock().unwrap();
        for (id, tab) in tabs.iter_mut() {
            if id == tab_id {
                // Zeige den aktiven Tab
                /* main_window
                .get_webview_window(&tab.webview_label)?
                .set_visible(true)?; */
                tab.is_visible = true;
            } else {
                // Verstecke alle anderen Tabs
                /* main_window
                .get_webview_window(&tab.webview_label)?
                .set_visible(false)?; */
                tab.is_visible = false;
            }
        }

        Ok(())
    }

    pub fn navigate_to_url(
        &self,
        app: &AppHandle,
        tab_id: &str,
        url: &str,
    ) -> Result<(), tauri::Error> {
        // Überprüfe URL mit Middleware
        //let processed_url = self.middleware.process_url(url);

        // Aktualisiere URL im Tab
        {
            let mut tabs = self.tabs.lock().unwrap();
            if let Some(tab) = tabs.get_mut(tab_id) {
                tab.url = url.to_string() //processed_url.to_string();
            }
        }

        // Navigiere zum URL im Webview
        let tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get(tab_id) {
            let main_window = app.get_webview_window("main").unwrap();
            /* let webview = main_window.get_webview_window(&tab.webview_label)?;
            webview.navigate(&processed_url)?; */
        }

        Ok(())
    }

    pub fn get_all_tabs(&self) -> Vec<Tab> {
        let tabs = self.tabs.lock().unwrap();
        tabs.values().cloned().collect()
    }

    pub fn get_active_tab_id(&self) -> Option<String> {
        let active_tab_id = self.active_tab_id.lock().unwrap();
        active_tab_id.clone()
    }

    pub fn update_tab_title(&self, tab_id: &str, title: &str) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.title = title.to_string();
        }
    }

    pub fn update_tab_loading_status(&self, tab_id: &str, is_loading: bool) {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get_mut(tab_id) {
            tab.is_loading = is_loading;
        }
    }

    // Weitere Methoden für Browser-Navigation
    pub fn go_back(&self, app: &AppHandle, tab_id: &str) -> Result<(), tauri::Error> {
        let tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get(tab_id) {
            let main_window = app.get_webview_window("main").unwrap();
            /* let webview = main_window.get_webview(&tab.webview_label)?;
            webview.evaluate_script("window.history.back()")?; */
        }
        Ok(())
    }

    pub fn go_forward(&self, app: &AppHandle, tab_id: &str) -> Result<(), tauri::Error> {
        let tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get(tab_id) {
            let main_window = app.get_webview_window("main").unwrap();
            /* let webview = main_window.get_webview(&tab.webview_label)?;
            webview.evaluate_script("window.history.forward()")?; */
        }
        Ok(())
    }

    pub fn inject_content_script(
        &self,
        app: &AppHandle,
        tab_id: &str,
        script: &str,
    ) -> Result<(), tauri::Error> {
        let tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.get(tab_id) {
            let main_window = app.get_webview_window("main").unwrap();
            /* let webview = main_window.get_webview(&tab.webview_label)?;
            webview.evaluate_script(script)?; */
        }
        Ok(())
    }

    pub fn clone(&self) -> Self {
        Self {
            tabs: Arc::clone(&self.tabs),
            active_tab_id: Arc::clone(&self.active_tab_id),
            //middleware: Arc::clone(&self.middleware),
        }
    }
}
