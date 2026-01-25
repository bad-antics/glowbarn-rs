//! Secure key storage

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, Zeroizing};
use std::collections::HashMap;
use std::path::PathBuf;

use super::encryption::AesGcmCipher;

/// Key store for managing encryption keys
pub struct KeyStore {
    /// Master key encrypted keys
    keys: HashMap<String, EncryptedKey>,
    
    /// Master key (derived from password)
    master_key: Option<Zeroizing<[u8; 32]>>,
    
    /// Storage path
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    /// Key ID
    pub id: String,
    
    /// Encrypted key data
    pub encrypted_data: Vec<u8>,
    
    /// Salt used for key derivation
    pub salt: [u8; 32],
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Key type
    pub key_type: KeyType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    DataEncryption,
    NetworkEncryption,
    SigningKey,
    APIKey,
    SessionKey,
}

impl KeyStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            keys: HashMap::new(),
            master_key: None,
            path: None,
        })
    }
    
    /// Initialize with master password
    pub fn init_with_password(&mut self, password: &str) -> Result<()> {
        let salt = super::secure_random_bytes(32);
        let key = derive_key(password, &salt, 100_000)?;
        self.master_key = Some(key);
        Ok(())
    }
    
    /// Unlock with master password
    pub fn unlock(&mut self, password: &str, salt: &[u8; 32]) -> Result<()> {
        let key = derive_key(password, salt, 100_000)?;
        self.master_key = Some(key);
        Ok(())
    }
    
    /// Lock the keystore
    pub fn lock(&mut self) {
        self.master_key = None;
    }
    
    /// Check if keystore is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }
    
    /// Store a key
    pub fn store_key(&mut self, id: &str, key: &[u8], key_type: KeyType) -> Result<()> {
        let master = self.master_key.as_ref()
            .ok_or_else(|| anyhow!("KeyStore is locked"))?;
        
        let cipher = AesGcmCipher::with_key(**master);
        let encrypted_data = cipher.encrypt(key)?;
        
        let salt = {
            let mut s = [0u8; 32];
            let random = super::secure_random_bytes(32);
            s.copy_from_slice(&random);
            s
        };
        
        let encrypted_key = EncryptedKey {
            id: id.to_string(),
            encrypted_data,
            salt,
            created_at: chrono::Utc::now(),
            key_type,
        };
        
        self.keys.insert(id.to_string(), encrypted_key);
        Ok(())
    }
    
    /// Retrieve a key
    pub fn get_key(&self, id: &str) -> Result<Zeroizing<Vec<u8>>> {
        let master = self.master_key.as_ref()
            .ok_or_else(|| anyhow!("KeyStore is locked"))?;
        
        let encrypted = self.keys.get(id)
            .ok_or_else(|| anyhow!("Key not found: {}", id))?;
        
        let cipher = AesGcmCipher::with_key(**master);
        let decrypted = cipher.decrypt(&encrypted.encrypted_data)?;
        
        Ok(Zeroizing::new(decrypted))
    }
    
    /// Delete a key
    pub fn delete_key(&mut self, id: &str) -> Result<()> {
        self.keys.remove(id)
            .ok_or_else(|| anyhow!("Key not found: {}", id))?;
        Ok(())
    }
    
    /// List all key IDs
    pub fn list_keys(&self) -> Vec<&str> {
        self.keys.keys().map(|s| s.as_str()).collect()
    }
    
    /// Generate and store a new random key
    pub fn generate_key(&mut self, id: &str, key_type: KeyType, size: usize) -> Result<()> {
        let key = super::secure_random_bytes(size);
        self.store_key(id, &key, key_type)?;
        Ok(())
    }
    
    /// Save keystore to file
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        let data = serde_json::to_vec_pretty(&self.keys)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    
    /// Load keystore from file
    pub fn load(&mut self, path: &std::path::Path) -> Result<()> {
        let data = std::fs::read(path)?;
        self.keys = serde_json::from_slice(&data)?;
        self.path = Some(path.to_owned());
        Ok(())
    }
}

/// Derive key from password using Argon2id
pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Result<Zeroizing<[u8; 32]>> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString},
        Params,
    };
    
    // Configure Argon2id
    let params = Params::new(
        65536,           // memory cost (64 MB)
        iterations.min(10), // time cost (iterations capped for Argon2)
        4,               // parallelism
        Some(32),        // output length
    ).map_err(|e| anyhow!("Argon2 params error: {}", e))?;
    
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );
    
    // Use salt to create a SaltString (needs base64 encoding)
    let salt_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD_NO_PAD, 
        &salt[..22]  // SaltString needs 22 bytes
    );
    let salt_string = SaltString::from_b64(&salt_b64)
        .map_err(|e| anyhow!("Salt error: {}", e))?;
    
    // Hash password
    let hash = argon2.hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| anyhow!("Password hashing failed: {}", e))?;
    
    // Extract key from hash output
    let hash_output = hash.hash
        .ok_or_else(|| anyhow!("No hash output"))?;
    let bytes = hash_output.as_bytes();
    
    let mut key = Zeroizing::new([0u8; 32]);
    key.copy_from_slice(&bytes[..32]);
    
    Ok(key)
}

/// Derive key using simpler PBKDF2 (fallback)
pub fn derive_key_pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<Zeroizing<[u8; 32]>> {
    use ring::pbkdf2;
    
    let mut key = Zeroizing::new([0u8; 32]);
    
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(iterations).unwrap(),
        salt,
        password.as_bytes(),
        &mut *key,
    );
    
    Ok(key)
}
