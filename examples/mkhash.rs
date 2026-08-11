//! Generate an argon2 PHC hash for a password — used to produce the seed
//! hashes in migrations/0003_signon_seed.sql, and handy if you want your
//! own demo users:
//!
//!     cargo run --example mkhash -- <password>

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;

fn main() {
    let password = std::env::args().nth(1).expect("usage: mkhash <password>");
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing failed");
    println!("{hash}");
}
