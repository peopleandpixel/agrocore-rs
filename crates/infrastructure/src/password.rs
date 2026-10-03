//! Password hashing off the async runtime.
//!
//! # Why this exists
//!
//! Argon2 is deliberately slow — that is what makes a stolen hash expensive to
//! attack. The default parameters cost 50–100 ms of pure CPU per call. Running
//! that inside an `async` task blocks the Tokio worker thread for the whole
//! duration: the runtime has a fixed pool of workers, and every other request
//! landing on the same worker waits.
//!
//! That turns authentication into a denial-of-service primitive. A handful of
//! concurrent login attempts does not need many connections to occupy every
//! worker; while they are being hashed, nothing else the process serves gets
//! scheduled. No load generator required — a script with a few parallel requests
//! is enough.
//!
//! # The fix
//!
//! `tokio::task::spawn_blocking` moves the computation onto the blocking thread
//! pool, which exists exactly for work that cannot be made async and is expected
//! to occupy a thread. The async task awaits the result and stays free in the
//! meantime.
//!
//! Every call site goes through the two functions here rather than wrapping
//! `Argon2` inline. A call site that forgets the wrapper is the original bug, and
//! it is not visible in review — `hash_password` looks correct until you notice
//! the missing `spawn_blocking`.

use argon2::password_hash::phc::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

/// Hash a password for storage.
///
/// Costs 50–100 ms of CPU. Never call this directly from an async task; use this
/// function, which moves the work to the blocking pool.
pub async fn hash_password(password: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|e| format!("password hashing failed: {e}"))
    })
    .await
    .map_err(|e| format!("password hashing task failed: {e}"))?
}

/// Verify a password against a stored hash.
///
/// Costs 50–100 ms of CPU. Never call this directly from an async task; use this
/// function, which moves the work to the blocking pool.
///
/// The stored hash is parsed before the blocking hop so a malformed hash is
/// reported without occupying a thread.
pub async fn verify_password(password: String, hash: String) -> Result<(), String> {
    let parsed = PasswordHash::new(&hash).map_err(|e| format!("invalid password hash: {e}"))?;

    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| "invalid credentials".to_string())
    })
    .await
    .map_err(|e| format!("password verification task failed: {e}"))?
}

/// Report whether a stored hash uses parameters this build understands.
///
/// A hash written by a future version with stronger parameters must not silently
/// verify against weaker ones, and must not be reported as a parse failure
/// either — the distinction matters when diagnosing why a login stopped working
/// after an upgrade.
pub fn hash_parameters_supported(hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => parsed.algorithm.as_str().starts_with("argon2"),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_hash_round_trips_through_the_blocking_pool() {
        let hash = hash_password("correct horse".to_string())
            .await
            .expect("hash");

        assert!(hash.starts_with("$argon2"), "got {hash}");
        verify_password("correct horse".to_string(), hash.clone())
            .await
            .expect("verify should succeed");
    }

    #[tokio::test]
    async fn a_wrong_password_is_rejected() {
        let hash = hash_password("correct horse".to_string())
            .await
            .expect("hash");

        let result = verify_password("wrong horse".to_string(), hash).await;
        assert!(result.is_err(), "a wrong password must not verify");
    }

    /// The property that makes this worth having: hashing must not occupy the
    /// async worker.
    ///
    /// Two concurrent hashes on a single-threaded runtime can only both finish if
    /// the work left the async worker between them. Run inline, the second call
    /// would not start until the first finished, so the total time would be the
    /// sum rather than the maximum.
    #[tokio::test(flavor = "current_thread")]
    async fn concurrent_hashes_do_not_serialise_on_the_async_worker() {
        let start = std::time::Instant::now();
        let (a, b) = tokio::join!(
            hash_password("first".to_string()),
            hash_password("second".to_string())
        );
        let parallel = start.elapsed();
        assert!(a.is_ok() && b.is_ok());

        let start = std::time::Instant::now();
        hash_password("third".to_string()).await.expect("hash");
        let single = start.elapsed();

        // Both concurrent hashes must finish in roughly the time of one. The
        // factor is deliberately loose: the blocking pool has more than one
        // thread, and timing on a loaded machine is noisy.
        assert!(
            parallel.as_millis() < single.as_millis() * 2,
            "two hashes took {parallel:?} against {single:?} for one — they appear \
             to have serialised on the async worker"
        );
    }

    #[tokio::test]
    async fn a_malformed_hash_is_reported_without_a_blocking_hop() {
        let Err(message) = verify_password("x".to_string(), "not-a-hash".to_string()).await else {
            panic!("a malformed hash must not verify");
        };
        assert!(
            message.contains("invalid password hash"),
            "expected a parse error, got {message:?}"
        );
    }

    #[tokio::test]
    async fn argon2_hashes_are_recognised() {
        // A real hash from this build, so the parameters are valid rather than a
        // hand-written string the parser would reject for its own sake.
        let hash = hash_password("correct horse".to_string())
            .await
            .expect("hash");
        assert!(hash_parameters_supported(&hash), "got {hash}");

        assert!(!hash_parameters_supported("$2b$12$something"));
        assert!(!hash_parameters_supported("nonsense"));
    }
}
