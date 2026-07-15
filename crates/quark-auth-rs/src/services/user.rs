//! UserService — user CRUD + role assignment.
//!
//! Wraps `quark_auth_proto::auth::v1::user_service_client::UserServiceClient`.
//!
//! Covers all 7 RPCs: `CreateUser`, `GetUser`, `ListUsers`, `UpdateUser`,
//! `DeleteUser`, `AssignRole`, `RevokeRole`. Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::user_service_client::UserServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `UserService`.
pub struct UserService {
    inner: UserServiceClient<Channel>,
}

impl UserService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: UserServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut UserServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreateUser` — create a user inside an organization.
    pub async fn create(
        &mut self,
        token: &str,
        organization_id: &str,
        handle: &str,
        email: &str,
        display_name: &str,
        api_key: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreateUserRequest {
            organization_id: organization_id.to_string(),
            handle: handle.to_string(),
            email: email.to_string(),
            display_name: display_name.to_string(),
            api_key: api_key.unwrap_or_default().to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetUser` — fetch a user by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetUserRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListUsers` — paginated list of users, optionally scoped to an org.
    /// Pass `""` for `organization_id` to list across orgs (subject to server authz).
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
        organization_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListUsersResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListUsersRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
            organization_id: organization_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_users(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdateUser` — patch a user. Pass `None` for fields you don't want to change.
    pub async fn update(
        &mut self,
        token: &str,
        id: &str,
        display_name: Option<&str>,
        email: Option<&str>,
        api_key: Option<&str>,
        phone: Option<&str>,
        password: Option<&str>,
        app_metadata: Option<&str>,
        user_metadata: Option<&str>,
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdateUserRequest {
            id: id.to_string(),
            display_name: display_name.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            api_key: api_key.map(|s| s.to_string()),
            phone: phone.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            app_metadata: app_metadata.map(|s| s.to_string()),
            user_metadata: user_metadata.map(|s| s.to_string()),
            code_challenge: code_challenge.map(|s| s.to_string()),
            code_challenge_method: code_challenge_method.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteUser` — delete a user by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteUserRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_user(req).await?;
        Ok(())
    }

    /// `AssignRole` — assign a role to a user.
    pub async fn assign_role(
        &mut self,
        token: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AssignRoleRequest {
            user_id: user_id.to_string(),
            role_id: role_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.assign_role(req).await?;
        Ok(resp.into_inner())
    }

    /// `RevokeRole` — revoke a role from a user.
    pub async fn revoke_role(
        &mut self,
        token: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::RevokeRoleRequest {
            user_id: user_id.to_string(),
            role_id: role_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.revoke_role(req).await?;
        Ok(resp.into_inner())
    }
}
