#[macro_export]
macro_rules! require_role {
    ($req:expr, $role:expr) => {{
        use $crate::security::auth::rbac::{has_permission, Permission};

        let ctx = $req
            .extensions()
            .get::<$crate::security::auth::verification::AuthContext>();

        if ctx.is_none() {
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }

        let ctx = ctx.unwrap();

        let ok = has_permission(&ctx.role, $role);

        if !ok {
            return Err(axum::http::StatusCode::FORBIDDEN);
        }
    }};
}
