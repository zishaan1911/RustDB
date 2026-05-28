use crate::security::auth::{
    api_key::{ApiKey, ApiRole},
    hashing::verify_api_key,
};

#[derive(Debug, Clone)]
pub struct AuthContext
{
    pub key_id: String,
    pub role: ApiRole,
    pub issued_at: i64,
}

/// Key format:
/// Authorization: Bearer <key_id>.<secret>
pub fn split_bearer(token: &str) -> Option<(&str, &str)>
{
    let cleaned = token.strip_prefix("Bearer ")?;
    let mut parts = cleaned.splitn(2, '.');

    let key_id = parts.next()?;
    let secret = parts.next()?;

    Some((key_id.trim(), secret.trim()))
}

/// Verify API key
pub fn verify_key(raw_secret: &str, stored: &ApiKey) -> bool
{
    if stored.revoked
    {
        return false;
    }

    verify_api_key(&stored.key_hash, raw_secret)
}

/// Strict role hierarchy
pub fn has_permission(user: &ApiRole, required: &ApiRole) -> bool
{
    match (user, required)
    {
        (ApiRole::Admin, _) => true,

        (ApiRole::ReadWrite, ApiRole::ReadWrite) => true,
        (ApiRole::ReadWrite, ApiRole::ReadOnly) => true,

        (ApiRole::ReadOnly, ApiRole::ReadOnly) => true,

        _ => false,
    }
}