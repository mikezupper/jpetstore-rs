//! Password hashing, the whole policy in two functions. argon2 with default
//! parameters (argon2id), a random salt per password, and the standard PHC
//! string format in the database — so the hash column documents its own
//! algorithm and can be re-hashed forward when parameters age.
//!
//! Hashing is deliberately slow (~tens of milliseconds); that's the security
//! property, not a bug. At this app's scale calling it inline is fine — a
//! high-traffic service would wrap these in tokio::task::spawn_blocking.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

use crate::web::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::Hashing)
}

/// Wrong password, unknown hash format, corrupted column — all the same
/// answer. Sign-in doesn't need to know which; the caller gets a bool.
pub fn verify(stored_hash: &str, password: &str) -> bool {
    PasswordHash::new(stored_hash)
        .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrips() {
        let hash = hash_password("j2ee").unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify(&hash, "j2ee"));
        assert!(!verify(&hash, "j2ee "));
    }

    #[test]
    fn garbage_hashes_verify_nothing() {
        assert!(!verify("j2ee", "j2ee")); // the original's column contents
        assert!(!verify("", "anything"));
    }
}
