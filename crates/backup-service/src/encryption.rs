use crate::config::{EncryptionConfig, EncryptionMethod, TargetEncryption};
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
    /// Age recipients are captured from config and will be consumed once
    /// `EncryptionMethod::Age` is implemented (currently returns an error).
    #[allow(dead_code)]
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
                fs::copy(input_path, output_path).map_err(BackupError::Io)?;
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
                fs::copy(input_path, output_path).map_err(BackupError::Io)?;
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

        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

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

        fs::write(output_path, output).map_err(BackupError::Io)?;
        Ok(())
    }

    async fn decrypt_with_aes(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let key = self
            .aes_key
            .ok_or_else(|| BackupError::Encryption("AES key not configured".to_string()))?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let encrypted_data = fs::read(input_path).map_err(BackupError::Io)?;

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

        fs::write(output_path, plaintext).map_err(BackupError::Io)?;
        Ok(())
    }
}

/// Encrypt bytes according to the target's encryption setting.
///
/// The existing encryption API works on files, so in-memory payloads go
/// through a temporary file. Upload paths that stream chunk by chunk
/// encrypt each chunk separately, which keeps memory bounded.
pub async fn encrypt_payload(
    data: &[u8],
    encryption: &TargetEncryption,
    config: &EncryptionConfig,
) -> BackupResult<Vec<u8>> {
    let manager = EncryptionManager::new(config.clone());
    if matches!(encryption, TargetEncryption::None) {
        return Ok(data.to_vec());
    }

    let dir = tempfile::tempdir().map_err(BackupError::Io)?;
    let plain = dir.path().join("plain.bin");
    let cipher = dir.path().join("cipher.bin");
    tokio::fs::write(&plain, data)
        .await
        .map_err(BackupError::Io)?;
    manager.encrypt_file(&plain, &cipher).await?;
    tokio::fs::read(&cipher).await.map_err(BackupError::Io)
}

/// Reverse of [`encrypt_payload`].
pub async fn decrypt_payload(
    data: &[u8],
    encryption: &TargetEncryption,
    config: &EncryptionConfig,
) -> BackupResult<Vec<u8>> {
    let manager = EncryptionManager::new(config.clone());
    if matches!(encryption, TargetEncryption::None) {
        return Ok(data.to_vec());
    }

    let dir = tempfile::tempdir().map_err(BackupError::Io)?;
    let cipher = dir.path().join("cipher.bin");
    let plain = dir.path().join("plain.bin");
    tokio::fs::write(&cipher, data)
        .await
        .map_err(BackupError::Io)?;
    manager.decrypt_file(&cipher, &plain).await?;
    tokio::fs::read(&plain).await.map_err(BackupError::Io)
}
