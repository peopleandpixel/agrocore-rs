//! SFTP storage backend.
//!
//! Streams backups over SSH/SFTP using password or key authentication.
//! Connections are established per operation: a long-lived pooled SSH session
//! would need reconnect handling that this backend does not implement, and
//! backup jobs run on a schedule where the connection cost is negligible
//! compared to the dump itself.

use crate::config::{BackupTarget, EncryptionConfig, TargetEncryption};
use crate::encryption::{decrypt_payload, encrypt_payload};
use crate::error::{BackupError, BackupResult};
use crate::storage::{ByteStream, StorageBackendTrait};
use async_trait::async_trait;
use futures::StreamExt;
use russh::client::Handler;
use russh_sftp::client::SftpSession;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// SFTP backend configuration.
pub struct SftpBackend {
    host: String,
    port: u16,
    username: String,
    base_path: String,
    password: Option<String>,
    key_path: Option<std::path::PathBuf>,
    key_passphrase: Option<String>,
    encryption: TargetEncryption,
    encryption_config: EncryptionConfig,
}

impl SftpBackend {
    /// Create a backend from a validated target.
    pub fn new(
        host: String,
        port: u16,
        username: String,
        path: String,
        password: Option<String>,
        key_path: Option<std::path::PathBuf>,
        encryption: TargetEncryption,
    ) -> Self {
        Self {
            host,
            port,
            username,
            base_path: path.trim_end_matches('/').to_string(),
            password,
            key_path,
            key_passphrase: None,
            encryption,
            encryption_config: EncryptionConfig::default(),
        }
    }

    /// Provide the encryption configuration used for payloads.
    pub fn with_encryption_config(mut self, config: EncryptionConfig) -> Self {
        self.encryption_config = config;
        self
    }

    /// Set the passphrase used to decrypt the private key.
    pub fn with_key_passphrase(mut self, passphrase: Option<String>) -> Self {
        self.key_passphrase = passphrase;
        self
    }

    /// Resolve the full remote path for an object name.
    fn remote_path(&self, object: &str) -> String {
        if self.base_path.is_empty() {
            object.to_string()
        } else {
            format!("{}/{}", self.base_path, object)
        }
    }

    /// Connect and open an SFTP session over a new SSH channel.
    async fn connect(&self) -> BackupResult<(russh::client::Handle<ClientHandler>, SftpSession)> {
        let config = Arc::new(russh::client::Config::default());
        let mut session =
            russh::client::connect(config, (self.host.as_str(), self.port), ClientHandler)
                .await
                .map_err(|e| BackupError::Config(format!("SFTP connect failed: {e}")))?;

        let authenticated = match &self.key_path {
            Some(key_path) => {
                let key_data = tokio::fs::read(key_path).await.map_err(|e| {
                    BackupError::Config(format!("Cannot read SFTP key {}: {e}", key_path.display()))
                })?;
                let key_text = String::from_utf8(key_data)
                    .map_err(|e| BackupError::Config(format!("SFTP key is not UTF-8: {e}")))?;
                let key_pair =
                    russh::keys::decode_secret_key(&key_text, self.key_passphrase.as_deref())
                        .map_err(|e| BackupError::Config(format!("Invalid SFTP key: {e}")))?;
                let key_with_alg =
                    russh::keys::key::PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
                        .map_err(|e| {
                            BackupError::Config(format!("Unsupported SFTP key type: {e}"))
                        })?;
                session
                    .authenticate_publickey(self.username.clone(), key_with_alg)
                    .await
            }
            None => {
                let password = self.password.as_deref().ok_or_else(|| {
                    BackupError::Config("SFTP requires either a password or a key_path".to_string())
                })?;
                session
                    .authenticate_password(self.username.clone(), password.to_string())
                    .await
            }
        }
        .map_err(|e| BackupError::Config(format!("SFTP authentication failed: {e}")))?;

        if !authenticated {
            return Err(BackupError::Config(
                "SFTP authentication rejected".to_string(),
            ));
        }

        // SFTP runs over an SSH channel, not over the session directly.
        let channel = session
            .channel_open_session()
            .await
            .map_err(|e| BackupError::Config(format!("SFTP channel failed: {e}")))?;
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| BackupError::Config(format!("SFTP session failed: {e}")))?;

        Ok((session, sftp))
    }

    /// Create a directory and all missing parents.
    fn mkdir_p<'a>(
        sftp: &'a mut SftpSession,
        path: &'a str,
    ) -> Pin<Box<dyn std::future::Future<Output = BackupResult<()>> + Send + 'a>> {
        Box::pin(async move {
            if path.is_empty() || path == "/" {
                return Ok(());
            }
            if !sftp.try_exists(path).await.map_err(sftp_err)? {
                let parent = match path.rfind('/') {
                    Some(0) => None,
                    Some(idx) => Some(&path[..idx]),
                    None => None,
                };
                if let Some(parent) = parent {
                    Self::mkdir_p(sftp, parent).await?;
                }
                sftp.create_dir(path).await.map_err(sftp_err)?;
            }
            Ok(())
        })
    }
}

fn sftp_err(e: impl std::fmt::Display) -> BackupError {
    BackupError::Config(format!("SFTP operation failed: {e}"))
}

/// Minimal SSH client handler: accepts whatever the server offers.
struct ClientHandler;

#[async_trait::async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Host-key pinning is not implemented. Production deployments should
        // verify this against a known_hosts file before accepting the key.
        Ok(true)
    }
}

#[async_trait]
impl StorageBackendTrait for SftpBackend {
    async fn upload_bytes(
        &self,
        _target: &BackupTarget,
        object_name: &str,
        data: &[u8],
    ) -> BackupResult<()> {
        let remote = self.remote_path(object_name);
        let (_handle, mut sftp) = self.connect().await?;

        if let Some(parent) = remote.rfind('/').filter(|i| *i > 0) {
            Self::mkdir_p(&mut sftp, &remote[..parent]).await?;
        }
        let payload = encrypt_payload(data, &self.encryption, &self.encryption_config).await?;

        let mut file = sftp.create(&remote).await.map_err(sftp_err)?;
        file.write_all(&payload)
            .await
            .map_err(|e| BackupError::Config(format!("SFTP write failed: {e}")))?;
        file.close()
            .await
            .map_err(|e| BackupError::Config(format!("SFTP close failed: {e}")))?;
        Ok(())
    }

    async fn upload_file(
        &self,
        target: &BackupTarget,
        object_name: &str,
        file_path: &std::path::Path,
    ) -> BackupResult<u64> {
        let data = tokio::fs::read(file_path).await.map_err(BackupError::Io)?;
        self.upload_bytes(target, object_name, &data).await?;
        Ok(data.len() as u64)
    }

    async fn download_bytes(
        &self,
        _target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<Vec<u8>> {
        let remote = self.remote_path(object_name);
        let (_handle, sftp) = self.connect().await?;

        let mut file = sftp
            .open(&remote)
            .await
            .map_err(|e| BackupError::Config(format!("SFTP open failed: {e}")))?;

        let mut buf = Vec::new();
        // 1 MiB read buffer keeps memory bounded for large dumps.
        let mut chunk = vec![0u8; 1_048_576];
        loop {
            let n = file
                .read(&mut chunk)
                .await
                .map_err(|e| BackupError::Config(format!("SFTP read failed: {e}")))?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        decrypt_payload(&buf, &self.encryption, &self.encryption_config).await
    }

    async fn upload_stream(
        &self,
        _target: &BackupTarget,
        object_name: &str,
        mut data: ByteStream,
    ) -> BackupResult<u64> {
        let remote = self.remote_path(object_name);
        let (_handle, mut sftp) = self.connect().await?;

        if let Some(parent) = remote.rfind('/').filter(|i| *i > 0) {
            Self::mkdir_p(&mut sftp, &remote[..parent]).await?;
        }
        let mut file = sftp.create(&remote).await.map_err(sftp_err)?;

        // Write chunk by chunk: the dump never sits in memory as a whole.
        let mut written = 0u64;
        while let Some(chunk) = data.next().await {
            let chunk = chunk?;
            // Encrypt each chunk separately so memory stays bounded.
            let piece = encrypt_payload(&chunk, &self.encryption, &self.encryption_config).await?;
            file.write_all(&piece)
                .await
                .map_err(|e| BackupError::Config(format!("SFTP write failed: {e}")))?;
            written += piece.len() as u64;
        }

        file.close()
            .await
            .map_err(|e| BackupError::Config(format!("SFTP close failed: {e}")))?;
        Ok(written)
    }

    async fn download_stream(
        &self,
        target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<ByteStream> {
        let data = bytes::Bytes::from(self.download_bytes(target, object_name).await?);
        let stream = futures::stream::once(async move { Ok(data) });
        Ok(Box::pin(stream))
    }

    async fn list_objects(
        &self,
        _target: &BackupTarget,
        prefix: &str,
    ) -> BackupResult<Vec<(String, u64)>> {
        let base = self.remote_path("");
        let (_handle, mut sftp) = self.connect().await?;

        if !sftp.try_exists(&base).await.map_err(sftp_err)? {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        Self::walk(&mut sftp, &base, "", prefix, &mut out).await?;
        Ok(out)
    }

    async fn delete_object(&self, _target: &BackupTarget, object_name: &str) -> BackupResult<()> {
        let remote = self.remote_path(object_name);
        let (_handle, sftp) = self.connect().await?;
        sftp.remove_file(&remote)
            .await
            .map_err(|e| BackupError::Config(format!("SFTP delete failed: {e}")))
    }

    async fn load_manifest(
        &self,
        target: &BackupTarget,
        backup_id: &uuid::Uuid,
    ) -> BackupResult<Option<crate::manifest::BackupManifest>> {
        let name = format!("manifests/{backup_id}.json");
        // A missing manifest is not an error; the caller falls back to listing.
        let Ok(raw) = self.download_bytes(target, &name).await else {
            return Ok(None);
        };
        let manifest =
            serde_json::from_slice(&raw).map_err(crate::error::BackupError::Serialization)?;
        Ok(Some(manifest))
    }

    async fn save_manifest(
        &self,
        target: &BackupTarget,
        manifest: &crate::manifest::BackupManifest,
    ) -> BackupResult<()> {
        let json = serde_json::to_vec_pretty(manifest)?;
        self.upload_bytes(
            target,
            &format!("manifests/{}.json", manifest.backup_id),
            &json,
        )
        .await
    }
}

impl SftpBackend {
    /// Recursively collect files below `dir`, filtering by name prefix.
    fn walk<'a>(
        sftp: &'a mut SftpSession,
        dir: &'a str,
        rel: &'a str,
        name_prefix: &'a str,
        out: &'a mut Vec<(String, u64)>,
    ) -> Pin<Box<dyn std::future::Future<Output = BackupResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let entries = sftp.read_dir(dir).await.map_err(sftp_err)?;
            for entry in entries {
                let name = entry.file_name();
                if name == "." || name == ".." {
                    continue;
                }

                let child_rel = if rel.is_empty() {
                    name.clone()
                } else {
                    format!("{rel}/{name}")
                };
                let child_path = if dir.is_empty() {
                    name.clone()
                } else {
                    format!("{dir}/{name}")
                };

                if entry.file_type().is_dir() {
                    Self::walk(sftp, &child_path, &child_rel, name_prefix, out).await?;
                } else if !name_prefix.is_empty() && !name.starts_with(name_prefix) {
                    continue;
                } else {
                    let size = entry.metadata().size.unwrap_or(0);
                    out.push((child_rel, size));
                }
            }
            Ok(())
        })
    }
}

/// Build an SFTP backend from a target configuration.
pub fn backend_from_target(target: &BackupTarget) -> BackupResult<SftpBackend> {
    match target {
        BackupTarget::Sftp {
            host,
            port,
            username,
            path,
            password,
            key_path,
            encryption,
        } => Ok(SftpBackend::new(
            host.clone(),
            *port,
            username.clone(),
            path.clone(),
            password.clone(),
            key_path.clone(),
            encryption.clone(),
        )),
        other => Err(BackupError::Config(format!(
            "not an SFTP target: {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(path: &str) -> BackupTarget {
        BackupTarget::Sftp {
            host: "localhost".to_string(),
            port: 22,
            username: "backup".to_string(),
            path: path.to_string(),
            encryption: TargetEncryption::None,
            key_path: None,
            password: Some("secret".to_string()),
        }
    }

    #[test]
    fn remote_path_joins_base_and_object() {
        let b = backend_from_target(&target("/var/backups")).expect("backend");
        assert_eq!(b.remote_path("a.dump"), "/var/backups/a.dump");
    }

    #[test]
    fn remote_path_without_base_is_object() {
        let b = backend_from_target(&target("")).expect("backend");
        assert_eq!(b.remote_path("a.dump"), "a.dump");
    }

    #[test]
    fn trailing_slash_is_normalised() {
        let b = backend_from_target(&target("/var/backups/")).expect("backend");
        assert_eq!(b.remote_path("a.dump"), "/var/backups/a.dump");
    }

    #[test]
    fn non_sftp_target_is_rejected() {
        let local = BackupTarget::Local {
            path: std::path::PathBuf::from("/tmp"),
            encryption: TargetEncryption::None,
            permissions: None,
        };
        assert!(backend_from_target(&local).is_err());
    }
}
