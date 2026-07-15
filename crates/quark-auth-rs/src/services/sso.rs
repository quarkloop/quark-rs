//! SSOService — SAML SSO entry points.
//!
//! Wraps `quark_auth_proto::auth::v1::sso_service_client::SsoServiceClient`.
//!
//! Covers all 3 RPCs: `SSORedirect`, `SAMLACS`, `SAMLMetadata`. All three are
//! anonymous (no bearer token) — they're part of the browser SSO flow.

use quark_auth_proto::auth::v1::sso_service_client::SsoServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;

/// Client for `SSOService`.
pub struct SsoService {
    inner: SsoServiceClient<Channel>,
}

impl SsoService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: SsoServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut SsoServiceClient<Channel> {
        &mut self.inner
    }

    /// `SSORedirect` — begin a SAML SSO flow. Pass either a `provider_id` or a
    /// `domain` (for auto-detection). Returns a redirect URL to the IdP.
    pub async fn redirect(
        &mut self,
        provider_id: &str,
        domain: &str,
        redirect_to: &str,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<quark_auth_proto::auth::v1::RedirectResponse, AuthClientError> {
        let resp = self
            .inner
            .sso_redirect(quark_auth_proto::auth::v1::SsoRedirectRequest {
                provider_id: provider_id.to_string(),
                domain: domain.to_string(),
                redirect_to: redirect_to.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `SAMLACS` — SAML Assertion Consumer Service: exchange a SAML response for tokens.
    pub async fn acs(
        &mut self,
        saml_response: &str,
        relay_state: &str,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let resp = self
            .inner
            .samlacs(quark_auth_proto::auth::v1::SamlacsRequest {
                saml_response: saml_response.to_string(),
                relay_state: relay_state.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `SAMLMetadata` — Service Provider metadata XML.
    pub async fn metadata(
        &mut self,
    ) -> Result<quark_auth_proto::auth::v1::SamlMetadataResponse, AuthClientError> {
        let resp = self.inner.saml_metadata(()).await?;
        Ok(resp.into_inner())
    }
}
