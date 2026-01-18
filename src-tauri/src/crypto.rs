use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use crate::error::{QuonitorError, Result};

const NONCE_SIZE: usize = 12;

pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    pub fn new() -> Result<Self> {
        // In production, this key should be derived from a master key stored securely
        // For now, we'll use a key from the OS keyring or environment
        let key = Self::get_or_create_master_key()?;
        let cipher = Aes256Gcm::new(&key.into());

        Ok(Self { cipher })
    }

    fn get_or_create_master_key() -> Result<[u8; 32]> {
        // Try to get key from keyring first
        if let Ok(entry) = keyring::Entry::new("quonitor", "master_key") {
            if let Ok(key_str) = entry.get_password() {
                if let Ok(key_bytes) = general_purpose::STANDARD.decode(&key_str) {
                    let mut key = [0u8; 32];
                    if key_bytes.len() == 32 {
                        key.copy_from_slice(&key_bytes);
                        return Ok(key);
                    }
                }
            }
        }

        // Fallback: Try file-based key in app data directory
        let data_dir = dirs::data_local_dir()
            .map(|p| p.join("quonitor"))
            .ok_or_else(|| QuonitorError::Encryption("Failed to get data directory".to_string()))?;
            
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| QuonitorError::Encryption(format!("Failed to create data dir: {}", e)))?;
            
        let key_path = data_dir.join("master.key");
        
        if key_path.exists() {
            // Read existing key from file
            let key_str = std::fs::read_to_string(&key_path)
                .map_err(|e| QuonitorError::Encryption(format!("Failed to read key file: {}", e)))?;
                
            let key_bytes = general_purpose::STANDARD
                .decode(key_str.trim())
                .map_err(|e| QuonitorError::Encryption(format!("Failed to decode file key: {}", e)))?;

            let mut key = [0u8; 32];
            if key_bytes.len() == 32 {
                key.copy_from_slice(&key_bytes);
                return Ok(key);
            }
        }

        // Generate new key if neither found
        let key = Aes256Gcm::generate_key(OsRng);
        let key_str = general_purpose::STANDARD.encode(&key);

        // Try to save to keyring
        if let Ok(entry) = keyring::Entry::new("quonitor", "master_key") {
            let _ = entry.set_password(&key_str);
        }

        // Always save to file as backup/primary
        std::fs::write(&key_path, &key_str)
            .map_err(|e| QuonitorError::Encryption(format!("Failed to write key file: {}", e)))?;

        Ok(key.into())
    }

    pub fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]); // In production, use random nonce per encryption

        let ciphertext = self.cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| QuonitorError::Encryption(format!("Encryption failed: {}", e)))?;

        Ok(ciphertext)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<String> {
        let nonce = Nonce::from_slice(&[0u8; NONCE_SIZE]);

        let plaintext = self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| QuonitorError::Encryption(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| QuonitorError::Encryption(format!("Invalid UTF-8: {}", e)))
    }
}
