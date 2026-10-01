use crate::config::BackupTarget;
use crate::error::{BackupError, BackupResult};
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use object_store::{ObjectStore, PutPayload, path::Path};
use std::sync::Arc;

/// Chunk size used when streaming to and from storage.
const STREAM_BUFFER_SIZE: usize = 1024 * 1024;
use futures::Stream;
use std::pin::Pin;
use tokio::io::AsyncWriteExt;

#[async_trait]
pub trait StorageBackendTrait: Send + Sync {
    async fn upload_bytes(
        &self,
        target: &BackupTarget,
        object_name: &str,
        data: &[u8],
    ) -> BackupResult<()>;
    async fn upload_file(
        &self,
        target: &BackupTarget,
        object_name: &str,
        file_path: &std::path::Path,
    ) -> BackupResult<u64>;
    async fn download_bytes(
        &self,
        target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<Vec<u8>>;

    /// Stream an object into storage without buffering it in memory.
    ///
    /// Required for database dumps, which can exceed available memory. The
    /// returned value is the number of bytes written.
    async fn upload_stream(
        &self,
        target: &BackupTarget,
        object_name: &str,
        data: ByteStream,
    ) -> BackupResult<u64>;

    /// Stream an object out of storage without buffering it in memory.
    async fn download_stream(
        &self,
        target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<ByteStream>;

    /// List objects under `prefix`, returning (name, size) pairs.
    async fn list_objects(
        &self,
        target: &BackupTarget,
        prefix: &str,
    ) -> BackupResult<Vec<(String, u64)>>;

    /// Delete an object. Used by retention cleanup.
    async fn delete_object(&self, target: &BackupTarget, object_name: &str) -> BackupResult<()>;

    /// Read a previously persisted manifest.
    async fn load_manifest(
        &self,
        target: &BackupTarget,
        backup_id: &uuid::Uuid,
    ) -> BackupResult<Option<crate::manifest::BackupManifest>>;

    /// Persist a manifest alongside the backup payload.
    async fn save_manifest(
        &self,
        target: &BackupTarget,
        manifest: &crate::manifest::BackupManifest,
    ) -> BackupResult<()>;
}

/// Byte stream used for streaming uploads and downloads.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, BackupError>> + Send>>;

pub struct StorageBackend {
    s3_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    azure_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    gcs_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    local_paths: Vec<(BackupTarget, std::path::PathBuf)>,
}

impl StorageBackend {
    /// Upload a byte stream via the object store's multipart API.
    ///
    /// The payload is never materialised in memory: chunks are uploaded as
    /// they arrive and the upload is only completed once the stream is
    /// exhausted, which keeps memory use bounded for arbitrarily large dumps.
    async fn upload_multipart(
        store: &dyn ObjectStore,
        object_name: &str,
        data: ByteStream,
    ) -> BackupResult<u64> {
        use futures::StreamExt;

        let path = Path::from(object_name);
        let mut upload = store
            .put_multipart(&path)
            .await
            .map_err(BackupError::ObjectStore)?;

        let mut data = data;
        let mut written: u64 = 0;
        while let Some(chunk) = data.next().await {
            let chunk = chunk?;
            if chunk.is_empty() {
                continue;
            }
            // Record the size before the chunk is consumed by put_part.
            written += chunk.len() as u64;
            upload
                .put_part(chunk.into())
                .await
                .map_err(BackupError::ObjectStore)?;
        }
        upload.complete().await.map_err(BackupError::ObjectStore)?;
        Ok(written)
    }

    /// Adapt already-materialised bytes to our stream type.
    ///
    /// `GetResult::bytes()` collects the object into memory; this wrapper only
    /// exists so the trait signature stays uniform across backends. Local and
    /// multipart paths stream instead and never call this.
    fn bytes_to_stream(bytes: Bytes) -> ByteStream {
        Box::pin(futures::stream::once(async move { Ok(bytes) }))
    }

    pub async fn new(targets: Vec<BackupTarget>) -> BackupResult<Self> {
        let mut s3_clients = Vec::new();
        let mut azure_clients = Vec::new();
        let mut gcs_clients = Vec::new();
        let mut local_paths = Vec::new();

        for target in targets {
            let target_for_storage = target.clone();
            match target {
                BackupTarget::S3 {
                    bucket,
                    region,
                    endpoint,
                    credentials,
                    ..
                } => {
                    let client = Self::create_s3_client(
                        &bucket,
                        &region,
                        endpoint.clone(),
                        credentials.clone(),
                    )
                    .await?;
                    s3_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                BackupTarget::MinIO {
                    bucket,
                    region,
                    endpoint,
                    credentials,
                    ..
                } => {
                    let client = Self::create_s3_client(
                        &bucket,
                        &region,
                        Some(endpoint.clone()),
                        credentials.clone(),
                    )
                    .await?;
                    s3_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                BackupTarget::B2 {
                    bucket,
                    region,
                    endpoint,
                    ref credentials,
                    ..
                } => {
                    let client = Self::create_s3_client(
                        &bucket,
                        &region,
                        Some(endpoint.clone()),
                        credentials.clone(),
                    )
                    .await?;
                    s3_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                BackupTarget::Wasabi {
                    bucket,
                    region,
                    ref credentials,
                    ..
                } => {
                    let client = Self::create_s3_client(
                        &bucket,
                        &region,
                        Some("s3.wasabisys.com".to_string()),
                        credentials.clone(),
                    )
                    .await?;
                    s3_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                BackupTarget::Local { path, .. } => {
                    tokio::fs::create_dir_all(&path)
                        .await
                        .map_err(BackupError::Io)?;
                    local_paths.push((target_for_storage.clone(), path.clone()));
                }
                BackupTarget::Azure {
                    account,
                    container,
                    credentials,
                    ..
                } => {
                    let client =
                        Self::create_azure_client(&account, &container, credentials.clone())
                            .await?;
                    azure_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                BackupTarget::Gcs {
                    bucket,
                    credentials_path,
                    ..
                } => {
                    let client = Self::create_gcs_client(&bucket, credentials_path.clone()).await?;
                    gcs_clients.push((
                        target_for_storage.clone(),
                        Arc::new(client) as Arc<dyn ObjectStore>,
                    ));
                }
                // SFTP and WebDAV are connection-based rather than
                // object-store based, so no client is constructed up front;
                // their backends are created lazily per operation.
                BackupTarget::Sftp { .. } | BackupTarget::WebDAV { .. } => {}
            }
        }

        Ok(Self {
            s3_clients,
            azure_clients,
            gcs_clients,
            local_paths,
        })
    }

    async fn create_s3_client(
        bucket: &str,
        region: &str,
        endpoint: Option<String>,
        credentials: Option<crate::config::S3Credentials>,
    ) -> BackupResult<Arc<dyn ObjectStore>> {
        use object_store::aws::AmazonS3Builder;
        use std::sync::Arc;

        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(bucket)
            .with_region(region);

        if let Some(endpoint) = endpoint {
            builder = builder.with_endpoint(endpoint);
        }

        if let Some(creds) = credentials {
            builder = builder
                .with_access_key_id(creds.access_key_id)
                .with_secret_access_key(creds.secret_access_key);
            if let Some(token) = creds.session_token {
                builder = builder.with_token(token);
            }
        }

        let client = builder.build().map_err(BackupError::ObjectStore)?;
        Ok(Arc::new(client))
    }

    async fn create_azure_client(
        account: &str,
        container: &str,
        credentials: Option<crate::config::AzureCredentials>,
    ) -> BackupResult<Arc<dyn ObjectStore>> {
        use object_store::azure::MicrosoftAzureBuilder;
        use std::sync::Arc;

        let mut builder = MicrosoftAzureBuilder::new()
            .with_account(account)
            .with_container_name(container);

        if let Some(creds) = credentials
            && let Some(key) = creds.account_key
        {
            builder = builder.with_access_key(key);
        }

        let client = builder.build().map_err(BackupError::ObjectStore)?;
        Ok(Arc::new(client))
    }

    async fn create_gcs_client(
        bucket: &str,
        credentials_path: Option<std::path::PathBuf>,
    ) -> BackupResult<Arc<dyn ObjectStore>> {
        use object_store::gcp::GoogleCloudStorageBuilder;
        use std::sync::Arc;

        let mut builder = GoogleCloudStorageBuilder::new().with_bucket_name(bucket);

        if let Some(path) = credentials_path {
            builder = builder.with_service_account_path(path.to_string_lossy().to_string());
        }

        let client = builder.build().map_err(BackupError::ObjectStore)?;
        Ok(Arc::new(client))
    }

    fn find_s3_store(
        &self,
        target: &BackupTarget,
    ) -> Option<(&BackupTarget, Arc<dyn ObjectStore>)> {
        self.s3_clients
            .iter()
            .find(|(t, _)| t.target_id() == target.target_id())
            .map(|(t, s)| (t, s.clone()))
    }

    fn find_azure_store(
        &self,
        target: &BackupTarget,
    ) -> Option<(&BackupTarget, Arc<dyn ObjectStore>)> {
        self.azure_clients
            .iter()
            .find(|(t, _)| t.target_id() == target.target_id())
            .map(|(t, s)| (t, s.clone()))
    }

    fn find_gcs_store(
        &self,
        target: &BackupTarget,
    ) -> Option<(&BackupTarget, Arc<dyn ObjectStore>)> {
        self.gcs_clients
            .iter()
            .find(|(t, _)| t.target_id() == target.target_id())
            .map(|(t, s)| (t, s.clone()))
    }

    fn find_local(&self, target: &BackupTarget) -> Option<(&BackupTarget, std::path::PathBuf)> {
        self.local_paths
            .iter()
            .find(|(t, _)| t.target_id() == target.target_id())
            .map(|(t, p)| (t, p.clone()))
    }
}

#[async_trait]
impl StorageBackendTrait for StorageBackend {
    async fn upload_bytes(
        &self,
        target: &BackupTarget,
        object_name: &str,
        data: &[u8],
    ) -> BackupResult<()> {
        match target {
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                if let Some((_, store)) = self.find_s3_store(target) {
                    let path = Path::from(object_name);
                    store
                        .put(&path, PutPayload::from(data.to_vec()))
                        .await
                        .map_err(BackupError::ObjectStore)?;
                }
            }
            BackupTarget::Azure { .. } => {
                if let Some((_, store)) = self.find_azure_store(target) {
                    let path = Path::from(object_name);
                    store
                        .put(&path, PutPayload::from(data.to_vec()))
                        .await
                        .map_err(BackupError::ObjectStore)?;
                }
            }
            BackupTarget::Gcs { .. } => {
                if let Some((_, store)) = self.find_gcs_store(target) {
                    let path = Path::from(object_name);
                    store
                        .put(&path, PutPayload::from(data.to_vec()))
                        .await
                        .map_err(BackupError::ObjectStore)?;
                }
            }
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let full_path = base.join(object_name);
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(BackupError::Io)?;
                }
                tokio::fs::write(full_path, data)
                    .await
                    .map_err(BackupError::Io)?;
            }
            _ => {
                return Err(BackupError::Config(format!(
                    "No storage backend configured for target: {}",
                    target.target_id()
                )));
            }
        }
        Ok(())
    }

    async fn upload_file(
        &self,
        target: &BackupTarget,
        object_name: &str,
        file_path: &std::path::Path,
    ) -> BackupResult<u64> {
        let data = tokio::fs::read(file_path).await.map_err(BackupError::Io)?;
        let size = data.len() as u64;
        self.upload_bytes(target, object_name, &data).await?;
        Ok(size)
    }

    async fn download_bytes(
        &self,
        target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<Vec<u8>> {
        match target {
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                if let Some((_, store)) = self.find_s3_store(target) {
                    let path = Path::from(object_name);
                    let result = store.get(&path).await.map_err(BackupError::ObjectStore)?;
                    let bytes = result.bytes().await.map_err(BackupError::ObjectStore)?;
                    Ok(bytes.to_vec())
                } else {
                    Err(BackupError::Config(format!(
                        "No S3 store for target: {}",
                        target.target_id()
                    )))
                }
            }
            BackupTarget::Azure { .. } => {
                if let Some((_, store)) = self.find_azure_store(target) {
                    let path = Path::from(object_name);
                    let result = store.get(&path).await.map_err(BackupError::ObjectStore)?;
                    let bytes = result.bytes().await.map_err(BackupError::ObjectStore)?;
                    Ok(bytes.to_vec())
                } else {
                    Err(BackupError::Config(format!(
                        "No Azure store for target: {}",
                        target.target_id()
                    )))
                }
            }
            BackupTarget::Gcs { .. } => {
                if let Some((_, store)) = self.find_gcs_store(target) {
                    let path = Path::from(object_name);
                    let result = store.get(&path).await.map_err(BackupError::ObjectStore)?;
                    let bytes = result.bytes().await.map_err(BackupError::ObjectStore)?;
                    Ok(bytes.to_vec())
                } else {
                    Err(BackupError::Config(format!(
                        "No GCS store for target: {}",
                        target.target_id()
                    )))
                }
            }
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let full_path = base.join(object_name);
                let data = tokio::fs::read(full_path).await.map_err(BackupError::Io)?;
                Ok(data)
            }
            BackupTarget::Sftp { .. } => {
                crate::sftp_backend::backend_from_target(target)?
                    .download_bytes(target, object_name)
                    .await
            }
            BackupTarget::WebDAV { .. } => {
                crate::webdav_backend::backend_from_target(target)?
                    .download_bytes(target, object_name)
                    .await
            }
        }
    }

    async fn upload_stream(
        &self,
        target: &BackupTarget,
        object_name: &str,
        data: ByteStream,
    ) -> BackupResult<u64> {
        let mut data = data;
        match target {
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let full_path = base.join(object_name);
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(BackupError::Io)?;
                }

                let mut file = tokio::fs::File::create(&full_path)
                    .await
                    .map_err(BackupError::Io)?;
                let mut written: u64 = 0;

                use futures::StreamExt;
                while let Some(chunk) = data.next().await {
                    let chunk = chunk?;
                    file.write_all(&chunk).await.map_err(BackupError::Io)?;
                    written += chunk.len() as u64;
                }
                file.flush().await.map_err(BackupError::Io)?;
                Ok(written)
            }
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                let (_, store) = self.find_s3_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No S3 store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                Self::upload_multipart(store.as_ref(), object_name, data).await
            }
            BackupTarget::Azure { .. } => {
                let (_, store) = self.find_azure_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No Azure store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                Self::upload_multipart(store.as_ref(), object_name, data).await
            }
            BackupTarget::Gcs { .. } => {
                let (_, store) = self.find_gcs_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No GCS store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                Self::upload_multipart(store.as_ref(), object_name, data).await
            }
            BackupTarget::Sftp { .. } => {
                crate::sftp_backend::backend_from_target(target)?
                    .upload_stream(target, object_name, data)
                    .await
            }
            BackupTarget::WebDAV { .. } => {
                crate::webdav_backend::backend_from_target(target)?
                    .upload_stream(target, object_name, data)
                    .await
            }
        }
    }

    async fn download_stream(
        &self,
        target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<ByteStream> {
        match target {
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let full = base.join(object_name);
                let file = tokio::fs::File::open(&full)
                    .await
                    .map_err(BackupError::Io)?;

                use tokio::io::AsyncReadExt;
                let stream = async_stream::stream! {
                    let mut file = file;
                    let mut buf = vec![0u8; STREAM_BUFFER_SIZE];
                    loop {
                        match file.read(&mut buf).await {
                            Ok(0) => break,
                            Ok(n) => yield Ok(bytes::Bytes::copy_from_slice(&buf[..n])),
                            Err(e) => {
                                yield Err(BackupError::Io(e));
                                break;
                            }
                        }
                    }
                };
                Ok(Box::pin(stream))
            }
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                let (_, store) = self.find_s3_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No S3 store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                let result = store
                    .get(&Path::from(object_name))
                    .await
                    .map_err(BackupError::ObjectStore)?;
                Ok(Self::bytes_to_stream(
                    result.bytes().await.map_err(BackupError::ObjectStore)?,
                ))
            }
            BackupTarget::Azure { .. } => {
                let (_, store) = self.find_azure_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No Azure store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                let result = store
                    .get(&Path::from(object_name))
                    .await
                    .map_err(BackupError::ObjectStore)?;
                Ok(Self::bytes_to_stream(
                    result.bytes().await.map_err(BackupError::ObjectStore)?,
                ))
            }
            BackupTarget::Gcs { .. } => {
                let (_, store) = self.find_gcs_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No GCS store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                let result = store
                    .get(&Path::from(object_name))
                    .await
                    .map_err(BackupError::ObjectStore)?;
                Ok(Self::bytes_to_stream(
                    result.bytes().await.map_err(BackupError::ObjectStore)?,
                ))
            }
            BackupTarget::Sftp { .. } => {
                crate::sftp_backend::backend_from_target(target)?
                    .download_stream(target, object_name)
                    .await
            }
            BackupTarget::WebDAV { .. } => {
                crate::webdav_backend::backend_from_target(target)?
                    .download_stream(target, object_name)
                    .await
            }
        }
    }

    async fn list_objects(
        &self,
        target: &BackupTarget,
        prefix: &str,
    ) -> BackupResult<Vec<(String, u64)>> {
        match target {
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let mut out = Vec::new();
                collect_local_objects(&base, &base, prefix, &mut out).await?;
                Ok(out)
            }
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                let (_, store) = self.find_s3_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No S3 store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                let prefix_path = Path::from(prefix);
                let mut out = Vec::new();
                let mut stream = store.list(Some(&prefix_path));
                while let Some(item) = stream.next().await {
                    let meta = item.map_err(BackupError::ObjectStore)?;
                    out.push((meta.location.to_string(), meta.size as u64));
                }
                Ok(out)
            }
            BackupTarget::Sftp { .. } => {
                crate::sftp_backend::backend_from_target(target)?
                    .list_objects(target, prefix)
                    .await
            }
            BackupTarget::WebDAV { .. } => {
                crate::webdav_backend::backend_from_target(target)?
                    .list_objects(target, prefix)
                    .await
            }
            _ => Err(BackupError::Config(format!(
                "List is not supported for target: {}",
                target.target_id()
            ))),
        }
    }

    async fn delete_object(&self, target: &BackupTarget, object_name: &str) -> BackupResult<()> {
        match target {
            BackupTarget::Local { path, .. } => {
                let base = self
                    .find_local(target)
                    .map(|(_, p)| p)
                    .unwrap_or_else(|| path.clone());
                let full = base.join(object_name);
                if full.exists() {
                    tokio::fs::remove_file(&full)
                        .await
                        .map_err(BackupError::Io)?;
                }
                Ok(())
            }
            BackupTarget::S3 { .. }
            | BackupTarget::MinIO { .. }
            | BackupTarget::B2 { .. }
            | BackupTarget::Wasabi { .. } => {
                let (_, store) = self.find_s3_store(target).ok_or_else(|| {
                    BackupError::Config(format!(
                        "No S3 store configured for target: {}",
                        target.target_id()
                    ))
                })?;
                let path = Path::from(object_name);
                store
                    .delete(&path)
                    .await
                    .map_err(BackupError::ObjectStore)?;
                Ok(())
            }
            BackupTarget::Sftp { .. } => {
                crate::sftp_backend::backend_from_target(target)?
                    .delete_object(target, object_name)
                    .await
            }
            BackupTarget::WebDAV { .. } => {
                crate::webdav_backend::backend_from_target(target)?
                    .delete_object(target, object_name)
                    .await
            }
            _ => Err(BackupError::Config(format!(
                "Delete is not supported for target: {}",
                target.target_id()
            ))),
        }
    }

    async fn load_manifest(
        &self,
        target: &BackupTarget,
        backup_id: &uuid::Uuid,
    ) -> BackupResult<Option<crate::manifest::BackupManifest>> {
        let object_name = format!("manifests/{backup_id}.json");
        let data = match self.download_bytes(target, &object_name).await {
            Ok(d) => d,
            // A backup without a manifest simply has none to load. Remote
            // backends report a missing object as a transport error carrying
            // the status, so treat those as absent too rather than failing a
            // restore that can still fall back to listing.
            Err(BackupError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(e) if is_missing_object(&e) => return Ok(None),
            Err(e) => return Err(e),
        };
        let manifest = serde_json::from_slice(&data).map_err(BackupError::Serialization)?;
        Ok(Some(manifest))
    }

    async fn save_manifest(
        &self,
        target: &BackupTarget,
        manifest: &crate::manifest::BackupManifest,
    ) -> BackupResult<()> {
        let manifest_json =
            serde_json::to_vec_pretty(manifest).map_err(BackupError::Serialization)?;
        let object_name = format!("manifests/{}.json", manifest.backup_id);
        self.upload_bytes(target, &object_name, &manifest_json)
            .await
    }
}

/// Whether a download error means "object does not exist".
///
/// Local storage surfaces a NotFound io error; the remote backends wrap the
/// HTTP status in a Config error message.
fn is_missing_object(e: &BackupError) -> bool {
    match e {
        BackupError::Config(msg) => msg.contains("404") || msg.to_lowercase().contains("not found"),
        _ => false,
    }
}

/// Recursively collect files under `root` whose relative path starts with
/// `prefix`, returning (relative path, size) pairs.
async fn collect_local_objects(
    root: &std::path::Path,
    dir: &std::path::Path,
    prefix: &str,
    out: &mut Vec<(String, u64)>,
) -> BackupResult<()> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(BackupError::Io(e)),
    };

    while let Some(entry) = entries.next_entry().await.map_err(BackupError::Io)? {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(collect_local_objects(root, &path, prefix, out)).await?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if rel.starts_with(prefix) {
                let size = entry.metadata().await.map_err(BackupError::Io)?.len();
                out.push((rel, size));
            }
        }
    }
    Ok(())
}
