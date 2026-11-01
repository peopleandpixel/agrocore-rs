use crate::config::{EncryptionConfig, EncryptionMethod, TargetEncryption};
use crate::error::{BackupError, BackupResult};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use age::{Decryptor, Encryptor, x25519};
use rand_core::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct EncryptionManager {
    default_method: EncryptionMethod,
    /// Age recipients parsed into Recipient instances for encryption.
    age_recipients: Vec<x25519::Recipient>,
    /// Age identities for decryption.
    age_identities: Vec<x25519::Identity>,
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

        // Parse Age recipients (public keys starting with "age1...")
        let age_recipients: Vec<x25519::Recipient> = config
            .age_recipients
            .iter()
            .filter_map(|r| r.parse().ok())
            .collect();

        // Parse Age identities (secret keys starting with "AGE-SECRET-KEY-1...")
        let age_identities: Vec<x25519::Identity> = config
            .age_recipients
            .iter()
            .filter_map(|r| <x25519::Identity as std::str::FromStr>::from_str(r).ok())
            .collect();

        Self {
            default_method: config.default,
            age_recipients,
            age_identities,
            aes_key,
        }
    }

    pub async fn encrypt_file(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        match self.default_method {
            EncryptionMethod::None => {
                fs::copy(input_path, output_path).map_err(BackupError::Io)?;
            }
            EncryptionMethod::Age => {
                self.encrypt_with_age(input_path, output_path)?;
            }
            EncryptionMethod::Aes256Gcm => {
                self.encrypt_with_aes(input_path, output_path).await?;
            }
            EncryptionMethod::AwsKms => {
                self.encrypt_with_aws_kms(input_path, output_path).await?;
            }
            EncryptionMethod::AzureKeyVault => {
                self.encrypt_with_azure_kv(input_path, output_path).await?;
            }
            EncryptionMethod::GcpKms => {
                self.encrypt_with_gcp_kms(input_path, output_path).await?;
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
                self.decrypt_with_age(input_path, output_path)?;
            }
            EncryptionMethod::Aes256Gcm => {
                self.decrypt_with_aes(input_path, output_path).await?;
            }
            EncryptionMethod::AwsKms => {
                self.decrypt_with_aws_kms(input_path, output_path).await?;
            }
            EncryptionMethod::AzureKeyVault => {
                self.decrypt_with_azure_kv(input_path, output_path).await?;
            }
            EncryptionMethod::GcpKms => {
                self.decrypt_with_gcp_kms(input_path, output_path).await?;
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

    /// Encrypt a file using Age with the configured recipients.
    fn encrypt_with_age(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        if self.age_recipients.is_empty() {
            return Err(BackupError::Encryption(
                "Age encryption requires at least one recipient".to_string(),
            ));
        }

        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

        let recipients: Vec<Box<dyn age::Recipient>> = self
            .age_recipients
            .iter()
            .map(|r| -> Box<dyn age::Recipient> { Box::new(r.clone()) })
            .collect();
        let encryptor = Encryptor::with_recipients(
            recipients.iter().map(|r| r.as_ref() as &dyn age::Recipient),
        )
        .map_err(|e| BackupError::Encryption(format!("Age encryptor creation failed: {e}")))?;

        let mut output_file = fs::File::create(output_path).map_err(BackupError::Io)?;
        let mut enc_writer = encryptor
            .wrap_output(&mut output_file)
            .map_err(|e| BackupError::Encryption(format!("Age wrap_output failed: {e}")))?;

        enc_writer
            .write_all(&input_data)
            .map_err(|e| BackupError::Encryption(format!("Age write failed: {e}")))?;
        enc_writer
            .finish()
            .map_err(|e| BackupError::Encryption(format!("Age finish failed: {e}")))?;

        Ok(())
    }

    /// Decrypt a file using Age with the configured identities.
    fn decrypt_with_age(&self, input_path: &Path, output_path: &Path) -> BackupResult<()> {
        if self.age_identities.is_empty() {
            return Err(BackupError::Encryption(
                "Age decryption requires at least one valid identity (private key)".to_string(),
            ));
        }

        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

        let decryptor = Decryptor::new(&input_data[..])
            .map_err(|e| BackupError::Encryption(format!("Age decryptor creation failed: {e}")))?;

        let mut output_file = fs::File::create(output_path).map_err(BackupError::Io)?;
        let identities: Vec<Box<dyn age::Identity>> = self
            .age_identities
            .iter()
            .map(|id| -> Box<dyn age::Identity> { Box::new(id.clone()) })
            .collect();
        let mut dec_reader = decryptor
            .decrypt(
                identities
                    .iter()
                    .map(|id| id.as_ref() as &dyn age::Identity),
            )
            .map_err(|e| BackupError::Encryption(format!("Age decrypt failed: {e}")))?;

        let mut buffer = [0u8; 8192];
        loop {
            let n = dec_reader
                .read(&mut buffer)
                .map_err(|e| BackupError::Encryption(format!("Age read failed: {e}")))?;
            if n == 0 {
                break;
            }
            output_file
                .write_all(&buffer[..n])
                .map_err(|e| BackupError::Encryption(format!("Age output write failed: {e}")))?;
        }

        Ok(())
    }

    /// Encrypt using AWS KMS envelope encryption.
    ///
    /// Generates a random data key, encrypts it with AWS KMS, and uses the
    /// data key with AES-256-GCM for the actual payload.
    async fn encrypt_with_aws_kms(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let key_id = std::env::var("AWS_KMS_KEY_ID")
            .or_else(|_| std::env::var("BACKUP_KMS_KEY_ID"))
            .unwrap_or_else(|_| "alias/agrocore-backup".to_string());

        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

        // Generate a random data key
        let mut data_key = [0u8; 32];
        OsRng
            .try_fill_bytes(&mut data_key)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate data key: {e}")))?;

        // Encrypt the data key with AWS KMS
        let encrypted_key = encrypt_data_key_with_aws_kms(&data_key, &key_id).await?;

        // Encrypt the payload with the data key
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate nonce: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| BackupError::Encryption(format!("AES encryption failed: {e}")))?;

        // Write: encrypted_key_len (4 bytes) | encrypted_key | nonce (12 bytes) | ciphertext
        let mut output = Vec::new();
        output.extend_from_slice(&(encrypted_key.len() as u32).to_le_bytes());
        output.extend_from_slice(&encrypted_key);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        fs::write(output_path, output).map_err(BackupError::Io)?;
        Ok(())
    }

    /// Decrypt using AWS KMS envelope encryption.
    async fn decrypt_with_aws_kms(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let encrypted_data = fs::read(input_path).map_err(BackupError::Io)?;
        if encrypted_data.len() < 16 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: too short".to_string(),
            ));
        }

        let key_len = u32::from_le_bytes(encrypted_data[..4].try_into().unwrap()) as usize;
        let encrypted_key = &encrypted_data[4..4 + key_len];
        let rest = &encrypted_data[4 + key_len..];
        if rest.len() < 12 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: missing nonce".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = rest.split_at(12);

        // Decrypt the data key with AWS KMS
        let data_key = decrypt_data_key_with_aws_kms(encrypted_key).await?;

        // Decrypt the payload
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| BackupError::Encryption(format!("AES decryption failed: {e}")))?;

        fs::write(output_path, plaintext).map_err(BackupError::Io)?;
        Ok(())
    }

    /// Encrypt using Azure Key Vault envelope encryption.
    async fn encrypt_with_azure_kv(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

        let mut data_key = [0u8; 32];
        OsRng
            .try_fill_bytes(&mut data_key)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate data key: {e}")))?;

        let encrypted_key = encrypt_data_key_with_azure_kv(&data_key).await?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate nonce: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| BackupError::Encryption(format!("AES encryption failed: {e}")))?;

        let mut output = Vec::new();
        output.extend_from_slice(&(encrypted_key.len() as u32).to_le_bytes());
        output.extend_from_slice(&encrypted_key);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        fs::write(output_path, output).map_err(BackupError::Io)?;
        Ok(())
    }

    /// Decrypt using Azure Key Vault envelope encryption.
    async fn decrypt_with_azure_kv(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let encrypted_data = fs::read(input_path).map_err(BackupError::Io)?;
        if encrypted_data.len() < 16 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: too short".to_string(),
            ));
        }

        let key_len = u32::from_le_bytes(encrypted_data[..4].try_into().unwrap()) as usize;
        let encrypted_key = &encrypted_data[4..4 + key_len];
        let rest = &encrypted_data[4 + key_len..];
        if rest.len() < 12 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: missing nonce".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = rest.split_at(12);
        let data_key = decrypt_data_key_with_azure_kv(encrypted_key).await?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| BackupError::Encryption(format!("AES decryption failed: {e}")))?;

        fs::write(output_path, plaintext).map_err(BackupError::Io)?;
        Ok(())
    }

    /// Encrypt using GCP KMS envelope encryption.
    async fn encrypt_with_gcp_kms(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let input_data = fs::read(input_path).map_err(BackupError::Io)?;

        let mut data_key = [0u8; 32];
        OsRng
            .try_fill_bytes(&mut data_key)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate data key: {e}")))?;

        let encrypted_key = encrypt_data_key_with_gcp_kms(&data_key).await?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| BackupError::Encryption(format!("Failed to generate nonce: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, input_data.as_ref())
            .map_err(|e| BackupError::Encryption(format!("AES encryption failed: {e}")))?;

        let mut output = Vec::new();
        output.extend_from_slice(&(encrypted_key.len() as u32).to_le_bytes());
        output.extend_from_slice(&encrypted_key);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);

        fs::write(output_path, output).map_err(BackupError::Io)?;
        Ok(())
    }

    /// Decrypt using GCP KMS envelope encryption.
    async fn decrypt_with_gcp_kms(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> BackupResult<()> {
        let encrypted_data = fs::read(input_path).map_err(BackupError::Io)?;
        if encrypted_data.len() < 16 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: too short".to_string(),
            ));
        }

        let key_len = u32::from_le_bytes(encrypted_data[..4].try_into().unwrap()) as usize;
        let encrypted_key = &encrypted_data[4..4 + key_len];
        let rest = &encrypted_data[4 + key_len..];
        if rest.len() < 12 {
            return Err(BackupError::Encryption(
                "Invalid encrypted file: missing nonce".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = rest.split_at(12);
        let data_key = decrypt_data_key_with_gcp_kms(encrypted_key).await?;

        let cipher = Aes256Gcm::new(GenericArray::from_slice(&data_key));
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

// ── KMS helper functions ──────────────────────────────────────────────

/// Encrypt a data key with AWS KMS using EncryptKeyMaterial API.
async fn encrypt_data_key_with_aws_kms(data_key: &[u8; 32], key_id: &str) -> BackupResult<Vec<u8>> {
    use aws_sdk_kms::{Client as KmsClient, Config as KmsConfig};

    let region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());

    let config = KmsConfig::builder()
        .region(aws_sdk_kms::config::Region::new(region))
        .build();
    let client = KmsClient::from_conf(config);

    let resp = client
        .encrypt()
        .key_id(key_id)
        .plaintext(data_key.to_vec().into())
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("AWS KMS encrypt failed: {e}")))?;

    let encrypted = resp
        .ciphertext_blob()
        .ok_or_else(|| BackupError::Encryption("AWS KMS returned no ciphertext".to_string()))?;

    Ok(encrypted.as_ref().to_vec())
}

/// Decrypt a data key with AWS KMS.
async fn decrypt_data_key_with_aws_kms(encrypted_key: &[u8]) -> BackupResult<[u8; 32]> {
    use aws_sdk_kms::{Client as KmsClient, Config as KmsConfig};

    let region = std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string());

    let config = KmsConfig::builder()
        .region(aws_sdk_kms::config::Region::new(region))
        .build();
    let client = KmsClient::from_conf(config);

    let resp = client
        .decrypt()
        .ciphertext_blob(encrypted_key.to_vec().into())
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("AWS KMS decrypt failed: {e}")))?;

    let plaintext = resp
        .plaintext()
        .ok_or_else(|| BackupError::Encryption("AWS KMS returned no plaintext".to_string()))?;

    let bytes = plaintext.as_ref();
    if bytes.len() != 32 {
        return Err(BackupError::Encryption(format!(
            "AWS KMS returned invalid key length: {}",
            bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(bytes);
    Ok(key)
}

/// Encrypt a data key with Azure Key Vault.
async fn encrypt_data_key_with_azure_kv(data_key: &[u8; 32]) -> BackupResult<Vec<u8>> {
    let vault_url = std::env::var("AZURE_KEY_VAULT_URL")
        .map_err(|_| BackupError::Encryption("AZURE_KEY_VAULT_URL not set".to_string()))?;
    let key_name =
        std::env::var("AZURE_KEY_VAULT_KEY_NAME").unwrap_or_else(|_| "agrocore-backup".to_string());

    let token = azure_get_access_token().await?;

    let url = format!(
        "{}/keys/{}/encrypt/RSA-OAEP?api-version=7.3",
        vault_url, key_name
    );
    let payload = serde_json::json!({
        "value": base64_encode(data_key)
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure KV request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(BackupError::Encryption(format!(
            "Azure KV encrypt failed: {} - {}",
            status, text
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure KV JSON parse: {e}")))?;

    let encrypted_b64 = body["encrypted"]
        .as_str()
        .ok_or_else(|| BackupError::Encryption("Azure KV missing encrypted field".to_string()))?;

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_b64)
        .map_err(|e| BackupError::Encryption(format!("Azure KV base64 decode: {e}")))
}

/// Decrypt a data key with Azure Key Vault.
async fn decrypt_data_key_with_azure_kv(encrypted_key: &[u8]) -> BackupResult<[u8; 32]> {
    let vault_url = std::env::var("AZURE_KEY_VAULT_URL")
        .map_err(|_| BackupError::Encryption("AZURE_KEY_VAULT_URL not set".to_string()))?;
    let key_name =
        std::env::var("AZURE_KEY_VAULT_KEY_NAME").unwrap_or_else(|_| "agrocore-backup".to_string());

    let token = azure_get_access_token().await?;

    let url = format!(
        "{}/keys/{}/decrypt/RSA-OAEP?api-version=7.3",
        vault_url, key_name
    );
    let payload = serde_json::json!({
        "value": base64_encode(encrypted_key)
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure KV request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(BackupError::Encryption(format!(
            "Azure KV decrypt failed: {} - {}",
            status, text
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure KV JSON parse: {e}")))?;

    let decrypted_b64 = body["result"]
        .as_str()
        .ok_or_else(|| BackupError::Encryption("Azure KV missing result field".to_string()))?;

    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, decrypted_b64)
        .map_err(|e| BackupError::Encryption(format!("Azure KV base64 decode: {e}")))?;

    if bytes.len() != 32 {
        return Err(BackupError::Encryption(format!(
            "Azure KV returned invalid key length: {}",
            bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Get Azure access token for Key Vault.
async fn azure_get_access_token() -> BackupResult<String> {
    let tenant_id = std::env::var("AZURE_TENANT_ID")
        .map_err(|_| BackupError::Encryption("AZURE_TENANT_ID not set".to_string()))?;
    let client_id = std::env::var("AZURE_CLIENT_ID")
        .map_err(|_| BackupError::Encryption("AZURE_CLIENT_ID not set".to_string()))?;
    let client_secret = std::env::var("AZURE_CLIENT_SECRET")
        .map_err(|_| BackupError::Encryption("AZURE_CLIENT_SECRET not set".to_string()))?;

    let url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    );
    let client = reqwest::Client::new();

    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}&scope=https://vault.azure.net/.default",
        client_id, client_secret
    );
    let resp = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure token request failed: {e}")))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BackupError::Encryption(format!("Azure token JSON parse: {e}")))?;

    body["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| BackupError::Encryption("Azure token missing access_token".to_string()))
}

/// Encrypt a data key with GCP KMS.
async fn encrypt_data_key_with_gcp_kms(data_key: &[u8; 32]) -> BackupResult<Vec<u8>> {
    let key_name = std::env::var("GCP_KMS_KEY_NAME")
        .map_err(|_| BackupError::Encryption("GCP_KMS_KEY_NAME not set".to_string()))?;

    let token = gcp_get_access_token().await?;

    let url = format!("https://cloudkms.googleapis.com/v1/{}/:encrypt", key_name);
    let plaintext_b64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data_key);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "plaintext": plaintext_b64 }))
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("GCP KMS request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(BackupError::Encryption(format!(
            "GCP KMS encrypt failed: {} - {}",
            status, text
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BackupError::Encryption(format!("GCP KMS JSON parse: {e}")))?;

    let encrypted_b64 = body["ciphertext"]
        .as_str()
        .ok_or_else(|| BackupError::Encryption("GCP KMS missing ciphertext field".to_string()))?;

    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted_b64)
        .map_err(|e| BackupError::Encryption(format!("GCP KMS base64 decode: {e}")))
}

/// Decrypt a data key with GCP KMS.
async fn decrypt_data_key_with_gcp_kms(encrypted_key: &[u8]) -> BackupResult<[u8; 32]> {
    let key_name = std::env::var("GCP_KMS_KEY_NAME")
        .map_err(|_| BackupError::Encryption("GCP_KMS_KEY_NAME not set".to_string()))?;

    let token = gcp_get_access_token().await?;

    let url = format!("https://cloudkms.googleapis.com/v1/{}/:decrypt", key_name);
    let ciphertext_b64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, encrypted_key);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "ciphertext": ciphertext_b64 }))
        .send()
        .await
        .map_err(|e| BackupError::Encryption(format!("GCP KMS request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(BackupError::Encryption(format!(
            "GCP KMS decrypt failed: {} - {}",
            status, text
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| BackupError::Encryption(format!("GCP KMS JSON parse: {e}")))?;

    let decrypted_b64 = body["plaintext"]
        .as_str()
        .ok_or_else(|| BackupError::Encryption("GCP KMS missing plaintext field".to_string()))?;

    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, decrypted_b64)
        .map_err(|e| BackupError::Encryption(format!("GCP KMS base64 decode: {e}")))?;

    if bytes.len() != 32 {
        return Err(BackupError::Encryption(format!(
            "GCP KMS returned invalid key length: {}",
            bytes.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Get GCP access token.
async fn gcp_get_access_token() -> BackupResult<String> {
    let client = reqwest::Client::new();

    // Try metadata server first (for GCE/GKE)
    let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
    let resp = client
        .get(metadata_url)
        .header("Metadata-Flavor", "Google")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    if let Ok(resp) = resp
        && resp.status().is_success()
    {
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BackupError::Encryption(format!("GCP metadata JSON parse: {e}")))?;
        if let Some(token) = body["access_token"].as_str() {
            return Ok(token.to_string());
        }
    }

    // Fall back to Application Default Credentials
    Err(BackupError::Encryption(
        "GCP credentials not available: not running on GCE/GKE and no ADC configured".to_string(),
    ))
}

/// Helper to base64-encode bytes.
fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}
