//! Secure memory handling

use std::ops::{Deref, DerefMut};
use zeroize::Zeroize;

/// Secure buffer that zeros memory on drop
#[derive(Clone)]
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }
    
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            data: slice.to_vec(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.data.zeroize();
    }
}

impl Deref for SecureBuffer {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for SecureBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Secure string that zeros memory on drop
#[derive(Clone)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    pub fn new(s: &str) -> Self {
        Self {
            data: s.to_string(),
        }
    }
    
    pub fn from_string(s: String) -> Self {
        Self { data: s }
    }
    
    pub fn as_str(&self) -> &str {
        &self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero the string data
        unsafe {
            let bytes = self.data.as_bytes_mut();
            bytes.zeroize();
        }
    }
}

impl Deref for SecureString {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Locked memory region (prevents paging to disk on supported systems)
#[cfg(unix)]
pub struct LockedMemory {
    data: Vec<u8>,
    locked: bool,
}

#[cfg(unix)]
impl LockedMemory {
    pub fn new(size: usize) -> Self {
        let mut mem = Self {
            data: vec![0u8; size],
            locked: false,
        };
        mem.lock();
        mem
    }
    
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut mem = Self {
            data: slice.to_vec(),
            locked: false,
        };
        mem.lock();
        mem
    }
    
    fn lock(&mut self) {
        #[cfg(target_os = "linux")]
        unsafe {
            let ptr = self.data.as_ptr() as *const libc::c_void;
            let len = self.data.len();
            if libc::mlock(ptr, len) == 0 {
                self.locked = true;
            }
        }
    }
    
    fn unlock(&mut self) {
        if self.locked {
            #[cfg(target_os = "linux")]
            unsafe {
                let ptr = self.data.as_ptr() as *const libc::c_void;
                let len = self.data.len();
                libc::munlock(ptr, len);
                self.locked = false;
            }
        }
    }
    
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

#[cfg(unix)]
impl Drop for LockedMemory {
    fn drop(&mut self) {
        self.data.zeroize();
        self.unlock();
    }
}

#[cfg(unix)]
impl Deref for LockedMemory {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(unix)]
impl DerefMut for LockedMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Non-unix fallback
#[cfg(not(unix))]
pub struct LockedMemory {
    data: Vec<u8>,
}

#[cfg(not(unix))]
impl LockedMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }
    
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            data: slice.to_vec(),
        }
    }
    
    pub fn is_locked(&self) -> bool {
        false  // Memory locking not supported
    }
}

#[cfg(not(unix))]
impl Drop for LockedMemory {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

#[cfg(not(unix))]
impl Deref for LockedMemory {
    type Target = [u8];
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(not(unix))]
impl DerefMut for LockedMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Constant-time comparison to prevent timing attacks
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

/// Secure random fill
pub fn secure_fill(buffer: &mut [u8]) {
    use ring::rand::{SecureRandom, SystemRandom};
    
    let rng = SystemRandom::new();
    rng.fill(buffer).expect("Failed to fill with random data");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_buffer() {
        let mut buf = SecureBuffer::new(32);
        secure_fill(&mut buf);
        assert_eq!(buf.len(), 32);
    }
    
    #[test]
    fn test_secure_string() {
        let s = SecureString::new("secret password");
        assert_eq!(s.as_str(), "secret password");
    }
    
    #[test]
    fn test_constant_time_compare() {
        let a = b"hello world";
        let b = b"hello world";
        let c = b"hello world!";
        let d = b"goodbye wor";
        
        assert!(constant_time_compare(a, b));
        assert!(!constant_time_compare(a, c));
        assert!(!constant_time_compare(a, d));
    }
}
