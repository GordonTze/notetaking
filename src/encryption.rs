use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::RngCore};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub salt: String,
}

pub struct Encryption {
    password_hash: Option<String>,
}

impl Encryption {
    pub fn new() -> Self {
        Self {
            password_hash: None,
        }
    }
    
    pub fn set_password(&mut self, password: &str) -> Result<(), String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?
            .to_string();
        
        self.password_hash = Some(hash);
        Ok(())
    }
    
    pub fn verify_password(&self, password: &str) -> bool {
        if let Some(hash) = &self.password_hash {
            if let Ok(parsed_hash) = PasswordHash::new(hash) {
                return Argon2::default()
                    .verify_password(password.as_bytes(), &parsed_hash)
                    .is_ok();
            }
        }
        false
    }
    
    pub fn is_password_set(&self) -> bool {
        self.password_hash.is_some()
    }
    
    fn derive_key(&self, password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
        let argon2 = Argon2::default();
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| format!("Salt encoding error: {}", e))?;
        
        let hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| format!("Key derivation error: {}", e))?;
        
        let hash_bytes = hash.hash.ok_or("No hash output")?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        Ok(key)
    }
    
    pub fn encrypt(&self, plaintext: &str, password: &str) -> Result<EncryptedData, String> {
        // Generate random salt
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        
        // Derive key from password
        let key = self.derive_key(password, &salt)?;
        
        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher creation error: {}", e))?;
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;
        
        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            salt: general_purpose::STANDARD.encode(&salt),
        })
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData, password: &str) -> Result<String, String> {
        // Decode base64
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.ciphertext)
            .map_err(|e| format!("Invalid ciphertext: {}", e))?;
        
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| format!("Invalid nonce: {}", e))?;
        
        let salt = general_purpose::STANDARD
            .decode(&encrypted.salt)
            .map_err(|e| format!("Invalid salt: {}", e))?;
        
        // Derive key
        let key = self.derive_key(password, &salt)?;
        
        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher creation error: {}", e))?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Decryption failed - wrong password?".to_string())?;
        
        String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let encryption = Encryption::new();
        let password = "test_password_123";
        let plaintext = "This is a secret note!";
        
        let encrypted = encryption.encrypt(plaintext, password).unwrap();
        let decrypted = encryption.decrypt(&encrypted, password).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_wrong_password() {
        let encryption = Encryption::new();
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let plaintext = "Secret data";
        
        let encrypted = encryption.encrypt(plaintext, password).unwrap();
        let result = encryption.decrypt(&encrypted, wrong_password);
        
        assert!(result.is_err());
    }
}
