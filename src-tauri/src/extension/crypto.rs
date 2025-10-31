use std::{
    fs,
    path::{Path, PathBuf},
};

// src-tauri/src/extension/crypto.rs
use crate::extension::error::ExtensionError;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

pub struct ExtensionCrypto;

impl ExtensionCrypto {
    /// Verifiziert Extension-Signatur
    pub fn verify_signature(
        public_key_hex: &str,
        content_hash_hex: &str,
        signature_hex: &str,
    ) -> Result<(), String> {
        let public_key_bytes =
            hex::decode(public_key_hex).map_err(|e| format!("Invalid public key: {}", e))?;
        let public_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
            .map_err(|e| format!("Invalid public key: {}", e))?;

        let signature_bytes =
            hex::decode(signature_hex).map_err(|e| format!("Invalid signature: {}", e))?;
        let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());

        let content_hash =
            hex::decode(content_hash_hex).map_err(|e| format!("Invalid content hash: {}", e))?;

        public_key
            .verify(&content_hash, &signature)
            .map_err(|e| format!("Signature verification failed: {}", e))
    }

    /// Berechnet Hash eines Verzeichnisses (für Verifikation)
    pub fn hash_directory(dir: &Path, manifest_path: &Path) -> Result<String, ExtensionError> {
        // 1. Alle Dateipfade rekursiv sammeln
        let mut all_files = Vec::new();
        Self::collect_files_recursively(dir, &mut all_files)
            .map_err(|e| ExtensionError::Filesystem { source: e })?;

        // 2. Konvertiere zu relativen Pfaden für konsistente Sortierung (wie im SDK)
        let mut relative_files: Vec<(String, PathBuf)> = all_files
            .into_iter()
            .map(|path| {
                let relative = path.strip_prefix(dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string()
                    // Normalisiere Pfad-Separatoren zu Unix-Style (/) für plattformübergreifende Konsistenz
                    .replace('\\', "/");
                (relative, path)
            })
            .collect();

        // 3. Sortiere nach relativen Pfaden
        relative_files.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();

        // Canonicalize manifest path for comparison (important on Android where symlinks may differ)
        // Also ensure the canonical path is still within the allowed directory (security check)
        let canonical_manifest_path = manifest_path.canonicalize()
            .unwrap_or_else(|_| manifest_path.to_path_buf());

        // Security: Verify canonical manifest path is still within dir
        let canonical_dir = dir.canonicalize()
            .unwrap_or_else(|_| dir.to_path_buf());

        if !canonical_manifest_path.starts_with(&canonical_dir) {
            return Err(ExtensionError::ManifestError {
                reason: format!("Manifest path resolves outside of extension directory (potential path traversal)"),
            });
        }

        // 4. Inhalte der sortierten Dateien hashen
        for (_relative, file_path) in relative_files {
            // Canonicalize file_path for comparison
            let canonical_file_path = file_path.canonicalize()
                .unwrap_or_else(|_| file_path.clone());

            if canonical_file_path == canonical_manifest_path {
                // FÜR DIE MANIFEST.JSON:
                let content_str = fs::read_to_string(&file_path)
                    .map_err(|e| ExtensionError::Filesystem { source: e })?;

                // Parse zu einem generischen JSON-Wert
                let mut manifest: serde_json::Value =
                    serde_json::from_str(&content_str).map_err(|e| {
                        ExtensionError::ManifestError {
                            reason: format!("Cannot parse manifest JSON: {}", e),
                        }
                    })?;

                // Entferne oder leere das Signaturfeld, um den "kanonischen Inhalt" zu erhalten
                if let Some(obj) = manifest.as_object_mut() {
                    obj.insert(
                        "signature".to_string(),
                        serde_json::Value::String("".to_string()),
                    );
                }

                // Serialisiere das modifizierte Manifest zurück (mit 2 Spaces, wie in JS)
                // serde_json sortiert die Keys automatisch alphabetisch
                let canonical_manifest_content =
                    serde_json::to_string_pretty(&manifest).map_err(|e| {
                        ExtensionError::ManifestError {
                            reason: format!("Failed to serialize manifest: {}", e),
                        }
                    })?;

                // Normalisiere Zeilenenden zu Unix-Style (\n), wie Node.js JSON.stringify es macht
                // Dies ist wichtig für plattformübergreifende Konsistenz (Desktop vs Android)
                let normalized_content = canonical_manifest_content.replace("\r\n", "\n");

                hasher.update(normalized_content.as_bytes());
            } else {
                // FÜR ALLE ANDEREN DATEIEN:
                let content =
                    fs::read(&file_path).map_err(|e| ExtensionError::Filesystem { source: e })?;
                hasher.update(&content);
            }
        }

        Ok(hex::encode(hasher.finalize()))
    }

    fn collect_files_recursively(dir: &Path, file_list: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_files_recursively(&path, file_list)?;
                } else {
                    file_list.push(path);
                }
            }
        }
        Ok(())
    }
}
