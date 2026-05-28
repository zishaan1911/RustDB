use serde::{Serialize, Deserialize};

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