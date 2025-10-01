// src-tauri/src/extension/crypto.rs
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

pub struct ExtensionCrypto;

impl ExtensionCrypto {
    /// Berechnet Hash vom Public Key (wie im SDK)
    pub fn calculate_key_hash(public_key_hex: &str) -> Result<String, String> {
        let public_key_bytes =
            hex::decode(public_key_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;

        let public_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())
            .map_err(|e| format!("Invalid public key: {}", e))?;

        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let result = hasher.finalize();

        // Ersten 20 Hex-Zeichen (10 Bytes) - wie im SDK
        Ok(hex::encode(&result[..10]))
    }

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
    pub fn hash_directory(dir: &std::path::Path) -> Result<String, String> {
        use std::fs;

        let mut hasher = Sha256::new();
        let mut entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| format!("Cannot read directory: {}", e))?
            .filter_map(|e| e.ok())
            .collect();

        // Sortieren für deterministische Hashes
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let content = fs::read(&path)
                    .map_err(|e| format!("Cannot read file {}: {}", path.display(), e))?;
                hasher.update(&content);
            } else if path.is_dir() {
                let subdir_hash = Self::hash_directory(&path)?;
                hasher.update(hex::decode(&subdir_hash).unwrap());
            }
        }

        Ok(hex::encode(hasher.finalize()))
    }
}
