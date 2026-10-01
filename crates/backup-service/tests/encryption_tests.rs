//! Encryption round-trip tests for the backup service.
//!
//! `EncryptionManager` is pure local crypto (AES-256-GCM), so it is fully
//! testable without any external service.

use agrocore_backup::config::{EncryptionConfig, EncryptionMethod};
use agrocore_backup::encryption::EncryptionManager;
use std::io::Write;

fn temp_paths(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("agrocore-enc-test-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    (dir.join("plain.bin"), dir.join("cipher.bin"))
}

fn aes_manager() -> EncryptionManager {
    EncryptionManager::new(EncryptionConfig {
        default: EncryptionMethod::Aes256Gcm,
        ..Default::default()
    })
}

#[test]
fn test_encryption_method_display() {
    assert_eq!(EncryptionMethod::None.to_string(), "none");
    assert_eq!(EncryptionMethod::Age.to_string(), "age");
    assert_eq!(EncryptionMethod::Aes256Gcm.to_string(), "aes256gcm");
}

#[test]
fn test_encryption_config_validation_age_requires_recipients() {
    let config = EncryptionConfig {
        default: EncryptionMethod::Age,
        age_recipients: vec![],
        ..Default::default()
    };
    assert!(config.validate().is_err());

    let valid = EncryptionConfig {
        default: EncryptionMethod::Age,
        age_recipients: vec!["age1abc".to_string()],
        ..Default::default()
    };
    assert!(valid.validate().is_ok());
}

#[test]
fn test_encryption_config_default() {
    assert_eq!(EncryptionConfig::default().default, EncryptionMethod::Age);
}

#[tokio::test]
async fn test_aes_roundtrip_restores_plaintext() {
    let (plain_path, cipher_path) = temp_paths("roundtrip");
    let plaintext = b"agrocore backup payload -- roundtrip".repeat(64);

    let mut f = std::fs::File::create(&plain_path).expect("create plain");
    f.write_all(&plaintext).expect("write plain");
    drop(f);

    let manager = aes_manager();
    manager
        .encrypt_file(&plain_path, &cipher_path)
        .await
        .expect("encrypt");

    let cipher_bytes = std::fs::read(&cipher_path).expect("read cipher");
    assert_ne!(
        cipher_bytes, plaintext,
        "ciphertext must differ from plaintext"
    );
    assert!(
        cipher_bytes.len() > plaintext.len(),
        "nonce + GCM tag must expand the payload"
    );

    let restored_path = cipher_path.with_file_name("restored.bin");
    manager
        .decrypt_file(&cipher_path, &restored_path)
        .await
        .expect("decrypt");

    assert_eq!(
        std::fs::read(&restored_path).expect("read restored"),
        plaintext,
        "decrypt must be the exact inverse of encrypt"
    );

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}

#[tokio::test]
async fn test_aes_nonce_differs_between_encryptions() {
    let (plain_path, cipher_path) = temp_paths("nonce");
    let plaintext = b"identical content";

    std::fs::write(&plain_path, plaintext).expect("write plain");

    let manager = aes_manager();
    let second = cipher_path.with_file_name("cipher2.bin");

    manager
        .encrypt_file(&plain_path, &cipher_path)
        .await
        .expect("enc1");
    manager
        .encrypt_file(&plain_path, &second)
        .await
        .expect("enc2");

    let a = std::fs::read(&cipher_path).expect("read1");
    let b = std::fs::read(&second).expect("read2");
    assert_ne!(a, b, "a random nonce must make identical inputs differ");

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}

#[tokio::test]
async fn test_aes_decrypt_rejects_tampered_ciphertext() {
    let (plain_path, cipher_path) = temp_paths("tamper");
    std::fs::write(&plain_path, b"authentic payload").expect("write plain");

    let manager = aes_manager();
    manager
        .encrypt_file(&plain_path, &cipher_path)
        .await
        .expect("enc");

    let mut bytes = std::fs::read(&cipher_path).expect("read");
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&cipher_path, &bytes).expect("write tampered");

    let out = cipher_path.with_file_name("tampered-out.bin");
    let err = manager
        .decrypt_file(&cipher_path, &out)
        .await
        .expect_err("GCM authentication must reject modified ciphertext");
    assert!(
        matches!(err, agrocore_backup::error::BackupError::Encryption(_)),
        "expected Encryption error, got {err:?}"
    );

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}

#[tokio::test]
async fn test_aes_decrypt_rejects_truncated_input() {
    let (plain_path, cipher_path) = temp_paths("truncated");
    std::fs::write(&plain_path, b"payload").expect("write");

    let manager = aes_manager();
    manager
        .encrypt_file(&plain_path, &cipher_path)
        .await
        .expect("enc");

    // Shorter than the 12-byte nonce prefix.
    std::fs::write(&cipher_path, b"short").expect("write short");

    let out = cipher_path.with_file_name("short-out.bin");
    let err = manager
        .decrypt_file(&cipher_path, &out)
        .await
        .expect_err("truncated ciphertext must be rejected");
    assert!(err.to_string().contains("too short"), "got {err}");

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}

#[tokio::test]
async fn test_none_method_copies_plaintext() {
    let (plain_path, cipher_path) = temp_paths("none");
    let payload = b"unencrypted archive";
    std::fs::write(&plain_path, payload).expect("write");

    let manager = EncryptionManager::new(EncryptionConfig {
        default: EncryptionMethod::None,
        ..Default::default()
    });
    manager
        .encrypt_file(&plain_path, &cipher_path)
        .await
        .expect("copy");

    assert_eq!(std::fs::read(&cipher_path).expect("read"), payload);

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}

#[tokio::test]
async fn test_unimplemented_methods_report_clear_errors() {
    let (plain_path, cipher_path) = temp_paths("unimpl");
    std::fs::write(&plain_path, b"payload").expect("write");

    for method in [
        EncryptionMethod::Age,
        EncryptionMethod::AwsKms,
        EncryptionMethod::AzureKeyVault,
        EncryptionMethod::GcpKms,
    ] {
        let method_label = method.to_string();
        let manager = EncryptionManager::new(EncryptionConfig {
            default: method,
            ..Default::default()
        });
        let err = manager
            .encrypt_file(&plain_path, &cipher_path)
            .await
            .expect_err("unimplemented backend must not silently succeed");
        assert!(
            err.to_string().contains("not yet implemented"),
            "{method_label} should report not-implemented, got {err}"
        );
    }

    let _ = std::fs::remove_dir_all(plain_path.parent().unwrap());
}
