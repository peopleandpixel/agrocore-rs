//! Deletion of a backup and the objects that belong to it (tasks.md J7).
//!
//! `delete_backup` answered `200 {"success": true}` after checking only that the
//! job existed. Nothing was removed from storage, so an admin who deleted a
//! backup to free space kept paying for it — and the retention sweep would later
//! find the object still there.
//!
//! These tests run against a real local storage backend, because the thing that
//! has to be proven is that files actually leave the disk.

use std::sync::Arc;

use agrocore_backup::config::{BackupTarget, TargetEncryption};
use agrocore_backup::service::{DeleteOutcome, delete_backup_objects};
use agrocore_backup::storage::{StorageBackend, StorageBackendTrait};
use chrono::{Duration, Utc};
use uuid::Uuid;

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "agrocore-delete-{name}-{}-{:?}",
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

fn local_target(dir: &std::path::Path) -> BackupTarget {
    BackupTarget::Local {
        path: dir.to_path_buf(),
        encryption: TargetEncryption::None,
        permissions: None,
    }
}

/// Write a dump object, as pg_dump does.
fn write_dump(dir: &std::path::Path, id: Uuid, when: chrono::DateTime<Utc>, size: usize) -> String {
    let name = format!("{id}_dump_{}.dump", when.format("%Y%m%d_%H%M%S"));
    std::fs::write(dir.join(&name), vec![0u8; size]).expect("write dump");
    name
}

async fn backend(dir: &std::path::Path) -> Arc<dyn StorageBackendTrait> {
    Arc::new(
        StorageBackend::new(vec![local_target(dir)])
            .await
            .expect("backend"),
    )
}

async fn delete(
    storage: &Arc<dyn StorageBackendTrait>,
    dir: &std::path::Path,
    id: Uuid,
) -> DeleteOutcome {
    delete_backup_objects(storage.as_ref(), &[local_target(dir)], id)
        .await
        .expect("delete")
}

/// The core assertion: the file is gone from disk afterwards.
#[tokio::test]
async fn deleting_a_backup_removes_the_file() {
    let dir = TempDir::new("basic");
    let storage = backend(&dir.0).await;

    let id = Uuid::new_v4();
    let name = write_dump(&dir.0, id, Utc::now(), 64);
    assert!(dir.0.join(&name).exists(), "fixture must exist first");

    let outcome = delete(&storage, &dir.0, id).await;

    assert!(
        !dir.0.join(&name).exists(),
        "the dump must not still be on disk after deletion"
    );
    assert_eq!(outcome.bytes_freed, 64);
    assert!(outcome.failed_targets.is_empty());
    assert!(outcome.deleted_objects.contains(&name));
}

/// Deleting an unknown id must fail rather than report success. Returning
/// success here would make a typo look like a working delete.
#[tokio::test]
async fn deleting_an_unknown_backup_fails() {
    let dir = TempDir::new("unknown");
    let storage = backend(&dir.0).await;

    // An unrelated dump exists, so the failure cannot be blamed on empty storage.
    write_dump(&dir.0, Uuid::new_v4(), Utc::now(), 8);

    let result =
        delete_backup_objects(storage.as_ref(), &[local_target(&dir.0)], Uuid::new_v4()).await;

    assert!(
        result.is_err(),
        "an unknown backup id must not report success"
    );
}

/// One backup replicated to two targets is two copies. Deleting only one leaves
/// a restorable backup behind, which is exactly what an admin asked to delete.
#[tokio::test]
async fn deletion_covers_every_target() {
    let primary = TempDir::new("primary");
    let replica = TempDir::new("replica");

    let storage = Arc::new(
        StorageBackend::new(vec![local_target(&primary.0), local_target(&replica.0)])
            .await
            .expect("backend"),
    );

    let id = Uuid::new_v4();
    let name = write_dump(&primary.0, id, Utc::now(), 32);
    write_dump(&replica.0, id, Utc::now(), 32);

    let outcome = delete_backup_objects(
        storage.as_ref(),
        &[local_target(&primary.0), local_target(&replica.0)],
        id,
    )
    .await
    .expect("delete");

    assert!(!primary.0.join(&name).exists(), "primary copy must be gone");
    assert!(!replica.0.join(&name).exists(), "replica copy must be gone");
    assert!(
        outcome.failed_targets.is_empty(),
        "neither target should have failed"
    );
    // Two copies of 32 bytes.
    assert_eq!(outcome.bytes_freed, 64);
}

/// A backup consists of more than its dump: a full backup also writes a
/// checksum and a manifest. Deleting only the dump would leave those behind and
/// the listing would keep showing the backup.
///
/// The manifest is written through `save_manifest` rather than by hand, so the
/// test uses the same object layout the service writes. Writing the file by hand
/// to a guessed path passed a list of objects but never loaded the manifest at
/// all — the deletion then ran on the name-matching fallback and the manifest
/// stayed behind.
#[tokio::test]
async fn deletion_removes_the_checksum_and_manifest_too() {
    let dir = TempDir::new("multi");
    let storage = backend(&dir.0).await;
    let target = local_target(&dir.0);

    let id = Uuid::new_v4();
    let dump = write_dump(&dir.0, id, Utc::now(), 16);
    let checksum = format!("{dump}.sha256");
    std::fs::write(dir.0.join(&checksum), b"deadbeef").expect("write checksum");

    let manifest = agrocore_backup::manifest::BackupManifest {
        backup_id: id,
        backup_type: agrocore_backup::service::BackupType::Full,
        status: agrocore_backup::service::BackupStatus::Completed,
        started_at: Utc::now(),
        completed_at: Utc::now(),
        targets: vec![agrocore_backup::manifest::TargetManifest {
            target_id: "local".into(),
            target_type: "Local".into(),
            objects: vec![
                agrocore_backup::manifest::ObjectManifest {
                    name: dump.clone(),
                    size_bytes: 16,
                    checksum_sha256: "0".repeat(64),
                    modified_at: Utc::now(),
                },
                agrocore_backup::manifest::ObjectManifest {
                    name: checksum.clone(),
                    size_bytes: 8,
                    checksum_sha256: "0".repeat(64),
                    modified_at: Utc::now(),
                },
            ],
            size_bytes: 24,
            encryption: None,
        }],
        total_size_bytes: 24,
        schema_version: None,
        git_commit: None,
        git_branch: None,
        app_version: "test".into(),
        checksums: vec![],
        metadata: Default::default(),
    };
    storage
        .save_manifest(&target, &manifest)
        .await
        .expect("save manifest");

    // The manifest must be discoverable, otherwise this test would silently
    // pass on the fallback path.
    assert!(
        storage
            .load_manifest(&target, &id)
            .await
            .expect("load manifest")
            .is_some(),
        "the manifest must be readable before deletion"
    );

    let outcome = delete(&storage, &dir.0, id).await;

    assert!(!dir.0.join(&dump).exists(), "dump must be gone");
    assert!(!dir.0.join(&checksum).exists(), "checksum must be gone");
    assert!(
        storage
            .load_manifest(&target, &id)
            .await
            .expect("reload manifest")
            .is_none(),
        "manifest must be gone, otherwise the listing still shows the backup"
    );
    assert!(
        outcome.deleted_objects.len() >= 3,
        "expected dump, checksum and manifest to be reported, got {:?}",
        outcome.deleted_objects
    );
}

/// Deleting one backup must not touch another one that shares the directory.
#[tokio::test]
async fn deletion_leaves_other_backups_alone() {
    let dir = TempDir::new("other");
    let storage = backend(&dir.0).await;

    let keep_id = Uuid::new_v4();
    let drop_id = Uuid::new_v4();
    let keep = write_dump(&dir.0, keep_id, Utc::now() - Duration::days(1), 8);
    let drop = write_dump(&dir.0, drop_id, Utc::now(), 8);

    delete(&storage, &dir.0, drop_id).await;

    assert!(!dir.0.join(&drop).exists());
    assert!(
        dir.0.join(&keep).exists(),
        "an unrelated backup must survive"
    );
}
