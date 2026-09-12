use crate::config::BackupTarget;
use crate::error::{BackupError, BackupResult};
use agrocore_logging::warn;
use async_trait::async_trait;
use object_store::{ObjectStore, PutPayload, path::Path};
use std::sync::Arc;

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
    async fn save_manifest(
        &self,
        target: &BackupTarget,
        manifest: &crate::manifest::BackupManifest,
    ) -> BackupResult<()>;
}

pub struct StorageBackend {
    s3_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    azure_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    gcs_clients: Vec<(BackupTarget, Arc<dyn ObjectStore>)>,
    local_paths: Vec<(BackupTarget, std::path::PathBuf)>,
}

impl StorageBackend {
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
                        .map_err(|e| BackupError::Io(e))?;
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
                BackupTarget::GCS {
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
                BackupTarget::SFTP { .. } => {
                    warn!("SFTP backend not yet implemented");
                }
                BackupTarget::WebDAV { .. } => {
                    warn!("WebDAV backend not yet implemented");
                }
                _ => {
                    warn!("Storage backend for target not yet implemented");
                }
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

        let client = builder.build().map_err(|e| BackupError::ObjectStore(e))?;
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

        if let Some(creds) = credentials {
            if let Some(key) = creds.account_key {
                builder = builder.with_access_key(key);
            }
        }

        let client = builder.build().map_err(|e| BackupError::ObjectStore(e))?;
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

        let client = builder.build().map_err(|e| BackupError::ObjectStore(e))?;
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
                        .map_err(|e| BackupError::ObjectStore(e))?;
                }
            }
            BackupTarget::Azure { .. } => {
                if let Some((_, store)) = self.find_azure_store(target) {
                    let path = Path::from(object_name);
                    store
                        .put(&path, PutPayload::from(data.to_vec()))
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                }
            }
            BackupTarget::GCS { .. } => {
                if let Some((_, store)) = self.find_gcs_store(target) {
                    let path = Path::from(object_name);
                    store
                        .put(&path, PutPayload::from(data.to_vec()))
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                }
            }
            BackupTarget::Local { path, .. } => {
                let full_path = path.join(object_name);
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| BackupError::Io(e))?;
                }
                tokio::fs::write(full_path, data)
                    .await
                    .map_err(|e| BackupError::Io(e))?;
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
        let data = tokio::fs::read(file_path)
            .await
            .map_err(|e| BackupError::Io(e))?;
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
                    let result = store
                        .get(&path)
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                    let bytes = result
                        .bytes()
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
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
                    let result = store
                        .get(&path)
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                    let bytes = result
                        .bytes()
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                    Ok(bytes.to_vec())
                } else {
                    Err(BackupError::Config(format!(
                        "No Azure store for target: {}",
                        target.target_id()
                    )))
                }
            }
            BackupTarget::GCS { .. } => {
                if let Some((_, store)) = self.find_gcs_store(target) {
                    let path = Path::from(object_name);
                    let result = store
                        .get(&path)
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                    let bytes = result
                        .bytes()
                        .await
                        .map_err(|e| BackupError::ObjectStore(e))?;
                    Ok(bytes.to_vec())
                } else {
                    Err(BackupError::Config(format!(
                        "No GCS store for target: {}",
                        target.target_id()
                    )))
                }
            }
            BackupTarget::Local { path, .. } => {
                let full_path = path.join(object_name);
                let data = tokio::fs::read(full_path)
                    .await
                    .map_err(|e| BackupError::Io(e))?;
                Ok(data)
            }
            _ => Err(BackupError::Config(format!(
                "No storage backend configured for target: {}",
                target.target_id()
            ))),
        }
    }

    async fn save_manifest(
        &self,
        target: &BackupTarget,
        manifest: &crate::manifest::BackupManifest,
    ) -> BackupResult<()> {
        let manifest_json =
            serde_json::to_vec_pretty(manifest).map_err(|e| BackupError::Serialization(e))?;
        let object_name = format!("manifests/{}.json", manifest.backup_id);
        self.upload_bytes(target, &object_name, &manifest_json)
            .await
    }
}
