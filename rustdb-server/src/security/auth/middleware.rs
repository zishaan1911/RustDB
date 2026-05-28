use std::sync::Arc;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::security::auth::{
    audit::{write_audit, AuditEvent},
    rate_limit::check_rate_limit,
    verification::{AuthContext, split_bearer, verify_key},
    api_key::ApiKey,
};

pub trait ApiKeyStore
{
    fn find_by_id(&self, id: &str) -> Result<Option<ApiKey>, ()>;
}

static DEV_MODE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn set_dev_mode(v: bool)
{
    DEV_MODE.store(v, std::sync::atomic::Ordering::Relaxed);
}

/// ZERO TRUST PIPELINE:
/// 1. Dev mode check
/// 2. Rate limit
/// 3. Auth
/// 4. RBAC handled later in handler/macros
/// 5. Audit log ALWAYS
pub async fn auth_middleware<B, S>(
    mut req: Request<B>,
    next: Next<B>,
    store: Arc<S>,
) -> Result<Response, StatusCode>
where
    S: ApiKeyStore + Send + Sync + 'static,
{
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if DEV_MODE.load(std::sync::atomic::Ordering::Relaxed)
    {
        return Ok(next.run(req).await);
    }

    let header = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let (key_id, secret) =
        split_bearer(header).ok_or(StatusCode::UNAUTHORIZED)?;

    // RATE LIMIT (pre-auth gate)
    if !check_rate_limit(key_id)
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let stored = store.find_by_id(key_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let ok = verify_key(secret, &stored);

    if !ok
    {
        write_audit(AuditEvent {
            key_id: key_id.to_string(),
            action: "auth_failed".into(),
            ip,
            success: false,
        });

        return Err(StatusCode::UNAUTHORIZED);
    }

    let ctx = AuthContext
    {
        key_id: stored.id.clone(),
        role: stored.role.clone(),
        issued_at: stored.created_at,
    };

    req.extensions_mut().insert(ctx);

    write_audit(AuditEvent {
        key_id: stored.id,
        action: "auth_success".into(),
        ip,
        success: true,
    });

    Ok(next.run(req).await)
}