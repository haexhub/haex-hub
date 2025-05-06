use mime;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::{
    http::{Request, Response, Uri},
    AppHandle, Error as TauriError, Manager, Runtime,
};

pub fn copy_directory(source: String, destination: String) -> Result<(), String> {
    println!(
        "Kopiere Verzeichnis von '{}' nach '{}'",
        source, destination
    );

    let source_path = PathBuf::from(&source);
    let destination_path = PathBuf::from(&destination);

    if !source_path.exists() || !source_path.is_dir() {
        return Err(format!(
            "Quellverzeichnis '{}' nicht gefunden oder ist kein Verzeichnis.",
            source
        ));
    }

    // Optionen für fs_extra::dir::copy
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true; // Überschreibe Zieldateien, falls sie existieren
    options.copy_inside = true; // Kopiere den *Inhalt* des Quellordners in den Zielordner
                                // options.content_only = true; // Alternative: nur Inhalt kopieren, Zielordner muss existieren
    options.buffer_size = 64000; // Standard-Puffergröße, kann angepasst werden

    // Führe die Kopieroperation aus
    match fs_extra::dir::copy(&source_path, &destination_path, &options) {
        Ok(bytes_copied) => {
            println!("Verzeichnis erfolgreich kopiert ({} bytes)", bytes_copied);
            Ok(()) // Erfolg signalisieren
        }
        Err(e) => {
            eprintln!("Fehler beim Kopieren des Verzeichnisses: {}", e);
            Err(format!("Fehler beim Kopieren: {}", e.to_string())) // Fehler als String zurückgeben
        }
    }
}

pub fn resolve_secure_extension_asset_path<R: Runtime>(
    app_handle: &AppHandle<R>,
    extension_id: &str,
    requested_asset_path: &str,
) -> Result<PathBuf, String> {
    // 1. Validiere die Extension ID
    if extension_id.is_empty()
        || !extension_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(format!("Ungültige Extension ID: {}", extension_id));
    }

    // 2. Bestimme das Basisverzeichnis für alle Erweiterungen (Resource Directory)
    let base_extensions_dir = app_handle
        .path()
        .resource_dir() // Korrekt für Ressourcen
        // Wenn du stattdessen App Local Data willst: .app_local_data_dir()
        .map_err(|e: TauriError| format!("Basis-Verzeichnis nicht gefunden: {}", e))?
        .join("extensions");

    // 3. Verzeichnis für die spezifische Erweiterung
    let specific_extension_dir = base_extensions_dir.join(extension_id);

    // 4. Bereinige den angeforderten Asset-Pfad
    let clean_relative_path = requested_asset_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .split('/')
        .filter(|&part| !part.is_empty() && part != "." && part != "..")
        .collect::<PathBuf>();

    if clean_relative_path.as_os_str().is_empty() && requested_asset_path != "/" {
        return Err("Leerer oder ungültiger Asset-Pfad".to_string());
    }

    // 5. Setze den finalen Pfad zusammen
    let final_path = specific_extension_dir.join(clean_relative_path);

    // 6. SICHERHEITSCHECK (wie vorher)
    match final_path.canonicalize() {
        Ok(canonical_path) => {
            let canonical_base = specific_extension_dir.canonicalize().map_err(|e| {
                format!(
                    "Kann Basis-Pfad '{}' nicht kanonisieren: {}",
                    specific_extension_dir.display(),
                    e
                )
            })?;
            if canonical_path.starts_with(&canonical_base) {
                Ok(canonical_path)
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err("Ungültiger oder nicht erlaubter Asset-Pfad (kanonisch)".to_string())
            }
        }
        Err(_) => {
            // Fehler bei canonicalize (z.B. Pfad existiert nicht)
            if final_path.starts_with(&specific_extension_dir) {
                Ok(final_path) // Nicht-kanonisierten Pfad zurückgeben
            } else {
                eprintln!( /* ... Sicherheitswarnung ... */ );
                Err("Ungültiger oder nicht erlaubter Asset-Pfad (nicht kanonisiert)".to_string())
            }
        }
    }
}

pub fn handle_extension_protocol<R: Runtime>(
    app_handle: &AppHandle<R>,
    request: &Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let uri_ref = request.uri(); // uri_ref ist &Uri
    println!("Protokoll Handler für: {}", uri_ref);

    let uri_string = uri_ref.to_string(); // Konvertiere zu String
    let parsed_uri = Uri::from_str(&uri_string)?; // Parse aus &str

    let extension_id = parsed_uri
        .host()
        .ok_or("Kein Host (Extension ID) in URI gefunden")?
        .to_string();

    let requested_asset_path = parsed_uri.path();

    let asset_path_to_load = if requested_asset_path == "/" || requested_asset_path.is_empty() {
        "index.html"
    } else {
        requested_asset_path
    };

    // Sicheren Dateisystempfad auflösen (nutzt jetzt AppHandle)
    let absolute_secure_path =
        resolve_secure_extension_asset_path(app_handle, &extension_id, asset_path_to_load)?;

    // Datei lesen und Response erstellen (Code wie vorher)
    match fs::read(&absolute_secure_path) {
        Ok(content) => {
            let mime_type = mime_guess::from_path(&absolute_secure_path)
                .first_or(mime::APPLICATION_OCTET_STREAM)
                .to_string();
            println!(
                "Liefere {} ({}) für Extension '{}'",
                absolute_secure_path.display(),
                mime_type,
                extension_id
            );
            // *** KORREKTUR: Verwende Response::builder() ***
            Response::builder()
                .status(200)
                .header("Content-Type", mime_type) // Setze Header über .header()
                .body(content) // body() gibt Result<Response<Vec<u8>>, Error> zurück
                .map_err(|e| e.into()) // Wandle http::Error in Box<dyn Error> um
        }
        Err(e) => {
            eprintln!(
                "Fehler beim Lesen der Datei {}: {}",
                absolute_secure_path.display(),
                e
            );
            let status_code = if e.kind() == std::io::ErrorKind::NotFound {
                404
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                403
            } else {
                500
            };
            // *** KORREKTUR: Verwende Response::builder() auch für Fehler ***
            Response::builder()
                .status(status_code)
                .body(Vec::new()) // Leerer Body für Fehler
                .map_err(|e| e.into()) // Wandle http::Error in Box<dyn Error> um
        }
    }
}
