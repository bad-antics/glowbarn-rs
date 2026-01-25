//! AES-256-GCM encryption

use aes_gcm::{
    Aes256Gcm,
    Key, Nonce,
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
};
use anyhow::{anyhow, Result};
use zeroize::Zeroizing;

/// AES-256-GCM cipher
pub struct AesGcmCipher {
    key: Zeroizing<[u8; 32]>,
}

impl AesGcmCipher {
    /// Create new cipher with random key
    pub fn new() -> Result<Self> {
        let mut key = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(&mut *key);
        Ok(Self { key })
    }
    
    /// Create cipher with provided key
    pub fn with_key(key: [u8; 32]) -> Self {
        Self { key: Zeroizing::new(key) }
    }
    
    /// Encrypt plaintext
    /// Returns: nonce (12 bytes) || ciphertext || tag (16 bytes)
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*self.key));
        
        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt ciphertext
    /// Input format: nonce (12 bytes) || ciphertext || tag (16 bytes)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 28 {  // 12 nonce + 16 tag minimum
            return Err(anyhow!("Ciphertext too short"));
        }
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*self.key));
        
        // Extract nonce
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];
        
        // Decrypt
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
    
    /// Get key (for secure storage)
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }
}

/// ChaCha20-Poly1305 cipher (alternative)
pub struct ChaCha20Cipher {
    key: Zeroizing<[u8; 32]>,
}

impl ChaCha20Cipher {
    pub fn new() -> Result<Self> {
        let mut key = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(&mut *key);
        Ok(Self { key })
    }
    
    pub fn with_key(key: [u8; 32]) -> Self {
        Self { key: Zeroizing::new(key) }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, aead::{Aead, KeyInit}};
        
        let cipher = ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&*self.key));
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("ChaCha20 encryption failed: {}", e))?;
        
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, aead::{Aead, KeyInit}};
        
        if data.len() < 28 {
            return Err(anyhow!("Ciphertext too short"));
        }
        
        let cipher = ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&*self.key));
        
        let nonce = chacha20poly1305::Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];
        
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("ChaCha20 decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

/// Encrypt file
pub fn encrypt_file(input_path: &std::path::Path, output_path: &std::path::Path, key: &[u8; 32]) -> Result<()> {
    let plaintext = std::fs::read(input_path)?;
    let cipher = AesGcmCipher::with_key(*key);
    let ciphertext = cipher.encrypt(&plaintext)?;
    std::fs::write(output_path, ciphertext)?;
    Ok(())
}

/// Decrypt file
pub fn decrypt_file(input_path: &std::path::Path, output_path: &std::path::Path, key: &[u8; 32]) -> Result<()> {
    let ciphertext = std::fs::read(input_path)?;
    let cipher = AesGcmCipher::with_key(*key);
    let plaintext = cipher.decrypt(&ciphertext)?;
    std::fs::write(output_path, plaintext)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_encrypt_decrypt() {
        let cipher = AesGcmCipher::new().unwrap();
        let plaintext = b"Hello, GlowBarn!";
        
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();
        
        assert_eq!(&decrypted, plaintext);
    }
    
    #[test]
    fn test_chacha20_encrypt_decrypt() {
        let cipher = ChaCha20Cipher::new().unwrap();
        let plaintext = b"Test message for ChaCha20!";
        
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();
        
        assert_eq!(&decrypted, plaintext);
    }
}
