use std::sync::Arc;
use axum::{body::Body, extract::State, http::{Request, StatusCode}};
use axum::routing::get;
use axum::Router;
use serial_test::serial;
use tower::ServiceExt;

use rustdb_server::security::auth::{
    api_key::{ApiKey, ApiRole},
    hashing::{hash_api_key, verify_api_key},
    middleware::{auth_middleware, ApiKeyStore, set_dev_mode},
    rate_limit::check_rate_limit,
    verification::{split_bearer, verify_key, AuthContext},
    rbac::{has_permission as rbac_has_permission, Permission},
    verification::has_permission as role_has_permission,
};

struct DummyStore {
    key: ApiKey,
}

impl ApiKeyStore for DummyStore {
    fn find_by_id(&self, id: &str) -> Result<Option<ApiKey>, ()> {
        if self.key.id == id {
            Ok(Some(self.key.clone()))
        } else {
            Ok(None)
        }
    }
}

async fn call_auth_request(store: Arc<DummyStore>, req: Request<Body>) -> StatusCode {
    let app = Router::new()
        .route("/test", get(|| async { StatusCode::OK }))
        .layer(axum::middleware::from_fn_with_state(store.clone(), move |State(store): State<Arc<DummyStore>>, req, next| {
            let store = store.clone();
            async move { auth_middleware(req, next, store).await }
        }));

    app.oneshot(req).await.unwrap().status()
}

#[test]
fn require_role_macro_allows_admin() {
    let mut req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    req.extensions_mut().insert(AuthContext {
        key_id: "admin-key".into(),
        role: ApiRole::Admin,
        issued_at: 1,
    });

    fn handler(req: Request<Body>) -> Result<&'static str, StatusCode> {
        require_role!(req, Permission::Write);
        Ok("allowed")
    }

    let result = handler(req);
    assert_eq!(result.unwrap(), "allowed");
}

#[test]
fn require_role_macro_blocks_insufficient_role() {
    let mut req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    req.extensions_mut().insert(AuthContext {
        key_id: "read-only-key".into(),
        role: ApiRole::ReadOnly,
        issued_at: 1,
    });

    fn handler(req: Request<Body>) -> Result<&'static str, StatusCode> {
        require_role!(req, Permission::Write);
        Ok("allowed")
    }

    let result = handler(req);
    assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
}

#[serial]
#[tokio::test]
async fn auth_middleware_allows_valid_key() {
    let secret = "super-secret";
    let hash = hash_api_key(secret).expect("hash generation failed");
    let stored = ApiKey {
        id: "valid-key".to_string(),
        key_hash: hash,
        role: ApiRole::ReadWrite,
        created_at: 1,
        revoked: false,
    };

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}.{}", stored.id, secret))
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::OK);
}

#[serial]
#[tokio::test]
async fn auth_middleware_rejects_missing_header() {
    let stored = ApiKey {
        id: "missing-header".to_string(),
        key_hash: hash_api_key("ignored").unwrap(),
        role: ApiRole::ReadOnly,
        created_at: 1,
        revoked: false,
    };

    let req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[serial]
#[tokio::test]
async fn auth_middleware_rejects_invalid_bearer_format() {
    let stored = ApiKey {
        id: "bad-format".to_string(),
        key_hash: hash_api_key("ignored").unwrap(),
        role: ApiRole::ReadOnly,
        created_at: 1,
        revoked: false,
    };

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", "Basic not-a-bearer")
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[serial]
#[tokio::test]
async fn auth_middleware_rejects_unknown_key_id() {
    let stored = ApiKey {
        id: "known-key".to_string(),
        key_hash: hash_api_key("secret").unwrap(),
        role: ApiRole::ReadOnly,
        created_at: 1,
        revoked: false,
    };

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer unknown-key.secret")
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[serial]
#[tokio::test]
async fn auth_middleware_rejects_wrong_secret() {
    let stored = ApiKey {
        id: "valid-key".to_string(),
        key_hash: hash_api_key("correct-secret").unwrap(),
        role: ApiRole::ReadOnly,
        created_at: 1,
        revoked: false,
    };

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer valid-key.wrong-secret")
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[serial]
#[tokio::test]
async fn auth_middleware_rejects_revoked_key() {
    let secret = "secret-revoked";
    let stored = ApiKey {
        id: "revoked-key".to_string(),
        key_hash: hash_api_key(secret).unwrap(),
        role: ApiRole::ReadWrite,
        created_at: 1,
        revoked: true,
    };

    let req = Request::builder()
        .uri("/test")
        .header("Authorization", format!("Bearer {}.{}", stored.id, secret))
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: stored }), req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[serial]
#[tokio::test]
async fn auth_middleware_allows_dev_mode_without_authorization() {
    set_dev_mode(true);

    let req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let status = call_auth_request(Arc::new(DummyStore { key: ApiKey {
        id: "dev-mode".into(),
        key_hash: hash_api_key("irrelevant").unwrap(),
        role: ApiRole::Admin,
        created_at: 1,
        revoked: false,
    }}), req).await;

    set_dev_mode(false);

    assert_eq!(status, StatusCode::OK);
}

#[serial]
#[tokio::test]
async fn rate_limit_denies_after_exhaustion() {
    let key_id = "rate-limit-test";

    for _ in 0..100 {
        assert!(check_rate_limit(key_id));
    }

    assert!(!check_rate_limit(key_id));
}

#[serial]
#[test]
fn split_bearer_parses_valid_token() {
    let token = "Bearer key123.secret-value";
    let result = split_bearer(token);
    assert_eq!(result, Some(("key123", "secret-value")));
}

#[serial]
#[test]
fn split_bearer_rejects_invalid_token() {
    assert!(split_bearer("Basic abc").is_none());
    assert!(split_bearer("Bearer no-dot").is_none());
}

#[serial]
#[test]
fn hashing_and_verification_work_together() {
    let secret = "verify-this";
    let hash = hash_api_key(secret).expect("hash must be generated");
    assert!(verify_api_key(&hash, secret));
    assert!(!verify_api_key(&hash, "wrong-secret"));
}

#[serial]
#[test]
fn verify_key_rejects_revoked_key() {
    let secret = "prime-secret";
    let stored = ApiKey {
        id: "revoked".into(),
        key_hash: hash_api_key(secret).unwrap(),
        role: ApiRole::ReadOnly,
        created_at: 1,
        revoked: true,
    };

    assert!(!verify_key(secret, &stored));
}

#[serial]
#[test]
fn verification_has_permission_behaviour() {
    assert!(role_has_permission(&ApiRole::Admin, &ApiRole::ReadOnly));
    assert!(role_has_permission(&ApiRole::ReadWrite, &ApiRole::ReadOnly));
    assert!(role_has_permission(&ApiRole::ReadWrite, &ApiRole::ReadWrite));
    assert!(!role_has_permission(&ApiRole::ReadOnly, &ApiRole::ReadWrite));
}

#[serial]
#[test]
fn rbac_has_permission_behaviour() {
    assert!(rbac_has_permission(&ApiRole::Admin, Permission::Admin));
    assert!(rbac_has_permission(&ApiRole::ReadWrite, Permission::Read));
    assert!(!rbac_has_permission(&ApiRole::ReadOnly, Permission::Write));
}
