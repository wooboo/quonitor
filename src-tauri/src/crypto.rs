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
        // Try to get key from keyring
        match keyring::Entry::new("quonitor", "master_key") {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(key_str) => {
                        // Decode existing key
                        let key_bytes = general_purpose::STANDARD
                            .decode(&key_str)
                            .map_err(|e| QuonitorError::Encryption(format!("Failed to decode key: {}", e)))?;

                        let mut key = [0u8; 32];
                        key.copy_from_slice(&key_bytes);
                        Ok(key)
                    }
                    Err(_) => {
                        // Generate new key
                        let key = Aes256Gcm::generate_key(OsRng);
                        let key_str = general_purpose::STANDARD.encode(&key);

                        entry.set_password(&key_str)
                            .map_err(|e| QuonitorError::Encryption(format!("Failed to store key: {}", e)))?;

                        Ok(key.into())
                    }
                }
            }
            Err(e) => {
                Err(QuonitorError::Encryption(format!("Failed to access keyring: {}", e)))
            }
        }
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
