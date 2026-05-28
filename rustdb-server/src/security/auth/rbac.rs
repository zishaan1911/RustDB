use crate::security::auth::api_key::ApiRole;

#[derive(Debug, Clone, PartialEq)]
pub enum Permission
{
    Read,
    Write,
    Admin,
}

/// Map role → permissions
pub fn role_permissions(role: &ApiRole) -> Vec<Permission>
{
    match role
    {
        ApiRole::Admin => vec![Permission::Read, Permission::Write, Permission::Admin],
        ApiRole::ReadWrite => vec![Permission::Read, Permission::Write],
        ApiRole::ReadOnly => vec![Permission::Read],
    }
}

/// Check permission
pub fn has_permission(role: &ApiRole, required: Permission) -> bool
{
    role_permissions(role).contains(&required)
}