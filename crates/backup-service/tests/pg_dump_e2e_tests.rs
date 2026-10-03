//! End-to-end backup tests against a real PostgreSQL instance.
//!
//! These are ignored by default because they need a live database:
//!
//! ```bash
//! DATABASE_URL=postgresql://agrocore:agrocore@localhost:5432/agrocore \
//!   cargo test -p agrocore-backup --test pg_dump_e2e_tests -- --ignored
//! ```
//!
//! They verify the streaming dump/restore path really works against
//! `pg_dump`/`pg_restore` rather than only compiling.

use agrocore_backup::config::{BackupTarget, PgDumpConfig, TargetEncryption};
use agrocore_backup::pg_dump::PgDump;
use agrocore_backup::storage::{StorageBackend, StorageBackendTrait};
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "agrocore-pgdump-{name}-{}-{:?}",
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

async fn pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    match PgPoolOptions::new().max_connections(1).connect(&url).await {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("skipping: cannot reach database: {e}");
            None
        }
    }
}

#[tokio::test]
#[ignore = "requires a live PostgreSQL instance"]
async fn dump_creates_restorable_object() {
    let Some(pool) = pool().await else {
        panic!("DATABASE_URL must point at a reachable database");
    };
    let dir = TempDir::new("dump");
    let target = local_target(&dir.0);

    let backend = StorageBackend::new(vec![target.clone()])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);

    let pg = PgDump::new(pool, PgDumpConfig::default());

    // Prefixing with the job id mirrors how run_backup names dump objects.
    let job_id = Uuid::new_v4();
    let size = pg
        .dump_to_storage(&storage, &target, &format!("{job_id}_"), job_id)
        .await
        .expect("pg_dump to storage");

    assert!(size > 0, "dump must not be empty");

    let objects = storage.list_objects(&target, "").await.expect("list");
    let dump_objects: Vec<_> = objects
        .iter()
        .filter(|(n, _)| n.ends_with(".dump"))
        .collect();
    assert_eq!(
        dump_objects.len(),
        1,
        "exactly one dump object: {objects:?}"
    );

    let (name, stored_size) = dump_objects[0];
    assert!(
        name.contains(&job_id.to_string()),
        "dump name must embed the job id, got {name}"
    );
    assert_eq!(*stored_size, size, "stored size must match reported size");

    // A dump written in custom format must be recognised by pg_restore.
    let output = std::process::Command::new("pg_restore")
        .args(["--list", dir.0.join(name).to_str().expect("utf8 path")])
        .output()
        .expect("run pg_restore --list");
    assert!(
        output.status.success(),
        "dump is not valid pg_restore format: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
#[ignore = "requires a live PostgreSQL instance"]
async fn restore_roundtrips_data() {
    let Some(pool) = pool().await else {
        panic!("DATABASE_URL must point at a reachable database");
    };
    let dir = TempDir::new("restore");
    let target = local_target(&dir.0);

    let backend = StorageBackend::new(vec![target.clone()])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);

    let pg = PgDump::new(pool.clone(), PgDumpConfig::default());
    let job_id = Uuid::new_v4();
    pg.dump_to_storage(&storage, &target, &format!("{job_id}_"), job_id)
        .await
        .expect("dump");

    let objects = storage.list_objects(&target, "").await.expect("list");
    let dump_name = objects
        .iter()
        .find(|(n, _)| n.ends_with(".dump"))
        .map(|(n, _)| n.clone())
        .expect("dump object");

    // Restore into a scratch database so the source stays intact.
    // Swap the database name in the connection URL.
    let admin_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let (scheme_rest, _db) = admin_url.rsplit_once('/').expect("database url");
    let scratch = "agrocore_restore_test";
    let scratch_url = format!("{scheme_rest}/{scratch}");

    sqlx::query(&format!("DROP DATABASE IF EXISTS {scratch}"))
        .execute(&pool)
        .await
        .ok();
    sqlx::query(&format!("CREATE DATABASE {scratch}"))
        .execute(&pool)
        .await
        .expect("create scratch database");

    let result = pg
        .restore_into_named_database(&storage, &target, &dump_name, &scratch_url)
        .await;
    result.expect("pg_restore from storage");

    // Verify the restored database has the same rows as the source before
    // tearing the scratch database down.
    let scratch_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&scratch_url)
        .await
        .expect("connect to scratch db");

    let src: (i64,) = sqlx::query_as("SELECT count(*) FROM sites")
        .fetch_one(&pool)
        .await
        .expect("count sites in source");
    let dst: (i64,) = sqlx::query_as("SELECT count(*) FROM sites")
        .fetch_one(&scratch_pool)
        .await
        .expect("count sites in restored db");

    drop(scratch_pool);
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {scratch}"))
        .execute(&pool)
        .await
        .ok();

    assert_eq!(src.0, dst.0, "row counts must match after restore");
    assert!(src.0 > 0, "source database should contain data");
}

#[tokio::test]
#[ignore = "requires a live PostgreSQL instance"]
async fn dump_rejects_invalid_connection() {
    // Point at a closed port: pg_dump must surface an error, not an empty dump.
    // The env var is process-wide and other tests read it, so restore it after.
    let previous = std::env::var("DATABASE_URL").ok();
    unsafe { std::env::set_var("DATABASE_URL", "postgresql://nobody@127.0.0.1:1/none") };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgresql://nobody@127.0.0.1:1/none")
        .expect("lazy pool");
    let dir = TempDir::new("badurl");
    let target = local_target(&dir.0);

    let backend = StorageBackend::new(vec![target.clone()])
        .await
        .expect("backend");
    let storage: Arc<dyn StorageBackendTrait> = Arc::new(backend);

    let pg = PgDump::new(pool, PgDumpConfig::default());
    let result = pg
        .dump_to_storage(
            &storage,
            &target,
            &format!("{}_", Uuid::new_v4()),
            Uuid::new_v4(),
        )
        .await;

    assert!(
        result.is_err(),
        "an unreachable database must fail loudly, not produce an empty dump"
    );

    // Leave the environment as we found it.
    unsafe {
        match previous {
            Some(v) => std::env::set_var("DATABASE_URL", v),
            None => std::env::remove_var("DATABASE_URL"),
        }
    }
    let _ = Utc::now();
}
