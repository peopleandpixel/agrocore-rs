//! Encryption utilities for sensitive data stored in the database.
//!
//! Uses AES-256-GCM to encrypt credentials (LPIS API keys, passwords, etc.)
//! before storing them in the `system_settings` table. The encryption key is
//! derived from the `CREDENTIALS_ENCRYPTION_KEY` environment variable.
//!
//! If the environment variable is not set, a default key is used for development,
//! but a warning is logged. In production, always set this variable to a strong
//! random key.

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine as _, engine::general_purpose::STANDARD as base64_engine};
use rand_core::RngCore;
use std::sync::OnceLock;

/// The encryption key, lazily initialized from the environment variable.
fn get_encryption_key() -> &'static [u8; 32] {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();
    
    KEY.get_or_init(|| {
        let passphrase = std::env::var("CREDENTIALS_ENCRYPTION_KEY")
            .unwrap_or_else(|_| {
                agrocore_logging::warn!(
                    "CREDENTIALS_ENCRYPTION_KEY not set! Using default key for development. \
                     Set this environment variable in production for secure credential storage."
                );
                "agrocore-default-credentials-key-32bytes!!".to_string()
            });
        
        // Derive a 32-byte key from the passphrase using SHA-256
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(passphrase.as_bytes());
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash);
        key
    })
}

/// Encrypt a plaintext string using AES-256-GCM.
///
/// Returns a base64-encoded string containing the nonce and ciphertext.
/// Format: `nonce (12 bytes) || ciphertext (variable) || tag (16 bytes)`
pub fn encrypt(plaintext: &str) -> String {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    
    // Generate a random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rand_core::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("Encryption failed");
    
    // Prepend nonce to ciphertext for storage
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    
    // Base64 encode for storage in JSON
    base64_engine.encode(&combined)
}

/// Decrypt a base64-encoded string that was encrypted with `encrypt()`.
///
/// Returns the plaintext string on success.
pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
    
    // Base64 decode
    let combined = base64_engine
        .decode(encrypted)
        .map_err(|e| format!("Failed to base64 decode: {e}"))?;
    
    // Extract nonce and ciphertext
    if combined.len() < 12 + 16 {
        return Err("Encrypted data too short".to_string());
    }
    
    let nonce_bytes = &combined[..12];
    let ciphertext = &combined[12..];
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // Decrypt
    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {e}"))?;
    
    // Convert to string
    String::from_utf8(plaintext_bytes)
        .map_err(|e| format!("Invalid UTF-8 in decrypted data: {e}"))
}

/// Check if a string looks like an encrypted credential.
///
/// Encrypted credentials are base64-encoded and at least 28 bytes long
/// (12-byte nonce + 16-byte tag, minimum).
pub fn is_encrypted(s: &str) -> bool {
    if s.len() < 28 {
        return false;
    }
    
    // Try to base64 decode; if it fails, it's not encrypted
    base64_engine.decode(s).is_ok()
}

/// Encrypt a credential value if it's not already encrypted.
///
/// This function is idempotent: if the value is already encrypted, it returns
/// the value unchanged. This prevents double-encryption when the function is
/// called multiple times.
pub fn encrypt_if_needed(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    
    // If it looks encrypted, assume it is and return as-is
    if is_encrypted(value) {
        value.to_string()
    } else {
        encrypt(value)
    }
}

/// Decrypt a credential value if it's encrypted.
///
/// If the value is not encrypted (plain text), returns it unchanged.
/// This allows backward compatibility with credentials that were stored
/// before encryption was enabled.
pub fn decrypt_if_encrypted(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(value.to_string());
    }
    
    // Try to decrypt; if it fails, assume it's plain text
    match decrypt(value) {
        Ok(plaintext) => Ok(plaintext),
        Err(_) => {
            // If decryption fails, assume it's plain text
            // This provides backward compatibility
            Ok(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "my-secret-api-key";
        let encrypted = encrypt(plaintext);
        
        // Encrypted value should be different from plaintext
        assert_ne!(plaintext, encrypted);
        
        // Decrypted value should match plaintext
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_encrypt_if_needed_idempotent() {
        let plaintext = "my-secret";
        let encrypted = encrypt_if_needed(plaintext);
        
        // Encrypting again should return the same value
        let encrypted_again = encrypt_if_needed(&encrypted);
        assert_eq!(encrypted, encrypted_again);
    }
    
    #[test]
    fn test_decrypt_if_encrypted_backward_compat() {
        let plaintext = "plain-text-value";
        
        // Plain text should be returned unchanged
        let result = decrypt_if_encrypted(plaintext).unwrap();
        assert_eq!(plaintext, result);
    }
    
    #[test]
    fn test_is_encrypted() {
        assert!(!is_encrypted("short"));
        assert!(!is_encrypted("not-base64!!!"));
        
        let encrypted = encrypt("test");
        assert!(is_encrypted(&encrypted));
    }
}
