//! Authentication and session management

use anyhow::{anyhow, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

/// Authentication manager
pub struct AuthManager {
    /// Active sessions
    sessions: HashMap<String, Session>,
    
    /// Failed login attempts
    failed_attempts: HashMap<String, (u32, DateTime<Utc>)>,
    
    /// Lockout threshold
    lockout_threshold: u32,
    
    /// Lockout duration
    lockout_duration: Duration,
    
    /// Minimum password length
    min_password_length: usize,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// Password strength result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub score: u32,  // 0-4 (weak to very strong)
    pub feedback: Vec<String>,
    pub acceptable: bool,
}

impl AuthManager {
    pub fn new(min_password_length: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            failed_attempts: HashMap::new(),
            lockout_threshold: 5,
            lockout_duration: Duration::minutes(15),
            min_password_length,
        }
    }
    
    /// Hash password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        
        let argon2 = Argon2::default();
        
        let hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {}", e))?;
        
        Ok(hash.to_string())
    }
    
    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Invalid hash format: {}", e))?;
        
        let argon2 = Argon2::default();
        
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    /// Check password strength
    pub fn check_password_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0u32;
        let mut feedback = Vec::new();
        
        // Length check
        if password.len() < self.min_password_length {
            feedback.push(format!(
                "Password should be at least {} characters", 
                self.min_password_length
            ));
        } else if password.len() >= 16 {
            score += 2;
        } else if password.len() >= self.min_password_length {
            score += 1;
        }
        
        // Character variety
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());
        
        if !has_lower {
            feedback.push("Add lowercase letters".to_string());
        }
        if !has_upper {
            feedback.push("Add uppercase letters".to_string());
        }
        if !has_digit {
            feedback.push("Add numbers".to_string());
        }
        if !has_special {
            feedback.push("Add special characters".to_string());
        }
        
        let variety_count = [has_lower, has_upper, has_digit, has_special]
            .iter().filter(|&&x| x).count();
        
        score += variety_count as u32;
        
        // Common patterns check
        let lower = password.to_lowercase();
        let common_patterns = [
            "password", "123456", "qwerty", "abc123", "letmein",
            "welcome", "admin", "login", "passw0rd", "master",
        ];
        
        if common_patterns.iter().any(|p| lower.contains(p)) {
            score = score.saturating_sub(2);
            feedback.push("Avoid common patterns".to_string());
        }
        
        // Repeated characters
        let mut prev_char = '\0';
        let mut repeat_count = 0;
        for c in password.chars() {
            if c == prev_char {
                repeat_count += 1;
                if repeat_count >= 3 {
                    score = score.saturating_sub(1);
                    feedback.push("Avoid repeated characters".to_string());
                    break;
                }
            } else {
                repeat_count = 1;
            }
            prev_char = c;
        }
        
        // Normalize score to 0-4
        score = score.min(4);
        
        let acceptable = score >= 2 && password.len() >= self.min_password_length;
        
        PasswordStrength {
            score,
            feedback,
            acceptable,
        }
    }
    
    /// Check if user is locked out
    pub fn is_locked_out(&self, identifier: &str) -> bool {
        if let Some((attempts, last_attempt)) = self.failed_attempts.get(identifier) {
            if *attempts >= self.lockout_threshold {
                let lockout_end = *last_attempt + self.lockout_duration;
                if Utc::now() < lockout_end {
                    return true;
                }
            }
        }
        false
    }
    
    /// Record failed login attempt
    pub fn record_failed_attempt(&mut self, identifier: &str) {
        let entry = self.failed_attempts
            .entry(identifier.to_string())
            .or_insert((0, Utc::now()));
        
        entry.0 += 1;
        entry.1 = Utc::now();
    }
    
    /// Clear failed attempts on successful login
    pub fn clear_failed_attempts(&mut self, identifier: &str) {
        self.failed_attempts.remove(identifier);
    }
    
    /// Create new session
    pub fn create_session(
        &mut self, 
        user_id: &str, 
        duration_secs: u64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Session {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let session = Session {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + Duration::seconds(duration_secs as i64),
            ip_address,
            user_agent,
            is_active: true,
        };
        
        self.sessions.insert(session_id, session.clone());
        session
    }
    
    /// Validate session
    pub fn validate_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id).and_then(|session| {
            if session.is_active && session.expires_at > Utc::now() {
                Some(session)
            } else {
                None
            }
        })
    }
    
    /// Invalidate session
    pub fn invalidate_session(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.is_active = false;
            true
        } else {
            false
        }
    }
    
    /// Invalidate all sessions for user
    pub fn invalidate_all_sessions(&mut self, user_id: &str) {
        for session in self.sessions.values_mut() {
            if session.user_id == user_id {
                session.is_active = false;
            }
        }
    }
    
    /// Cleanup expired sessions
    pub fn cleanup_sessions(&mut self) {
        let now = Utc::now();
        self.sessions.retain(|_, session| {
            session.expires_at > now
        });
    }
    
    /// Get active sessions for user
    pub fn get_active_sessions(&self, user_id: &str) -> Vec<&Session> {
        let now = Utc::now();
        self.sessions.values()
            .filter(|s| s.user_id == user_id && s.is_active && s.expires_at > now)
            .collect()
    }
}

/// Generate secure session token
pub fn generate_session_token() -> String {
    use base64::Engine;
    
    let bytes = super::secure_random_bytes(32);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

/// Generate CSRF token
pub fn generate_csrf_token() -> String {
    use base64::Engine;
    
    let bytes = super::secure_random_bytes(32);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_hash_verify() {
        let auth = AuthManager::new(12);
        let password = "SecurePassword123!";
        
        let hash = auth.hash_password(password).unwrap();
        assert!(auth.verify_password(password, &hash).unwrap());
        assert!(!auth.verify_password("WrongPassword", &hash).unwrap());
    }
    
    #[test]
    fn test_password_strength() {
        let auth = AuthManager::new(12);
        
        let weak = auth.check_password_strength("password");
        assert!(!weak.acceptable);
        
        let strong = auth.check_password_strength("SecureP@ssw0rd123!");
        assert!(strong.acceptable);
        assert!(strong.score >= 3);
    }
    
    #[test]
    fn test_session_management() {
        let mut auth = AuthManager::new(12);
        
        let session = auth.create_session("user1", 3600, None, None);
        assert!(auth.validate_session(&session.id).is_some());
        
        auth.invalidate_session(&session.id);
        assert!(auth.validate_session(&session.id).is_none());
    }
}
