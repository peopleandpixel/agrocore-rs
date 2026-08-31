use crate::config::{EncryptionConfig, EncryptionMethod, TargetEncryption};
use crate::error::{BackupError, BackupResult};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use age::{Decryptor, Encryptor, secrecy::SecretString};
use rand_core::RngCore;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

pub struct EncryptionManager {
    default_method: EncryptionMethod,
    age_recipients: Vec<String>,
    aes_key: Option<[u8; 32]>,
}

impl EncryptionManager {
    pub fn new(config: EncryptionConfig) -> Self {
        let aes_key = if matches!(config.default, EncryptionMethod::Aes256Gcm) {
            // In production, this should come from a secure key management system
            // For now, generate a deterministic key from a passphrase (NOT SECURE FOR PRODUCTION)
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
                self.encrypt_with_age(input_path, output_path).await?;
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
                self.decrypt_with_age(input_path, output_path).await?;
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

    async fn encrypt_with_age(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let input_data = fs::read(input_path).map_err(|e| BackupError::Io(e))?;

        let recipients: Vec<Box<dyn age::Recipient>> = self
            .age_recipients
            .iter()
            .map(|r| {
                let recipient: age::Recipient = r
                    .parse()
                    .map_err(|e| BackupError::Encryption(format!("Invalid age recipient: {e}")))?;
                Ok(Box::new(recipient) as Box<dyn age::Recipient>)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let encryptor = Encryptor::with_recipients(
            recipients.iter().map(|r| r.as_ref()).collect::<Vec<_>>(),
        )
        .map_err(|e| BackupError::Encryption(format!("Failed to create age encryptor: {e}")))?;

        let mut encrypted = Vec::new();
        {
            let mut writer = encryptor.wrap_output(&mut encrypted).map_err(|e| {
                BackupError::Encryption(format!("Failed to create age writer: {e}"))
            })?;
            use std::io::Write;
            writer
                .write_all(&input_data)
                .map_err(|e| BackupError::Io(e))?;
            writer.finish().map_err(|e| {
                BackupError::Encryption(format!("Failed to finish age encryption: {e}"))
            })?;
        }

        fs::write(output_path, encrypted).map_err(|e| BackupError::Io(e))?;
        Ok(())
    }

    async fn decrypt_with_age(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let encrypted_data = fs::read(input_path).map_err(|e| BackupError::Io(e))?;

        // For decryption, we need the private key (identity)
        // This would typically come from a secure key store
        let identity_path = std::env::var("BACKUP_AGE_IDENTITY")
            .unwrap_or_else(|_| "backup-identity.txt".to_string());

        let identity = age::scrypt::Identity::new(SecretString::new(identity_path.into()));

        let decryptor = Decryptor::new(&encrypted_data[..])
            .map_err(|e| BackupError::Encryption(format!("Failed to create age decryptor: {e}")))?;

        let mut decrypted = Vec::new();
        {
            let identity_trait: &dyn age::Identity = &identity;
            let mut reader = decryptor
                .decrypt(std::iter::once(identity_trait))
                .map_err(|e| BackupError::Encryption(format!("Failed to decrypt: {e}")))?;
            use std::io::Read;
            reader
                .read_to_end(&mut decrypted)
                .map_err(|e| BackupError::Io(e))?;
        }

        fs::write(output_path, decrypted).map_err(|e| BackupError::Io(e))?;
        Ok(())
    }

    async fn encrypt_with_aes(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        let key = self
            .aes_key
            .ok_or_else(|| BackupError::Encryption("AES key not configured".to_string()))?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let input_data = fs::read(input_path).map_err(|e| BackupError::Io(e))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| BackupError::Encryption(format!("AES encryption failed: {e}")))?;

        // Write nonce + ciphertext
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
