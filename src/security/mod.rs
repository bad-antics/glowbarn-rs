// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Security module - encryption, secure storage, authentication

mod encryption;
mod keystore;
mod auth;
mod secure_memory;

pub use encryption::*;
pub use keystore::*;
pub use auth::*;
pub use secure_memory::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable encryption for stored data
    pub encrypt_storage: bool,
    
    /// Enable encryption for network traffic
    pub encrypt_network: bool,
    
    /// Key derivation iterations (higher = slower but more secure)
    pub kdf_iterations: u32,
    
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    
    /// Enable audit logging
    pub audit_logging: bool,
    
    /// Minimum password length
    pub min_password_length: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encrypt_storage: true,
            encrypt_network: true,
            kdf_iterations: 100_000,
            session_timeout_secs: 3600,  // 1 hour
            audit_logging: true,
            min_password_length: 12,
        }
    }
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    cipher: AesGcmCipher,
    keystore: KeyStore,
    auth: AuthManager,
    audit: Option<AuditLog>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let keystore = KeyStore::new()?;
        let cipher = AesGcmCipher::new()?;
        let auth = AuthManager::new(config.min_password_length);
        let audit = if config.audit_logging {
            Some(AuditLog::new())
        } else {
            None
        };
        
        Ok(Self {
            config,
            cipher,
            keystore,
            auth,
            audit,
        })
    }
    
    /// Encrypt data with AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.encrypt(plaintext)
    }
    
    /// Decrypt data
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.cipher.decrypt(ciphertext)
    }
    
    /// Hash password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String> {
        self.auth.hash_password(password)
    }
    
    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        self.auth.verify_password(password, hash)
    }
    
    /// Generate secure random bytes
    pub fn random_bytes(&self, len: usize) -> Vec<u8> {
        secure_random_bytes(len)
    }
    
    /// Log security audit event
    pub fn log_audit(&self, event: AuditEvent) {
        if let Some(ref audit) = self.audit {
            audit.log(event);
        }
    }
}

/// Audit event for security logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub description: String,
    pub user: Option<String>,
    pub ip_address: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Login,
    Logout,
    PasswordChange,
    DataAccess,
    DataExport,
    ConfigChange,
    SessionExpired,
    AuthFailure,
    EncryptionOperation,
    SystemStart,
    SystemStop,
}

/// Simple audit log
pub struct AuditLog {
    events: std::sync::RwLock<Vec<AuditEvent>>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::new()),
        }
    }
    
    pub fn log(&self, event: AuditEvent) {
        if let Ok(mut events) = self.events.write() {
            info!(
                event_type = ?event.event_type,
                success = event.success,
                "Audit: {}", event.description
            );
            events.push(event);
            
            // Keep only last 10000 events in memory
            if events.len() > 10000 {
                let drain_count = events.len() - 10000;
                events.drain(0..drain_count);
            }
        }
    }
    
    pub fn get_events(&self, limit: usize) -> Vec<AuditEvent> {
        self.events.read()
            .map(|events| events.iter().rev().take(limit).cloned().collect())
            .unwrap_or_default()
    }
}

/// Generate secure random bytes using ring
pub fn secure_random_bytes(len: usize) -> Vec<u8> {
    use ring::rand::{SecureRandom, SystemRandom};
    
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes).expect("Failed to generate random bytes");
    bytes
}
