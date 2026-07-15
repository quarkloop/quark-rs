//! RoleService — roles CRUD + permission grants (org-scoped).
//!
//! Wraps `quark_auth_proto::auth::v1::role_service_client::RoleServiceClient`.
//!
//! Covers all 7 RPCs: `CreateRole`, `GetRole`, `ListRoles`, `UpdateRole`,
//! `DeleteRole`, `GrantPermission`, `RevokePermission`. Every RPC requires a
//! bearer token.

use quark_auth_proto::auth::v1::role_service_client::RoleServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `RoleService`.
pub struct RoleService {
    inner: RoleServiceClient<Channel>,
}

impl RoleService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: RoleServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut RoleServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreateRole` — create a role under an organization.
    pub async fn create(
        &mut self,
        token: &str,
        organization_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Role, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreateRoleRequest {
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_role(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetRole` — fetch a role by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Role, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetRoleRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_role(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListRoles` — paginated role list scoped to an organization.
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
        organization_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListRolesResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListRolesRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
            organization_id: organization_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_roles(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdateRole` — patch a role's name/description.
    pub async fn update(
        &mut self,
        token: &str,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Role, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdateRoleRequest {
            id: id.to_string(),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_role(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteRole` — delete a role by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteRoleRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_role(req).await?;
        Ok(())
    }

    /// `GrantPermission` — grant a permission to a role.
    pub async fn grant_permission(
        &mut self,
        token: &str,
        role_id: &str,
        permission: &str,
    ) -> Result<quark_auth_proto::auth::v1::Role, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GrantPermissionRequest {
            role_id: role_id.to_string(),
            permission: permission.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.grant_permission(req).await?;
        Ok(resp.into_inner())
    }

    /// `RevokePermission` — revoke a permission from a role.
    pub async fn revoke_permission(
        &mut self,
        token: &str,
        role_id: &str,
        permission: &str,
    ) -> Result<quark_auth_proto::auth::v1::Role, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::RevokePermissionRequest {
            role_id: role_id.to_string(),
            permission: permission.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.revoke_permission(req).await?;
        Ok(resp.into_inner())
    }
}
