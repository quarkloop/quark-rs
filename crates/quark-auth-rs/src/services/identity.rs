//! IdentityService — OAuth identity management for the authenticated user.
//!
//! Wraps `quark_auth_proto::auth::v1::identity_service_client::IdentityServiceClient`.
//!
//! Covers all 3 RPCs: `ListIdentities`, `LinkIdentity`, `DeleteIdentity`.
//! Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::identity_service_client::IdentityServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `IdentityService`.
pub struct IdentityService {
    inner: IdentityServiceClient<Channel>,
}

impl IdentityService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: IdentityServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut IdentityServiceClient<Channel> {
        &mut self.inner
    }

    /// `ListIdentities` — list the OAuth identities linked to the authenticated user.
    pub async fn list(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListIdentitiesResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListIdentitiesRequest {});
        attach_bearer(&mut req, token);
        let resp = self.inner.list_identities(req).await?;
        Ok(resp.into_inner())
    }

    /// `LinkIdentity` — begin linking a new external OAuth identity; returns a
    /// redirect URL.
    pub async fn link(
        &mut self,
        token: &str,
        provider: &str,
        scopes: &str,
        redirect_to: &str,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<quark_auth_proto::auth::v1::RedirectResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::LinkIdentityRequest {
            provider: provider.to_string(),
            scopes: scopes.to_string(),
            redirect_to: redirect_to.to_string(),
            code_challenge: code_challenge.to_string(),
            code_challenge_method: code_challenge_method.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.link_identity(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteIdentity` — unlink an OAuth identity by ID.
    pub async fn delete(
        &mut self,
        token: &str,
        identity_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteIdentityRequest {
            identity_id: identity_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_identity(req).await?;
        Ok(())
    }
}
