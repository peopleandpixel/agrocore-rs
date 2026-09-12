use crate::config::{EncryptionConfig, EncryptionMethod};
use crate::error::{BackupError, BackupResult};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand_core::RngCore;
use std::fs;
use std::path::Path;

pub struct EncryptionManager {
    default_method: EncryptionMethod,
    age_recipients: Vec<String>,
    aes_key: Option<[u8; 32]>,
}

impl EncryptionManager {
    pub fn new(config: EncryptionConfig) -> Self {
        let aes_key = if matches!(config.default, EncryptionMethod::Aes256Gcm) {
            let passphrase = std::env::var("BACKUP_AES_KEY")
                .unwrap_or_else(|_| "default-backup-key-change-in-production".to_string());
            let mut key = [0u8; 32];
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            passphrase.hash(&mut hasher);
            let hash = hasher.finish();
            for (i, byte) in hash.to_le_bytes().iter().enumerate() {
                if i < 32 {
                    key[i] = *byte;
                }
            }
            Some(key)
        } else {
            None
        };

        Self {
            default_method: config.default,
            age_recipients: config.age_recipients,
            aes_key,
        }
    }

    pub async fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        match self.default_method {
            EncryptionMethod::None => {
                fs::copy(input_path, output_path).map_err(|e| BackupError::Io(e))?;
            }
            EncryptionMethod::Age => {
                return Err(BackupError::Encryption(
                    "Age encryption not yet implemented".to_string(),
                ));
            }
            EncryptionMethod::Aes256Gcm => {
                self.encrypt_with_aes(input_path, output_path).await?;
            }
            EncryptionMethod::AwsKms => {
                return Err(BackupError::Encryption(
                    "AWS KMS encryption not yet implemented".to_string(),
                ));
            }
            EncryptionMethod::AzureKeyVault => {
                return Err(BackupError::Encryption(
                    "Azure Key Vault encryption not yet implemented".to_string(),
                ));
            }
            EncryptionMethod::GcpKms => {
                return Err(BackupError::Encryption(
                    "GCP KMS encryption not yet implemented".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub async fn decrypt_file(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        match self.default_method {
            EncryptionMethod::None => {
                fs::copy(input_path, output_path).map_err(|e| BackupError::Io(e))?;
            }
            EncryptionMethod::Age => {
                return Err(BackupError::Encryption(
                    "Age decryption not yet implemented".to_string(),
                ));
            }
            EncryptionMethod::Aes256Gcm => {
                self.decrypt_with_aes(input_path, output_path).await?;
            }
            _ => {
                return Err(BackupError::Encryption(
                    "Decryption not supported for this method".to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn encrypt_with_aes(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let key = self
            .aes_key
            .ok_or_else(|| BackupError::Encryption("AES key not configured".to_string()))?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let input_data = fs::read(input_path).map_err(|e| BackupError::Io(e))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate nonce: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| BackupError::Encryption(format!("AES encryption failed: {e}")))?;

        let mut output = Vec::with_capacity(12 + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        fs::write(output_path, output).map_err(|e| BackupError::Io(e))?;
        Ok(())
    }

    async fn decrypt_with_aes(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let key = self
            .aes_key
            .ok_or_else(|| BackupError::Encryption("AES key not configured".to_string()))?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let encrypted_data = fs::read(input_path).map_err(|e| BackupError::Io(e))?;

        if encrypted_data.len() < 12 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: too short".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| BackupError::Encryption(format!("AES decryption failed: {e}")))?;

        fs::write(output_path, plaintext).map_err(|e| BackupError::Io(e))?;
        Ok(())
    }
}
