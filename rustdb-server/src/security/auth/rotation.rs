use crate::security::auth::api_key::ApiKey;
use crate::security::auth::hashing::hash_api_key;
use std::time::{SystemTime, UNIX_EPOCH};

/// Key rotation state
#[derive(Debug, Clone)]
pub struct ApiKeyRotation
{
    pub current_key_id: String,
    pub previous_key_id: Option<String>,
    pub rotated_at: i64,
}

/// Generate new rotated key pair
pub fn rotate_key(existing: &ApiKey, new_secret: &str) -> ApiKey
{
    ApiKey
    {
        id: uuid::Uuid::new_v4().to_string(),
        key_hash: hash_api_key(new_secret).unwrap(),
        role: existing.role.clone(),
        created_at: now_ns(),
        revoked: false,
    }
}

/// Mark old key as revoked (soft revoke for overlap window)
pub fn revoke_key(mut key: ApiKey) -> ApiKey
{
    key.revoked = true;
    key
}

fn now_ns() -> i64
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}