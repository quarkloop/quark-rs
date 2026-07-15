//! AuthService — authentication entry points.
//!
//! Wraps `quark_auth_proto::auth::v1::auth_service_client::AuthServiceClient`.
//!
//! Covers all 19 RPCs of `AuthService`:
//! `Login`, `Refresh`, `Logout`, `VerifyToken`, `Signup`, `Token`, `Verify`,
//! `MagicLink`, `Otp`, `Recover`, `Resend`, `Reauthenticate`, `GetSettings`,
//! `GetJwks`, `GetOpenIdConfiguration`, `Health`, `ExternalProviderRedirect`,
//! `ExternalProviderCallback`, `Invite`.
//!
//! Anonymous (no bearer token) RPCs: `Login`, `Refresh`, `Logout`,
//! `VerifyToken`, `Signup`, `Token`, `Verify`, `MagicLink`, `Otp`, `Recover`,
//! `Resend`, `GetSettings`, `GetJwks`, `GetOpenIdConfiguration`, `Health`,
//! `ExternalProviderRedirect`, `ExternalProviderCallback`.
//!
//! Authenticated (bearer token required) RPCs: `Reauthenticate`, `Invite`.

use std::collections::HashMap;

use quark_auth_proto::auth::v1::auth_service_client::AuthServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `AuthService`.
pub struct AuthService {
    inner: AuthServiceClient<Channel>,
}

impl AuthService {
    /// Wrap a generated `AuthServiceClient` over a shared channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: AuthServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch for advanced use).
    pub fn inner(&mut self) -> &mut AuthServiceClient<Channel> {
        &mut self.inner
    }

    // ─── API-key auth ─────────────────────────────────────────────────────────

    /// `Login` — exchange a handle + API key for an access/refresh token pair.
    pub async fn login(
        &mut self,
        handle: &str,
        api_key: &str,
    ) -> Result<quark_auth_proto::auth::v1::LoginResponse, AuthClientError> {
        let resp = self
            .inner
            .login(quark_auth_proto::auth::v1::LoginRequest {
                handle: handle.to_string(),
                api_key: api_key.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Refresh` — rotate tokens using a refresh token.
    pub async fn refresh(
        &mut self,
        refresh_token: &str,
    ) -> Result<quark_auth_proto::auth::v1::LoginResponse, AuthClientError> {
        let resp = self
            .inner
            .refresh(quark_auth_proto::auth::v1::RefreshRequest {
                refresh_token: refresh_token.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Logout` — revoke a refresh token. `scope` is `"global"`, `"local"`, or
    /// `"others"`; pass `""` to let the server default to `"global"`.
    pub async fn logout(
        &mut self,
        refresh_token: &str,
        scope: &str,
    ) -> Result<(), AuthClientError> {
        self.inner
            .logout(quark_auth_proto::auth::v1::LogoutRequest {
                refresh_token: refresh_token.to_string(),
                scope: scope.to_string(),
            })
            .await?;
        Ok(())
    }

    /// `VerifyToken` — introspect an access token.
    pub async fn verify_token(
        &mut self,
        access_token: &str,
    ) -> Result<quark_auth_proto::auth::v1::VerifyTokenResponse, AuthClientError> {
        let resp = self
            .inner
            .verify_token(quark_auth_proto::auth::v1::VerifyTokenRequest {
                access_token: access_token.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    // ─── Supabase auth flows ─────────────────────────────────────────────────

    /// `Signup` — register a new user. `data` is user-metadata key/value pairs.
    pub async fn signup(
        &mut self,
        email: &str,
        phone: &str,
        password: &str,
        data: &[(&str, &str)],
        channel: &str,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<quark_auth_proto::auth::v1::SignupResponse, AuthClientError> {
        let resp = self
            .inner
            .signup(quark_auth_proto::auth::v1::SignupRequest {
                email: email.to_string(),
                phone: phone.to_string(),
                password: password.to_string(),
                data: data
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<_, _>>(),
                channel: channel.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Token` — OAuth2 token endpoint handling all grant types.
    ///
    /// Pass the fields relevant to your `grant_type`; unused fields should be
    /// empty strings.
    pub async fn token(
        &mut self,
        grant_type: &str,
        email: &str,
        password: &str,
        refresh_token: &str,
        otp: &str,
        phone: &str,
        signed_message: &str,
        signature: &str,
        code: &str,
        code_verifier: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let resp = self
            .inner
            .token(quark_auth_proto::auth::v1::TokenRequest {
                grant_type: grant_type.to_string(),
                email: email.to_string(),
                password: password.to_string(),
                refresh_token: refresh_token.to_string(),
                otp: otp.to_string(),
                phone: phone.to_string(),
                signed_message: signed_message.to_string(),
                signature: signature.to_string(),
                code: code.to_string(),
                code_verifier: code_verifier.to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Verify` — verify a token delivered out-of-band (signup, recovery,
    /// magiclink, email/phone change, reauthenticate, …).
    pub async fn verify(
        &mut self,
        verify_type: &str,
        token: &str,
        password: &str,
        phone: &str,
        code_verifier: &str,
        redirect_to: &str,
    ) -> Result<quark_auth_proto::auth::v1::VerifyResponse, AuthClientError> {
        let resp = self
            .inner
            .verify(quark_auth_proto::auth::v1::VerifyRequest {
                r#type: verify_type.to_string(),
                token: token.to_string(),
                password: password.to_string(),
                phone: phone.to_string(),
                code_verifier: code_verifier.to_string(),
                redirect_to: redirect_to.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `MagicLink` — email a magic link. `data` is user metadata.
    pub async fn magic_link(
        &mut self,
        email: &str,
        data: &[(&str, &str)],
        code_challenge: &str,
        code_challenge_method: &str,
        gotrue_meta_security: &str,
    ) -> Result<(), AuthClientError> {
        self.inner
            .magic_link(quark_auth_proto::auth::v1::MagicLinkRequest {
                email: email.to_string(),
                data: data
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<_, _>>(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
                gotrue_meta_security: gotrue_meta_security.to_string(),
            })
            .await?;
        Ok(())
    }

    /// `Otp` — send an OTP via email/SMS/WhatsApp.
    pub async fn otp(
        &mut self,
        email: &str,
        phone: &str,
        channel: &str,
        otp_type: &str,
        data: &str,
        gotrue_meta_security: &str,
        create_user: bool,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<(), AuthClientError> {
        self.inner
            .otp(quark_auth_proto::auth::v1::OtpRequest {
                email: email.to_string(),
                phone: phone.to_string(),
                channel: channel.to_string(),
                r#type: otp_type.to_string(),
                data: data.to_string(),
                gotrue_meta_security: gotrue_meta_security.to_string(),
                create_user,
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
            })
            .await?;
        Ok(())
    }

    /// `Recover` — send a password-recovery link.
    pub async fn recover(
        &mut self,
        email: &str,
        code_challenge: &str,
        code_challenge_method: &str,
        gotrue_meta_security: &str,
    ) -> Result<(), AuthClientError> {
        self.inner
            .recover(quark_auth_proto::auth::v1::RecoverRequest {
                email: email.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
                gotrue_meta_security: gotrue_meta_security.to_string(),
            })
            .await?;
        Ok(())
    }

    /// `Resend` — resend a signup/SMS/email-change/phone-change OTP.
    pub async fn resend(
        &mut self,
        email: &str,
        phone: &str,
        resend_type: &str,
        gotrue_meta_security: &str,
        code_challenge: &str,
        code_challenge_method: &str,
    ) -> Result<quark_auth_proto::auth::v1::ResendResponse, AuthClientError> {
        let resp = self
            .inner
            .resend(quark_auth_proto::auth::v1::ResendRequest {
                email: email.to_string(),
                phone: phone.to_string(),
                r#type: resend_type.to_string(),
                gotrue_meta_security: gotrue_meta_security.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Reauthenticate` — trigger a reauthentication flow (sends an email).
    /// Requires a bearer token.
    pub async fn reauthenticate(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ReauthenticateResponse, AuthClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.reauthenticate(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetSettings` — public auth-service settings (enabled providers, MFA, …).
    pub async fn get_settings(
        &mut self,
    ) -> Result<quark_auth_proto::auth::v1::Settings, AuthClientError> {
        let resp = self.inner.get_settings(()).await?;
        Ok(resp.into_inner())
    }

    /// `GetJwks` — JSON Web Key Set for verifying JWTs.
    pub async fn get_jwks(&mut self) -> Result<quark_auth_proto::auth::v1::JwkSet, AuthClientError> {
        let resp = self.inner.get_jwks(()).await?;
        Ok(resp.into_inner())
    }

    /// `GetOpenIdConfiguration` — OIDC discovery document.
    pub async fn get_open_id_configuration(
        &mut self,
    ) -> Result<quark_auth_proto::auth::v1::OpenIdConfiguration, AuthClientError> {
        let resp = self.inner.get_open_id_configuration(()).await?;
        Ok(resp.into_inner())
    }

    /// `Health` — service health check.
    pub async fn health(&mut self) -> Result<quark_auth_proto::auth::v1::HealthStatus, AuthClientError> {
        let resp = self.inner.health(()).await?;
        Ok(resp.into_inner())
    }

    // ─── External OAuth ──────────────────────────────────────────────────────

    /// `ExternalProviderRedirect` — get the redirect URL to start an external
    /// OAuth flow (Google, GitHub, …).
    pub async fn external_provider_redirect(
        &mut self,
        provider: &str,
        scopes: &str,
        code_challenge: &str,
        code_challenge_method: &str,
        redirect_to: &str,
        invite_token: &str,
    ) -> Result<quark_auth_proto::auth::v1::RedirectResponse, AuthClientError> {
        let resp = self
            .inner
            .external_provider_redirect(quark_auth_proto::auth::v1::ExternalProviderRedirectRequest {
                provider: provider.to_string(),
                scopes: scopes.to_string(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: code_challenge_method.to_string(),
                redirect_to: redirect_to.to_string(),
                invite_token: invite_token.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `ExternalProviderCallback` — exchange an OAuth callback code/state for tokens.
    pub async fn external_provider_callback(
        &mut self,
        code: &str,
        state: &str,
        provider: &str,
    ) -> Result<quark_auth_proto::auth::v1::TokenResponse, AuthClientError> {
        let resp = self
            .inner
            .external_provider_callback(quark_auth_proto::auth::v1::ExternalProviderCallbackRequest {
                code: code.to_string(),
                state: state.to_string(),
                provider: provider.to_string(),
            })
            .await?;
        Ok(resp.into_inner())
    }

    /// `Invite` — invite a user by email. `data` is a JSON string of user
    /// metadata. Requires a bearer token.
    pub async fn invite(
        &mut self,
        token: &str,
        email: &str,
        data: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::InviteRequest {
            email: email.to_string(),
            data: data.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.invite(req).await?;
        Ok(resp.into_inner())
    }
}
