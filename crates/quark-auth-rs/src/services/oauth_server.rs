//! OAuthServerService — auth-service acting as an OAuth2/OIDC provider.
//!
//! Wraps `quark_auth_proto::auth::v1::oauth_server_service_client::OAuthServerServiceClient`.
//!
//! Covers all 8 RPCs: `OAuthServerAuthorize`, `OAuthServerToken`,
//! `OAuthServerUserInfo`, `OAuthServerClientDynamicRegister`,
//! `OAuthServerGetAuthorization`, `OAuthServerConsent`,
//! `UserListOAuthGrants`, `UserRevokeOAuthGrant`.
//!
//! Anonymous (no bearer token) RPCs: `OAuthServerAuthorize`, `OAuthServerToken`,
//! `OAuthServerUserInfo`, `OAuthServerClientDynamicRegister`.
//!
//! Authenticated (bearer token required) RPCs: `OAuthServerGetAuthorization`,
//! `OAuthServerConsent`, `UserListOAuthGrants`, `UserRevokeOAuthGrant`.

use quark_auth_proto::auth::v1::o_auth_server_service_client::OAuthServerServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `OAuthServerService`.
pub struct OAuthServerService {
    inner: OAuthServerServiceClient<Channel>,
}

impl OAuthServerService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: OAuthServerServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut OAuthServerServiceClient<Channel> {
        &mut self.inner
    }

    // ─── public OAuth2 endpoints ─────────────────────────────────────────────

    /// `OAuthServerAuthorize` — authorization endpoint; returns a redirect URL.
    pub async fn authorize(
        &mut self,
        response_type: &str,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
        state: &str,
        code_challenge: &str,
        code_challenge_method: &str,
        nonce: &str,
    ) -> Result<quark_auth_proto::auth::v1::RedirectResponse, AuthClientError> {
        let resp = self
            .inner
            .o_auth_server_authorize(quark_auth_proto::auth::v1::OAuthAuthorizeRequest {
                response_type: response_type.to_string(),
                client_id: client_id.to_string(),
                redirect_uri: redirect_uri.to_string(),
                scope: scope.to_string(),
                state: state.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
                nonce: nonce.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `OAuthServerToken` — token endpoint (`authorization_code` or `refresh_token` grant).
    pub async fn token(
        &mut self,
        grant_type: &str,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
        code_verifier: &str,
        refresh_token: &str,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let resp = self
            .inner
            .o_auth_server_token(quark_auth_proto::auth::v1::OAuthTokenRequest {
                grant_type: grant_type.to_string(),
                code: code.to_string(),
                redirect_uri: redirect_uri.to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                code_verifier: code_verifier.to_string(),
                refresh_token: refresh_token.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `OAuthServerUserInfo` — OIDC UserInfo endpoint. (Server resolves the
    /// caller from the access token in the `Authorization` header; this client
    /// does not attach one — callers that need it can use [`Self::user_info_with_token`].)
    pub async fn user_info(
        &mut self,
    ) -> Result<quark_auth_proto::auth::v1::UserInfoResponse, AuthClientError> {
        let resp = self.inner.o_auth_server_user_info(()).await?;
        Ok(resp.into_inner())
    }

    /// `OAuthServerUserInfo` with an explicit bearer (access) token attached.
    pub async fn user_info_with_token(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::UserInfoResponse, AuthClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.o_auth_server_user_info(req).await?;
        Ok(resp.into_inner())
    }

    /// `OAuthServerClientDynamicRegister` — RFC 7591 dynamic client registration.
    pub async fn client_dynamic_register(
        &mut self,
        redirect_uris: &[&str],
        client_name: &str,
        scopes: &[&str],
        token_endpoint_auth_method: &str,
    ) -> Result<quark_auth_proto::auth::v1::OAuthClient, AuthClientError> {
        let resp = self
            .inner
            .o_auth_server_client_dynamic_register(
                quark_auth_proto::auth::v1::OAuthClientDynamicRegisterRequest {
                    redirect_uris: redirect_uris.iter().map(|s| s.to_string()).collect(),
                    client_name: client_name.to_string(),
                    scopes: scopes.iter().map(|s| s.to_string()).collect(),
                    token_endpoint_auth_method: token_endpoint_auth_method.to_string(),
                },
            )
            .await?;
        Ok(resp.into_inner())
    }

    // ─── authenticated (resource-owner) endpoints ───────────────────────────

    /// `OAuthServerGetAuthorization` — fetch a pending authorization by ID.
    /// Requires a bearer token.
    pub async fn get_authorization(
        &mut self,
        token: &str,
        authorization_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::OAuthAuthorization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::OAuthGetAuthorizationRequest {
            authorization_id: authorization_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.o_auth_server_get_authorization(req).await?;
        Ok(resp.into_inner())
    }

    /// `OAuthServerConsent` — grant or deny consent for a pending authorization.
    /// Requires a bearer token. Returns tokens on consent.
    pub async fn consent(
        &mut self,
        token: &str,
        authorization_id: &str,
        consent: bool,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::OAuthConsentRequest {
            authorization_id: authorization_id.to_string(),
            consent,
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.o_auth_server_consent(req).await?;
        Ok(resp.into_inner())
    }

    /// `UserListOAuthGrants` — list the OAuth grants the authenticated user has
    /// consented to. Requires a bearer token.
    pub async fn list_grants(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListOAuthGrantsResponse, AuthClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.user_list_o_auth_grants(req).await?;
        Ok(resp.into_inner())
    }

    /// `UserRevokeOAuthGrant` — revoke an OAuth grant by ID. Requires a bearer token.
    pub async fn revoke_grant(
        &mut self,
        token: &str,
        grant_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::RevokeOAuthGrantRequest {
            grant_id: grant_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.user_revoke_o_auth_grant(req).await?;
        Ok(())
    }
}
