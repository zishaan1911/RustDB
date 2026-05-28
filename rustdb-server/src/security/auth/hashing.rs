use argon2::{
    Argon2,
    password_hash::{SaltString, PasswordHash, PasswordHasher, PasswordVerifier},
};
use rand_core::OsRng;

/// Production-grade Argon2 config
fn argon2_instance() -> Argon2<'static>
{
    Argon2::default()
}

/// Hash API key (Argon2id)
/// Never store raw keys
pub fn hash_api_key(raw: &str) -> Result<String, argon2::password_hash::Error>
{
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = argon2_instance();

    let hash = argon2.hash_password(raw.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

/// Constant-time verification
pub fn verify_api_key(hash: &str, raw: &str) -> bool
{
    let parsed = match PasswordHash::new(hash)
    {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };

    argon2_instance()
        .verify_password(raw.as_bytes(), &parsed)
        .is_ok()
}