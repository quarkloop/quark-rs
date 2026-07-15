//! MFAService — multi-factor authentication factors.
//!
//! Wraps `quark_auth_proto::auth::v1::mfa_service_client::MfaServiceClient`.
//!
//! Covers all 5 RPCs: `EnrollFactor`, `ChallengeFactor`, `VerifyFactor`,
//! `UnenrollFactor`, `ListFactors`. Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::mfa_service_client::MfaServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `MFAService`.
pub struct MfaService {
    inner: MfaServiceClient<Channel>,
}

impl MfaService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: MfaServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut MfaServiceClient<Channel> {
        &mut self.inner
    }

    /// `EnrollFactor` — begin enrolling a new MFA factor.
    /// `factor_type` is one of `"totp"`, `"phone"`, `"webauthn"`.
    pub async fn enroll(
        &mut self,
        token: &str,
        friendly_name: &str,
        factor_type: &str,
        issuer: &str,
        phone: &str,
    ) -> Result<quark_auth_proto::auth::v1::EnrollFactorResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::EnrollFactorRequest {
            friendly_name: friendly_name.to_string(),
            factor_type: factor_type.to_string(),
            issuer: issuer.to_string(),
            phone: phone.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.enroll_factor(req).await?;
        Ok(resp.into_inner())
    }

    /// `ChallengeFactor` — issue a challenge for a factor (e.g. SMS OTP).
    /// `channel` is `"sms"` or `"whatsapp"`.
    pub async fn challenge(
        &mut self,
        token: &str,
        factor_id: &str,
        channel: &str,
    ) -> Result<quark_auth_proto::auth::v1::ChallengeFactorResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ChallengeFactorRequest {
            factor_id: factor_id.to_string(),
            channel: channel.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.challenge_factor(req).await?;
        Ok(resp.into_inner())
    }

    /// `VerifyFactor` — verify a challenge with the user-supplied code.
    pub async fn verify(
        &mut self,
        token: &str,
        factor_id: &str,
        challenge_id: &str,
        code: &str,
    ) -> Result<quark_auth_proto::auth::v1::Factor, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::VerifyFactorRequest {
            factor_id: factor_id.to_string(),
            challenge_id: challenge_id.to_string(),
            code: code.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.verify_factor(req).await?;
        Ok(resp.into_inner())
    }

    /// `UnenrollFactor` — remove an MFA factor.
    pub async fn unenroll(
        &mut self,
        token: &str,
        factor_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Factor, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UnenrollFactorRequest {
            factor_id: factor_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.unenroll_factor(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListFactors` — list the authenticated user's MFA factors.
    pub async fn list(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListFactorsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListFactorsRequest {});
        attach_bearer(&mut req, token);
        let resp = self.inner.list_factors(req).await?;
        Ok(resp.into_inner())
    }
}
