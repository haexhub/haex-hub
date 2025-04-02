use std::sync::{Arc, Mutex};
use tauri::http::{Request, Response, ResponseBuilder};

pub struct RoutingMiddleware {
    extensions: Arc<Mutex<Vec<Box<dyn MiddlewareExtension + Send + Sync>>>>,
}

pub trait MiddlewareExtension: Send + Sync {
    fn name(&self) -> &str;
    fn process_url(&self, url: &str) -> String;
    fn process_navigation(&self, url: &str) -> bool;
    fn process_request(&self, request: &Request, response: &mut Response) -> bool;
}

impl RoutingMiddleware {
    pub fn new() -> Self {
        let mut middleware = Self {
            extensions: Arc::new(Mutex::new(Vec::new())),
        };

        // Registriere Standard-Erweiterungen
        //middleware.register_extension(Box::new(AdBlockerExtension::new()));

        middleware
    }

    pub fn register_extension(&mut self, extension: Box<dyn MiddlewareExtension + Send + Sync>) {
        let mut extensions = self.extensions.lock().unwrap();
        extensions.push(extension);
    }

    pub fn process_url(&self, url: &str) -> String {
        let extensions = self.extensions.lock().unwrap();
        let mut processed_url = url.to_string();

        for extension in extensions.iter() {
            processed_url = extension.process_url(&processed_url);
        }

        processed_url
    }

    pub fn process_navigation(&self, url: &str) -> bool {
        let extensions = self.extensions.lock().unwrap();

        for extension in extensions.iter() {
            if !extension.process_navigation(url) {
                return false;
            }
        }

        true
    }

    pub fn process_request(&self, request: &Request, response: &mut Response) -> bool {
        let extensions = self.extensions.lock().unwrap();

        for extension in extensions.iter() {
            if extension.process_request(request, response) {
                return true;
            }
        }

        false
    }
}

// Beispiel für eine Ad-Blocker-Erweiterung
struct AdBlockerExtension {
    block_patterns: Vec<String>,
}

impl AdBlockerExtension {
    fn new() -> Self {
        Self {
            block_patterns: vec![
                "ads".to_string(),
                "analytics".to_string(),
                "tracker".to_string(),
                "banner".to_string(),
            ],
        }
    }

    fn is_blocked_url(&self, url: &str) -> bool {
        for pattern in &self.block_patterns {
            if url.contains(pattern) {
                return true;
            }
        }
        false
    }
}

impl MiddlewareExtension for AdBlockerExtension {
    fn name(&self) -> &str {
        "AdBlocker"
    }

    fn process_url(&self, url: &str) -> String {
        // Für vollständige Navigationen blockieren wir normalerweise nicht die ganze Seite
        url.to_string()
    }

    fn process_navigation(&self, url: &str) -> bool {
        // Blockiere nur vollständige Navigationen zu Werbeseiten
        let is_ad_site = url.contains("doubleclick.net")
            || url.contains("googleadservices.com")
            || url.contains("ads.example.com");
        !is_ad_site
    }

    fn process_request(&self, request: &Request, response: &mut Response) -> bool {
        let url = request.uri().to_string();
        if self.is_blocked_url(&url) {
            println!("AdBlocker: Blockiere Anfrage: {}", url);
            *response = ResponseBuilder::new()
                .status(403)
                .body("Zugriff verweigert durch AdBlocker".as_bytes().to_vec())
                .unwrap();
            return true;
        }
        false
    }
}
