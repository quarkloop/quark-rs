//! PasskeyService — WebAuthn passkey authentication + management.
//!
//! Wraps `quark_auth_proto::auth::v1::passkey_service_client::PasskeyServiceClient`.
//!
//! Covers all 7 RPCs: `PasskeyAuthenticationOptions`,
//! `PasskeyAuthenticationVerify`, `PasskeyRegistrationOptions`,
//! `PasskeyRegistrationVerify`, `ListPasskeys`, `UpdatePasskey`, `DeletePasskey`.
//!
//! Anonymous (no bearer token) RPCs: `PasskeyAuthenticationOptions`,
//! `PasskeyAuthenticationVerify`.
//!
//! Authenticated (bearer token required) RPCs: `PasskeyRegistrationOptions`,
//! `PasskeyRegistrationVerify`, `ListPasskeys`, `UpdatePasskey`, `DeletePasskey`.

use quark_auth_proto::auth::v1::passkey_service_client::PasskeyServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `PasskeyService`.
pub struct PasskeyService {
    inner: PasskeyServiceClient<Channel>,
}

impl PasskeyService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: PasskeyServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut PasskeyServiceClient<Channel> {
        &mut self.inner
    }

    // ─── anonymous (login flow) ──────────────────────────────────────────────

    /// `PasskeyAuthenticationOptions` — get WebAuthn assertion options for login.
    /// `email` is optional; if set, the server returns allowed credentials for that user.
    pub async fn authentication_options(
        &mut self,
        email: &str,
    ) -> Result<quark_auth_proto::auth::v1::PasskeyAuthOptionsResponse, AuthClientError> {
        let resp = self
            .inner
            .passkey_authentication_options(quark_auth_proto::auth::v1::PasskeyAuthOptionsRequest {
                email: email.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `PasskeyAuthenticationVerify` — verify a WebAuthn assertion; returns tokens on success.
    pub async fn authentication_verify(
        &mut self,
        credential_response: &str,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let resp = self
            .inner
            .passkey_authentication_verify(quark_auth_proto::auth::v1::PasskeyAuthVerifyRequest {
                credential_response: credential_response.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    // ─── authenticated (management) ──────────────────────────────────────────

    /// `PasskeyRegistrationOptions` — get WebAuthn attestation options for the
    /// authenticated user. Requires a bearer token.
    pub async fn registration_options(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::PasskeyRegOptionsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::PasskeyOptionsRequest {});
        attach_bearer(&mut req, token);
        let resp = self.inner.passkey_registration_options(req).await?;
        Ok(resp.into_inner())
    }

    /// `PasskeyRegistrationVerify` — verify a WebAuthn attestation and register
    /// a new passkey. Requires a bearer token.
    pub async fn registration_verify(
        &mut self,
        token: &str,
        friendly_name: &str,
        credential_response: &str,
    ) -> Result<quark_auth_proto::auth::v1::Passkey, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::PasskeyRegistrationVerifyRequest {
            friendly_name: friendly_name.to_string(),
            credential_response: credential_response.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.passkey_registration_verify(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListPasskeys` — list the authenticated user's passkeys.
    pub async fn list(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListPasskeysResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListPasskeysRequest {});
        attach_bearer(&mut req, token);
        let resp = self.inner.list_passkeys(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdatePasskey` — rename a passkey.
    pub async fn update(
        &mut self,
        token: &str,
        passkey_id: &str,
        friendly_name: &str,
    ) -> Result<quark_auth_proto::auth::v1::Passkey, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdatePasskeyRequest {
            passkey_id: passkey_id.to_string(),
            friendly_name: friendly_name.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_passkey(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeletePasskey` — remove a passkey.
    pub async fn delete(&mut self, token: &str, passkey_id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeletePasskeyRequest {
            passkey_id: passkey_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_passkey(req).await?;
        Ok(())
    }
}
