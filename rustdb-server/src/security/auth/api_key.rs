use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};

use crate::security::auth::hashing::hash_api_key;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiRole
{
    Admin,
    ReadWrite,
    ReadOnly,
}

#[derive(Debug, Clone)]
pub struct ApiKey
{
    pub id: String,
    pub key_hash: String,
    pub role: ApiRole,
    pub created_at: i64,
    pub revoked: bool,
}

#[derive(Debug, Clone)]
pub struct ApiKeyPair
{
    pub key_id: String,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyCreateRequest
{
    pub role: ApiRole,
}
/// Generate a new API key and raw secret pair.
///
/// The returned `ApiKey` is safe to persist in storage.
/// The `ApiKeyPair` contains the raw secret that must be delivered securely.
pub fn generate_api_key(role: ApiRole) -> Result<(ApiKey, ApiKeyPair), argon2::password_hash::Error>
{
    let key_id = uuid::Uuid::new_v4().to_string();
    let secret = generate_secret();
    let key_hash = hash_api_key(&secret)?;

    let api_key = ApiKey {
        id: key_id.clone(),
        key_hash,
        role,
        created_at: now_ns(),
        revoked: false,
    };

    let pair = ApiKeyPair {
        key_id,
        secret,
    };

    Ok((api_key, pair))
}

fn generate_secret() -> String
{
    let mut buffer = [0u8; 32];
    OsRng.fill_bytes(&mut buffer);
    general_purpose::URL_SAFE_NO_PAD.encode(&buffer)
}

fn now_ns() -> i64
{
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}
