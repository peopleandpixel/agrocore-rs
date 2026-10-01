//! Retention policy tests.
//!
//! Exercises GFS selection and the timestamp parsing that decides which
//! storage objects retention is allowed to touch.

use chrono::{DateTime, Duration, TimeZone, Utc};
use std::sync::Arc;

use agrocore_backup::config::{BackupTarget, RetentionConfig, TargetEncryption};
use agrocore_backup::storage::{StorageBackend, StorageBackendTrait};

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "agrocore-retention-{name}-{}-{:?}",
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

/// Write a dump object whose name encodes `when`, as pg_dump does.
fn write_dump(dir: &std::path::Path, when: DateTime<Utc>, size: usize) -> String {
    let name = format!("dump_{}.dump", when.format("%Y%m%d_%H%M%S"));
    std::fs::write(dir.join(&name), vec![0u8; size]).expect("write dump");
    name
}

fn retention_config() -> RetentionConfig {
    RetentionConfig {
        daily: 3,
        weekly: 2,
        monthly: 1,
        yearly: 1,
        // No grace period so the age filter does not hide the selection logic.
        grace_period_days: 0,
        ..Default::default()
    }
}

#[tokio::test]
async fn retention_deletes_backups_beyond_policy() {
    let dir = TempDir::new("delete");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    // 10 daily dumps; the policy keeps 3.
    let now = Utc::now();
    let mut names = Vec::new();
    for day in 0..10 {
        names.push(write_dump(&dir.0, now - Duration::days(day), 16));
    }
    // An unrelated object that retention must not touch.
    std::fs::write(dir.0.join("notes.txt"), b"keep me").expect("write notes");

    let manager = agrocore_backup::retention::RetentionManager::new(retention_config());

    manager
        .cleanup(&storage, std::slice::from_ref(&target))
        .await
        .expect("cleanup");

    let remaining = std::fs::read_dir(&dir.0)
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().to_string_lossy().ends_with(".dump"))
        .count();

    // GFS: 3 daily + 2 weekly slots are filled by the 10 dumps written here
    // (days 0-3 -> daily, days 4-9 -> weekly), so exactly 5 must survive.
    assert_eq!(
        remaining, 5,
        "policy keeps 3 daily + 2 weekly dumps, {remaining} survived"
    );
    assert!(
        dir.0.join("notes.txt").exists(),
        "retention must not delete unmanaged objects"
    );
}

#[tokio::test]
async fn retention_leaves_recent_backups_alone() {
    let dir = TempDir::new("recent");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    let now = Utc::now();
    for day in 0..2 {
        write_dump(&dir.0, now - Duration::days(day), 8);
    }

    agrocore_backup::retention::RetentionManager::new(retention_config())
        .cleanup(&storage, &[target])
        .await
        .expect("cleanup");

    let remaining = std::fs::read_dir(&dir.0)
        .expect("read dir")
        .filter_map(Result::ok)
        .count();
    assert_eq!(remaining, 2, "recent backups are within policy");
}

#[tokio::test]
async fn retention_respects_grace_period() {
    let dir = TempDir::new("grace");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    let now = Utc::now();
    // 8 dumps, all far older than the 30-day grace period.
    for day in 0..8 {
        write_dump(&dir.0, now - Duration::days(40 + day), 12);
    }

    let config = RetentionConfig {
        grace_period_days: 30,
        ..retention_config()
    };
    agrocore_backup::retention::RetentionManager::new(config)
        .cleanup(&storage, &[target])
        .await
        .expect("cleanup");

    let remaining = std::fs::read_dir(&dir.0)
        .expect("read dir")
        .filter_map(Result::ok)
        .count();
    assert!(
        remaining <= 3,
        "grace period expired, old dumps should be pruned, {remaining} remain"
    );
}

#[tokio::test]
async fn retention_ignores_non_timestamp_objects() {
    let dir = TempDir::new("noname");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    // Names that end in .dump but carry no parseable timestamp.
    for name in ["dump_notadate.dump", "dump.dump", "manifests/x.json"] {
        let parent = std::path::Path::new(name)
            .parent()
            .unwrap_or(std::path::Path::new(""));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(dir.0.join(parent)).expect("mkdir");
        }
        std::fs::write(dir.0.join(name), b"x").expect("write");
    }

    agrocore_backup::retention::RetentionManager::new(retention_config())
        .cleanup(&storage, &[target])
        .await
        .expect("cleanup");

    for name in ["dump_notadate.dump", "dump.dump", "manifests/x.json"] {
        assert!(
            dir.0.join(name).exists(),
            "{name} has no timestamp and must be left alone"
        );
    }
}

#[tokio::test]
async fn retention_on_empty_storage_is_noop() {
    let dir = TempDir::new("emptystore");
    let backend = StorageBackend::new(vec![local_target(&dir.0)])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);
    let target = local_target(&dir.0);

    agrocore_backup::retention::RetentionManager::new(retention_config())
        .cleanup(&storage, &[target])
        .await
        .expect("cleanup on empty storage must succeed");
}

/// Sanity check that the dump timestamp format matches what pg_dump writes.
#[test]
fn dump_timestamp_format_round_trips() {
    let when = Utc.with_ymd_and_hms(2026, 3, 15, 14, 30, 5).unwrap();
    let name = format!("db/dump_{}.dump", when.format("%Y%m%d_%H%M%S"));
    assert_eq!(name, "db/dump_20260315_143005.dump");

    let parsed =
        chrono::NaiveDateTime::parse_from_str("20260315 143005", "%Y%m%d %H%M%S").expect("parse");
    assert_eq!(
        DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc),
        when
    );
}
/// Direct check of the timestamp parser used by retention.
#[test]
fn retention_parser_accepts_generated_names() {
    let when = Utc::now();
    let name = format!("dump_{}.dump", when.format("%Y%m%d_%H%M%S"));
    let listed = agrocore_backup::retention::RetentionManager::debug_parse(&name);
    assert!(listed.is_some(), "parser rejected generated name {name}");
}
