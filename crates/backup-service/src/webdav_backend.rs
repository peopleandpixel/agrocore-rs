//! WebDAV storage backend.
//!
//! Talks to any WebDAV server (Nextcloud, ownCloud, Apache mod_dav) over
//! HTTP. Uploads stream via chunked transfer encoding so large dumps never
//! have to be buffered, and the object list comes from a recursive `PROPFIND`.

use crate::config::{BackupTarget, EncryptionConfig, TargetEncryption};
use crate::encryption::{decrypt_payload, encrypt_payload};
use crate::error::{BackupError, BackupResult};
use crate::storage::{ByteStream, StorageBackendTrait};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::StreamExt;
use std::sync::Arc;

/// Default chunk size for streamed uploads.
const CHUNK_SIZE: usize = 1_048_576;

/// WebDAV backend configuration.
pub struct WebdavBackend {
    base_url: String,
    username: String,
    password: String,
    prefix: String,
    encryption: TargetEncryption,
    encryption_config: EncryptionConfig,
    chunk_size: usize,
}

impl WebdavBackend {
    /// Create a backend from a validated target.
    pub fn new(
        url: String,
        username: String,
        password: String,
        prefix: String,
        encryption: TargetEncryption,
        chunk_size: Option<usize>,
    ) -> Self {
        let base = url.trim_end_matches('/').to_string();
        Self {
            base_url: base,
            username,
            password,
            prefix: prefix.trim_matches('/').to_string(),
            encryption,
            encryption_config: EncryptionConfig::default(),
            chunk_size: chunk_size.unwrap_or(CHUNK_SIZE).max(64 * 1024),
        }
    }

    /// Provide the encryption configuration used for payloads.
    pub fn with_encryption_config(mut self, config: EncryptionConfig) -> Self {
        self.encryption_config = config;
        self
    }

    /// HTTP client with basic auth and no redirect following, so credentials
    /// are never replayed to another host.
    fn client(&self) -> BackupResult<reqwest::Client> {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| BackupError::Config(format!("WebDAV client build failed: {e}")))
    }

    /// Full URL for an object name.
    fn object_url(&self, object_name: &str) -> String {
        let mut url = self.base_url.clone();
        if !self.prefix.is_empty() {
            url.push('/');
            url.push_str(&self.prefix);
        }
        url.push('/');
        // Percent-encode each path segment, keeping separators intact.
        for (i, segment) in object_name.split('/').enumerate() {
            if i > 0 {
                url.push('/');
            }
            url.push_str(&encode_segment(segment));
        }
        url
    }

    /// URL of the collection that holds objects.
    fn base_collection_url(&self) -> String {
        if self.prefix.is_empty() {
            self.base_url.clone()
        } else {
            format!("{}/{}/", self.base_url, self.prefix)
        }
    }

    /// Create a remote collection, ignoring "already exists".
    async fn ensure_collection(&self, url: &str) -> BackupResult<()> {
        let client = self.client()?;
        let resp = client
            .head(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV HEAD failed: {e}")))?;

        if resp.status().is_success() {
            return Ok(());
        }
        if resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            // Some servers forbid HEAD; assume the collection exists.
            return Ok(());
        }

        let resp = client
            .request(webdav_method("MKCOL"), url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV MKCOL failed: {e}")))?;

        if resp.status().is_success() || resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            return Ok(());
        }
        Err(BackupError::Config(format!(
            "WebDAV MKCOL {url} failed with status {}",
            resp.status()
        )))
    }
}

/// Percent-encode a single path segment.
/// WebDAV extension methods are not in reqwest's Method enum.
fn webdav_method(name: &str) -> reqwest::Method {
    reqwest::Method::from_bytes(name.as_bytes()).expect("static method name is a valid HTTP token")
}

fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Extract the href and content length of every `<D:response>` element.
fn parse_propfind(body: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    // Response blocks are delimited by D:response tags. A simple scan avoids
    // pulling in a full XML parser for this fixed server-generated shape.
    let mut rest = body;
    while let Some(start) = rest.find("<D:response") {
        let after = &rest[start..];
        let Some(end) = after.find("</D:response>") else {
            break;
        };
        let block = &after[..end];

        let href = block
            .split_once("<D:href>")
            .and_then(|(_, v)| v.split_once("</D:href>"))
            .map(|(v, _)| v.trim().to_string());
        let size = block
            .split_once("<D:getcontentlength>")
            .and_then(|(_, v)| v.split_once("</D:getcontentlength>"))
            .and_then(|(v, _)| v.trim().parse::<u64>().ok())
            .unwrap_or(0);

        // A collection has no content length in this listing, and its href
        // ends with a slash.
        if let Some(href) = href {
            let is_collection = href.ends_with('/');
            if !is_collection && !href.ends_with("/:") {
                out.push((href, size));
            }
        }
        rest = &after[end..];
    }
    out
}

/// Turn an href into a path relative to the collection URL.
fn href_to_name(href: &str, base_collection: &str) -> Option<String> {
    let path = percent_decode(href);

    // A WebDAV href is a server-absolute path, while the configured
    // collection URL carries scheme and host. Reduce the collection URL to its
    // path so the two can be compared.
    let base = base_collection
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_collection);
    // Drop the host, keeping only the path.
    let base = match base.split_once('/') {
        Some((_host, path)) => path,
        None => "",
    };
    let base = percent_decode(base);
    let base = base.trim_end_matches('/').trim_start_matches('/');

    let path = path.trim_start_matches('/');
    let stripped = path.strip_prefix(base)?.trim_start_matches('/');
    if stripped.is_empty() {
        return None;
    }
    Some(stripped.to_string())
}

/// Percent-decode a URL path.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[async_trait]
impl StorageBackendTrait for WebdavBackend {
    async fn upload_bytes(
        &self,
        _target: &BackupTarget,
        object_name: &str,
        data: &[u8],
    ) -> BackupResult<()> {
        let url = self.object_url(object_name);
        self.ensure_collection(&self.base_collection_url()).await?;

        let payload = encrypt_payload(data, &self.encryption, &self.encryption_config).await?;

        let client = self.client()?;
        let resp = client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(payload)
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV PUT failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(BackupError::Config(format!(
                "WebDAV PUT {url} failed with status {}",
                resp.status()
            )));
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
        self.upload_bytes(target, object_name, &data).await?;
        Ok(data.len() as u64)
    }

    async fn download_bytes(
        &self,
        _target: &BackupTarget,
        object_name: &str,
    ) -> BackupResult<Vec<u8>> {
        let url = self.object_url(object_name);
        let client = self.client()?;
        let resp = client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV GET failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(BackupError::Config(format!(
                "WebDAV GET {url} failed with status {}",
                resp.status()
            )));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV body read failed: {e}")))?;
        decrypt_payload(&bytes, &self.encryption, &self.encryption_config).await
    }

    async fn upload_stream(
        &self,
        _target: &BackupTarget,
        object_name: &str,
        data: ByteStream,
    ) -> BackupResult<u64> {
        let url = self.object_url(object_name);
        self.ensure_collection(&self.base_collection_url()).await?;

        let client = self.client()?;
        // Chunked body: reqwest streams the async stream to the socket, so the
        // dump is never fully buffered in memory.
        let chunk_size = self.chunk_size;
        // Encrypt per chunk: the whole dump never has to be in memory.
        // The encryption state travels with the unfold state so the closure
        // can borrow it instead of moving out of the capture.
        let enc_state = (self.encryption.clone(), self.encryption_config.clone());
        let body = futures::stream::unfold(
            (data, BytesMut::with_capacity(chunk_size), enc_state),
            move |(mut stream, mut buf, enc)| async move {
                if buf.is_empty() {
                    loop {
                        match stream.next().await {
                            Some(Ok(chunk)) => {
                                buf.extend_from_slice(&chunk);
                                if buf.len() >= chunk_size {
                                    break;
                                }
                            }
                            Some(Err(e)) => {
                                return Some((Err(e), (stream, buf, enc)));
                            }
                            None => break,
                        }
                    }
                }
                if buf.is_empty() {
                    return None;
                }
                let raw = buf.split().freeze();
                let out = match encrypt_payload(&raw, &enc.0, &enc.1).await {
                    Ok(v) => v,
                    Err(e) => return Some((Err(e), (stream, buf, enc))),
                };
                Some((Ok(out), (stream, buf, enc)))
            },
        );

        let resp = client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(reqwest::Body::wrap_stream(body))
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV PUT stream failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(BackupError::Config(format!(
                "WebDAV PUT {url} failed with status {}",
                resp.status()
            )));
        }
        Ok(0)
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
        let url = self.base_collection_url();
        let client = self.client()?;

        // Ask for a recursive listing with object sizes.
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:"><D:prop><D:getcontentlength/></D:prop></D:propfind>"#;

        let resp = client
            .request(webdav_method("PROPFIND"), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "infinity")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV PROPFIND failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(BackupError::Config(format!(
                "WebDAV PROPFIND {url} failed with status {}",
                resp.status()
            )));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV PROPFIND body failed: {e}")))?;

        let mut out = Vec::new();
        for (href, size) in parse_propfind(&text) {
            let Some(name) = href_to_name(&href, &url) else {
                continue;
            };
            if prefix.is_empty() || name.starts_with(prefix) {
                out.push((name, size));
            }
        }
        Ok(out)
    }

    async fn delete_object(&self, _target: &BackupTarget, object_name: &str) -> BackupResult<()> {
        let url = self.object_url(object_name);
        let client = self.client()?;
        let resp = client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| BackupError::Config(format!("WebDAV DELETE failed: {e}")))?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(BackupError::Config(format!(
                "WebDAV DELETE {url} failed with status {}",
                resp.status()
            )));
        }
        Ok(())
    }

    async fn load_manifest(
        &self,
        target: &BackupTarget,
        backup_id: &uuid::Uuid,
    ) -> BackupResult<Option<crate::manifest::BackupManifest>> {
        let name = format!("manifests/{backup_id}.json");
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

/// Build a WebDAV backend from a target configuration.
pub fn backend_from_target(target: &BackupTarget) -> BackupResult<WebdavBackend> {
    match target {
        BackupTarget::WebDAV {
            url,
            username,
            password,
            prefix,
            encryption,
            chunk_size,
        } => Ok(WebdavBackend::new(
            url.clone(),
            username.clone(),
            password.clone(),
            prefix.clone(),
            encryption.clone(),
            *chunk_size,
        )),
        other => Err(BackupError::Config(format!(
            "not a WebDAV target: {other:?}"
        ))),
    }
}

// Keep an Arc-based helper available for callers that share one backend
// across tasks.
#[allow(dead_code)]
fn _shared(b: WebdavBackend) -> Arc<WebdavBackend> {
    Arc::new(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> WebdavBackend {
        WebdavBackend::new(
            "https://cloud.example.com/remote.php/dav/files/backup".to_string(),
            "user".to_string(),
            "pass".to_string(),
            "agrocore".to_string(),
            TargetEncryption::None,
            None,
        )
    }

    #[test]
    fn object_url_joins_prefix_and_name() {
        assert_eq!(
            backend().object_url("dump.dump"),
            "https://cloud.example.com/remote.php/dav/files/backup/agrocore/dump.dump"
        );
    }

    #[test]
    fn trailing_slash_on_base_is_normalised() {
        let b = WebdavBackend::new(
            "https://example.com/dav/".to_string(),
            "u".to_string(),
            "p".to_string(),
            String::new(),
            TargetEncryption::None,
            None,
        );
        assert_eq!(b.object_url("a.dump"), "https://example.com/dav/a.dump");
    }

    #[test]
    fn empty_prefix_is_omitted() {
        let b = WebdavBackend::new(
            "https://example.com/dav".to_string(),
            "u".to_string(),
            "p".to_string(),
            String::new(),
            TargetEncryption::None,
            None,
        );
        assert_eq!(b.object_url("a.dump"), "https://example.com/dav/a.dump");
        assert_eq!(b.base_collection_url(), "https://example.com/dav");
    }

    #[test]
    fn path_segments_are_percent_encoded() {
        let b = backend();
        let url = b.object_url("my backup+1.dump");
        assert!(url.ends_with("my%20backup%2B1.dump"), "got {url}");
    }

    #[test]
    fn nested_paths_keep_separators() {
        let url = backend().object_url("manifests/abc.json");
        assert!(url.ends_with("/manifests/abc.json"), "got {url}");
    }

    #[test]
    fn propfind_parses_hrefs_and_sizes() {
        let body = r#"<?xml version="1.0"?>
<D:multistatus xmlns:D="DAV:">
  <D:response><D:href>/dav/agrocore/</D:href></D:response>
  <D:response><D:href>/dav/agrocore/a.dump</D:href><D:getcontentlength>1234</D:getcontentlength></D:response>
  <D:response><D:href>/dav/agrocore/sub/b.dump</D:href><D:getcontentlength>99</D:getcontentlength></D:response>
</D:multistatus>"#;
        let parsed = parse_propfind(body);
        assert_eq!(parsed.len(), 2, "collections must be skipped: {parsed:?}");
        assert_eq!(parsed[0].1, 1234);
        assert_eq!(parsed[1].0, "/dav/agrocore/sub/b.dump");
    }

    #[test]
    fn href_maps_to_relative_name() {
        let name = href_to_name("/dav/agrocore/a.dump", "https://h/dav/agrocore/").expect("name");
        assert_eq!(name, "a.dump");

        let nested = href_to_name("/dav/agrocore/manifests/x.json", "https://h/dav/agrocore/")
            .expect("name");
        assert_eq!(nested, "manifests/x.json");
    }

    #[test]
    fn href_outside_collection_yields_none() {
        assert!(href_to_name("/other/a.dump", "https://h/dav/agrocore/").is_none());
    }

    #[test]
    fn percent_decoding_roundtrips() {
        assert_eq!(percent_decode("my%20backup%2B1.dump"), "my backup+1.dump");
    }

    #[test]
    fn non_webdav_target_is_rejected() {
        let local = BackupTarget::Local {
            path: std::path::PathBuf::from("/tmp"),
            encryption: TargetEncryption::None,
            permissions: None,
        };
        assert!(backend_from_target(&local).is_err());
    }

    #[test]
    fn chunk_size_has_a_floor() {
        let b = WebdavBackend::new(
            "https://example.com/dav".to_string(),
            "u".to_string(),
            "p".to_string(),
            String::new(),
            TargetEncryption::None,
            Some(10),
        );
        assert_eq!(b.chunk_size, 64 * 1024, "tiny chunks are rejected");
    }
}
