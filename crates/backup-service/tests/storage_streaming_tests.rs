//! Storage backend tests: streaming upload/download, listing and deletion.
//!
//! These exercise the Local backend against the real filesystem, which is the
//! only backend that does not need credentials from the environment.

use agrocore_backup::config::{BackupTarget, TargetEncryption};
use agrocore_backup::storage::{ByteStream, StorageBackend, StorageBackendTrait};
use bytes::Bytes;

fn local_target(dir: &std::path::Path) -> BackupTarget {
    BackupTarget::Local {
        path: dir.to_path_buf(),
        encryption: TargetEncryption::None,
        permissions: None,
    }
}

fn chunked(data: &[u8], chunk_size: usize) -> ByteStream {
    let chunks: Vec<Result<Bytes, agrocore_backup::error::BackupError>> = data
        .chunks(chunk_size)
        .map(|c| Ok(Bytes::copy_from_slice(c)))
        .collect();
    Box::pin(futures::stream::iter(chunks))
}

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "agrocore-storage-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("create temp dir");
        Self(p)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn upload_stream_writes_all_chunks() {
    let dir = TempDir::new("upload");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");

    let payload: Vec<u8> = (0..500_000u32).map(|i| (i % 251) as u8).collect();
    let target = local_target(&dir.0);

    let written = backend
        .upload_stream(&target, "data/payload.bin", chunked(&payload, 64 * 1024))
        .await
        .expect("upload");

    assert_eq!(
        written,
        payload.len() as u64,
        "reported size must match input"
    );

    let stored = std::fs::read(dir.0.join("data/payload.bin")).expect("read back");
    assert_eq!(stored, payload, "streamed content must match byte for byte");
}

#[tokio::test]
async fn upload_stream_handles_empty_payload() {
    let dir = TempDir::new("empty");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    let written = backend
        .upload_stream(&target, "empty.bin", chunked(&[], 1024))
        .await
        .expect("upload");

    assert_eq!(written, 0);
    assert!(dir.0.join("empty.bin").exists());
}

#[tokio::test]
async fn download_stream_roundtrips_uploaded_data() {
    let dir = TempDir::new("roundtrip");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    let payload: Vec<u8> = (0..300_000u32).map(|i| (i % 256) as u8).collect();
    backend
        .upload_stream(&target, "rt.bin", chunked(&payload, 32 * 1024))
        .await
        .expect("upload");

    let mut stream = backend
        .download_stream(&target, "rt.bin")
        .await
        .expect("stream");
    let mut received = Vec::new();
    while let Some(chunk) = futures::StreamExt::next(&mut stream).await {
        received.extend_from_slice(&chunk.expect("chunk"));
    }

    assert_eq!(received, payload, "download must reconstruct the upload");
}

#[tokio::test]
async fn download_stream_reports_missing_object() {
    let dir = TempDir::new("missing");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    // ByteStream is not Debug, so match the Result manually.
    match backend.download_stream(&target, "does-not-exist.bin").await {
        Ok(_) => panic!("missing object must not yield a stream"),
        Err(agrocore_backup::error::BackupError::Io(_)) => {}
        Err(other) => panic!("expected IO error, got {other:?}"),
    }
}

#[tokio::test]
async fn list_objects_reports_names_and_sizes() {
    let dir = TempDir::new("list");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    std::fs::create_dir_all(dir.0.join("nested")).expect("mkdir");
    std::fs::write(dir.0.join("a.dump"), vec![0u8; 10]).expect("write a");
    std::fs::write(dir.0.join("nested/b.dump"), vec![0u8; 25]).expect("write b");
    std::fs::write(dir.0.join("ignored.txt"), vec![0u8; 5]).expect("write txt");

    let all = backend.list_objects(&target, "").await.expect("list all");
    assert_eq!(all.len(), 3, "all objects must be listed: {all:?}");

    let dumps = backend.list_objects(&target, "").await.expect("list");
    let sizes: Vec<u64> = dumps.iter().map(|(_, s)| *s).collect();
    assert!(
        sizes.contains(&10) && sizes.contains(&25),
        "sizes: {sizes:?}"
    );

    let filtered = backend
        .list_objects(&target, "nested/")
        .await
        .expect("list prefix");
    assert_eq!(
        filtered.len(),
        1,
        "prefix filter must exclude others: {filtered:?}"
    );
    assert_eq!(filtered[0].0, "nested/b.dump");
}

#[tokio::test]
async fn list_objects_on_empty_directory_is_empty() {
    let dir = TempDir::new("emptydir");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    let objects = backend.list_objects(&target, "").await.expect("list");
    assert!(objects.is_empty(), "empty dir must yield no objects");
}

#[tokio::test]
async fn delete_object_removes_file() {
    let dir = TempDir::new("delete");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    std::fs::write(dir.0.join("doomed.dump"), b"data").expect("write");
    assert!(dir.0.join("doomed.dump").exists());

    backend
        .delete_object(&target, "doomed.dump")
        .await
        .expect("delete");
    assert!(!dir.0.join("doomed.dump").exists(), "file must be gone");
}

#[tokio::test]
async fn delete_object_is_idempotent() {
    let dir = TempDir::new("delete-missing");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let target = local_target(&dir.0);

    // Retention may race with manual cleanup; deleting an absent object is fine.
    backend
        .delete_object(&target, "never-existed.dump")
        .await
        .expect("deleting an absent object must succeed");
}
