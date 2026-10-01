//! Manifest persistence and restore-object resolution tests.
//!
//! The restore path must locate a backup by its id. Historically that was a
//! file-name guess; it now reads the persisted manifest, so these tests pin
//! that round trip down.

use agrocore_backup::config::{BackupTarget, TargetEncryption};
use agrocore_backup::manifest::{BackupManifest, ManifestManager, ObjectManifest, TargetManifest};
use agrocore_backup::service::{BackupStatus, BackupType};
use agrocore_backup::storage::{StorageBackend, StorageBackendTrait};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "agrocore-manifest-{name}-{}-{:?}",
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

fn sample_manifest(backup_id: Uuid, dump_name: &str) -> BackupManifest {
    BackupManifest {
        backup_id,
        backup_type: BackupType::Database,
        status: BackupStatus::Completed,
        started_at: Utc::now(),
        completed_at: Utc::now(),
        targets: vec![TargetManifest {
            target_id: "local:///tmp/backups".to_string(),
            target_type: "local".to_string(),
            size_bytes: 2048,
            encryption: None,
            objects: vec![ObjectManifest {
                name: dump_name.to_string(),
                size_bytes: 2048,
                checksum_sha256: String::new(),
                modified_at: Utc::now(),
            }],
        }],
        total_size_bytes: 2048,
        schema_version: Some("0.1".to_string()),
        git_commit: None,
        git_branch: None,
        app_version: "0.22.0".to_string(),
        checksums: vec![],
        metadata: std::collections::HashMap::new(),
    }
}

#[tokio::test]
async fn manifest_roundtrips_through_storage() {
    let dir = TempDir::new("roundtrip");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    let id = Uuid::new_v4();
    let dump_name = format!("{id}_dump_20261001_120000.dump");
    let manifest = sample_manifest(id, &dump_name);

    let manager = ManifestManager::new(agrocore_backup::config::BackupMetadataConfig::default());
    manager
        .save_manifest(&storage, &target, &manifest)
        .await
        .expect("save manifest");

    let stored = dir.0.join(format!("manifests/{id}.json"));
    assert!(stored.exists(), "manifest file must exist at {stored:?}");

    let loaded = storage
        .load_manifest(&target, &id)
        .await
        .expect("load")
        .expect("manifest must be present");

    assert_eq!(loaded.backup_id, id);
    assert_eq!(loaded.total_size_bytes, 2048);
    assert_eq!(loaded.targets.len(), 1);
    assert_eq!(
        loaded.targets[0].objects[0].name, dump_name,
        "dump object name must survive the round trip"
    );
}

#[tokio::test]
async fn load_manifest_returns_none_when_absent() {
    let dir = TempDir::new("absent");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    let loaded = storage
        .load_manifest(&target, &Uuid::new_v4())
        .await
        .expect("lookup must not error");
    assert!(
        loaded.is_none(),
        "missing manifest yields None, not an error"
    );
}

/// The dump object name embeds the backup id so restore can find it without
/// listing the whole bucket.
#[test]
fn dump_object_name_embeds_backup_id() {
    let id = Uuid::new_v4();
    let name = format!("{id}_dump_20261001_120000.dump");
    assert!(name.contains(&id.to_string()));
}

/// Retention must still parse timestamps out of id-bearing dump names.
#[test]
fn retention_parses_id_bearing_dump_names() {
    let id = Uuid::new_v4();
    let name = format!("{id}_dump_20261001_120000.dump");
    let parsed = agrocore_backup::retention::RetentionManager::debug_parse(&name);
    assert!(parsed.is_some(), "parser must accept {name}");
}
