//! AdminService — admin-only operations.
//!
//! Wraps `quark_auth_proto::auth::v1::admin_service_client::AdminServiceClient`.
//!
//! Covers all 28 RPCs: user management (`AdminListUsers`, `AdminCreateUser`,
//! `AdminGetUser`, `AdminUpdateUser`, `AdminDeleteUser`, `AdminGenerateLink`),
//! factor management (`AdminListUserFactors`, `AdminDeleteFactor`,
//! `AdminUpdateFactor`), passkey management (`AdminListPasskeys`,
//! `AdminDeletePasskey`), audit log (`AdminListAuditLogs`), SSO provider
//! management (`AdminListSSOProviders`, `AdminCreateSSOProvider`,
//! `AdminGetSSOProvider`, `AdminUpdateSSOProvider`, `AdminDeleteSSOProvider`),
//! OAuth client management (`AdminOAuthClientRegister`, `AdminOAuthClientList`,
//! `AdminOAuthClientGet`, `AdminOAuthClientUpdate`, `AdminOAuthClientDelete`,
//! `AdminOAuthClientRegenerateSecret`), and custom OAuth provider management
//! (`AdminListCustomOAuthProviders`, `AdminCreateCustomOAuthProvider`,
//! `AdminGetCustomOAuthProvider`, `AdminUpdateCustomOAuthProvider`,
//! `AdminDeleteCustomOAuthProvider`).
//!
//! Every RPC requires an admin bearer token.

use quark_auth_proto::auth::v1::admin_service_client::AdminServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `AdminService`.
pub struct AdminService {
    inner: AdminServiceClient<Channel>,
}

impl AdminService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: AdminServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut AdminServiceClient<Channel> {
        &mut self.inner
    }

    // ─── user management ─────────────────────────────────────────────────────

    /// `AdminListUsers` — paginated user list with optional sort + free-text filter.
    pub async fn list_users(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
        sort_by: &str,
        filter: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListUsersResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminListUsersRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
            sort_by: sort_by.to_string(),
            filter: filter.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_users(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminCreateUser` — create a user with admin-specified attributes.
    pub async fn create_user(
        &mut self,
        token: &str,
        email: &str,
        phone: &str,
        password: &str,
        role: &str,
        data: &str,
        app_metadata: &str,
        email_confirm: bool,
        phone_confirm: bool,
        ban_duration: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminCreateUserRequest {
            email: email.to_string(),
            phone: phone.to_string(),
            password: password.to_string(),
            role: role.to_string(),
            data: data.to_string(),
            app_metadata: app_metadata.to_string(),
            email_confirm,
            phone_confirm,
            ban_duration: ban_duration.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_create_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminGetUser` — fetch a user by ID.
    pub async fn get_user(
        &mut self,
        token: &str,
        user_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminGetUserRequest {
            user_id: user_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_get_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminUpdateUser` — patch a user. Pass `None` for fields you don't want to change.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_user(
        &mut self,
        token: &str,
        user_id: &str,
        email: Option<&str>,
        phone: Option<&str>,
        password: Option<&str>,
        role: Option<&str>,
        data: Option<&str>,
        app_metadata: Option<&str>,
        email_confirm: Option<bool>,
        phone_confirm: Option<bool>,
        ban_duration: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::User, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminUpdateUserRequest {
            user_id: user_id.to_string(),
            email: email.map(|s| s.to_string()),
            phone: phone.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            role: role.map(|s| s.to_string()),
            data: data.map(|s| s.to_string()),
            app_metadata: app_metadata.map(|s| s.to_string()),
            email_confirm,
            phone_confirm,
            ban_duration: ban_duration.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_update_user(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminDeleteUser` — delete a user. `should_soft_delete=true` marks the
    /// user deleted (sets `deleted_at`); `false` hard-deletes the row.
    pub async fn delete_user(
        &mut self,
        token: &str,
        user_id: &str,
        should_soft_delete: bool,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminDeleteUserRequest {
            user_id: user_id.to_string(),
            should_soft_delete,
        });
        attach_bearer(&mut req, token);
        self.inner.admin_delete_user(req).await?;
        Ok(())
    }

    /// `AdminGenerateLink` — generate a one-time action link (signup, magiclink,
    /// recovery, invite, email change). `link_type` is one of `signup`,
    /// `magiclink`, `recovery`, `invite`, `email_change_current`,
    /// `email_change_new`.
    pub async fn generate_link(
        &mut self,
        token: &str,
        link_type: &str,
        email: &str,
        password: &str,
        data: &str,
        redirect_to: &str,
    ) -> Result<quark_auth_proto::auth::v1::AdminGenerateLinkResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminGenerateLinkRequest {
            r#type: link_type.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            data: data.to_string(),
            redirect_to: redirect_to.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_generate_link(req).await?;
        Ok(resp.into_inner())
    }

    // ─── factor management ───────────────────────────────────────────────────

    /// `AdminListUserFactors` — list a user's MFA factors.
    pub async fn list_user_factors(
        &mut self,
        token: &str,
        user_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListFactorsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminListUserFactorsRequest {
            user_id: user_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_user_factors(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminDeleteFactor` — delete a user's MFA factor.
    pub async fn delete_factor(
        &mut self,
        token: &str,
        user_id: &str,
        factor_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminDeleteFactorRequest {
            user_id: user_id.to_string(),
            factor_id: factor_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.admin_delete_factor(req).await?;
        Ok(())
    }

    /// `AdminUpdateFactor` — update a user's MFA factor (force-verify, rename).
    pub async fn update_factor(
        &mut self,
        token: &str,
        user_id: &str,
        factor_id: &str,
        force_verified: bool,
        friendly_name: &str,
    ) -> Result<quark_auth_proto::auth::v1::Factor, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminUpdateFactorRequest {
            user_id: user_id.to_string(),
            factor_id: factor_id.to_string(),
            force_verified,
            friendly_name: friendly_name.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_update_factor(req).await?;
        Ok(resp.into_inner())
    }

    // ─── passkey management ──────────────────────────────────────────────────

    /// `AdminListPasskeys` — list a user's passkeys.
    pub async fn list_passkeys(
        &mut self,
        token: &str,
        user_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListPasskeysResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminListPasskeysRequest {
            user_id: user_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_passkeys(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminDeletePasskey` — delete a user's passkey.
    pub async fn delete_passkey(
        &mut self,
        token: &str,
        user_id: &str,
        passkey_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminDeletePasskeyRequest {
            user_id: user_id.to_string(),
            passkey_id: passkey_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.admin_delete_passkey(req).await?;
        Ok(())
    }

    // ─── audit log ───────────────────────────────────────────────────────────

    /// `AdminListAuditLogs` — paginated audit-log entries.
    pub async fn list_audit_logs(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
    ) -> Result<quark_auth_proto::auth::v1::ListAuditLogsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminListAuditLogsRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_audit_logs(req).await?;
        Ok(resp.into_inner())
    }

    // ─── SSO provider management ─────────────────────────────────────────────

    /// `AdminListSSOProviders` — list all configured SSO providers.
    pub async fn list_sso_providers(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListSsoProvidersResponse, AuthClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_sso_providers(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminCreateSSOProvider` — register a new SAML SSO provider.
    pub async fn create_sso_provider(
        &mut self,
        token: &str,
        name: &str,
        sso_provider_type: &str,
        metadata_url: &str,
        metadata_xml: &str,
        attribute_mapping: &str,
        domains: &[&str],
    ) -> Result<quark_auth_proto::auth::v1::SsoProvider, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminCreateSsoProviderRequest {
            name: name.to_string(),
            sso_provider_type: sso_provider_type.to_string(),
            metadata_url: metadata_url.to_string(),
            metadata_xml: metadata_xml.to_string(),
            attribute_mapping: attribute_mapping.to_string(),
            domains: domains.iter().map(|s| s.to_string()).collect(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_create_sso_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminGetSSOProvider` — fetch an SSO provider by ID.
    pub async fn get_sso_provider(
        &mut self,
        token: &str,
        idp_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::SsoProvider, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminGetSsoProviderRequest {
            idp_id: idp_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_get_sso_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminUpdateSSOProvider` — patch an SSO provider. `domains` replaces the
    /// existing domain list (it's a `repeated` field, not optional).
    pub async fn update_sso_provider(
        &mut self,
        token: &str,
        idp_id: &str,
        name: Option<&str>,
        metadata_url: Option<&str>,
        metadata_xml: Option<&str>,
        attribute_mapping: Option<&str>,
        domains: &[&str],
    ) -> Result<quark_auth_proto::auth::v1::SsoProvider, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminUpdateSsoProviderRequest {
            idp_id: idp_id.to_string(),
            name: name.map(|s| s.to_string()),
            metadata_url: metadata_url.map(|s| s.to_string()),
            metadata_xml: metadata_xml.map(|s| s.to_string()),
            attribute_mapping: attribute_mapping.map(|s| s.to_string()),
            domains: domains.iter().map(|s| s.to_string()).collect(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_update_sso_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminDeleteSSOProvider` — delete an SSO provider by ID.
    pub async fn delete_sso_provider(
        &mut self,
        token: &str,
        idp_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminDeleteSsoProviderRequest {
            idp_id: idp_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.admin_delete_sso_provider(req).await?;
        Ok(())
    }

    // ─── OAuth client management ─────────────────────────────────────────────

    /// `AdminOAuthClientRegister` — register a first-party OAuth client.
    pub async fn oauth_client_register(
        &mut self,
        token: &str,
        name: &str,
        redirect_uris: &[&str],
        scopes: &[&str],
    ) -> Result<quark_auth_proto::auth::v1::OAuthClient, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminOAuthClientRegisterRequest {
            name: name.to_string(),
            redirect_uris: redirect_uris.iter().map(|s| s.to_string()).collect(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_o_auth_client_register(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminOAuthClientList` — list all registered OAuth clients.
    pub async fn oauth_client_list(
        &mut self,
        token: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListOAuthClientsResponse, AuthClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_o_auth_client_list(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminOAuthClientGet` — fetch an OAuth client by ID.
    pub async fn oauth_client_get(
        &mut self,
        token: &str,
        client_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::OAuthClient, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminOAuthClientGetRequest {
            client_id: client_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_o_auth_client_get(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminOAuthClientUpdate` — patch an OAuth client. `redirect_uris` and
    /// `scopes` replace the existing lists (`repeated` fields).
    pub async fn oauth_client_update(
        &mut self,
        token: &str,
        client_id: &str,
        name: Option<&str>,
        redirect_uris: &[&str],
        scopes: &[&str],
    ) -> Result<quark_auth_proto::auth::v1::OAuthClient, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminOAuthClientUpdateRequest {
            client_id: client_id.to_string(),
            name: name.map(|s| s.to_string()),
            redirect_uris: redirect_uris.iter().map(|s| s.to_string()).collect(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_o_auth_client_update(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminOAuthClientDelete` — delete an OAuth client by ID.
    pub async fn oauth_client_delete(
        &mut self,
        token: &str,
        client_id: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::AdminOAuthClientDeleteRequest {
            client_id: client_id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.admin_o_auth_client_delete(req).await?;
        Ok(())
    }

    /// `AdminOAuthClientRegenerateSecret` — rotate an OAuth client's secret.
    pub async fn oauth_client_regenerate_secret(
        &mut self,
        token: &str,
        client_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::OAuthClient, AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminOAuthClientRegenerateSecretRequest {
                client_id: client_id.to_string(),
            },
        );
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_o_auth_client_regenerate_secret(req).await?;
        Ok(resp.into_inner())
    }

    // ─── custom OAuth provider management ────────────────────────────────────

    /// `AdminListCustomOAuthProviders` — list custom OAuth/OIDC providers.
    /// `provider_type` is `"oauth2"` or `"oidc"` (pass `""` for all).
    pub async fn list_custom_oauth_providers(
        &mut self,
        token: &str,
        provider_type: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListCustomOAuthProvidersResponse, AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminListCustomOAuthProvidersRequest {
                provider_type: provider_type.to_string(),
            },
        );
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_list_custom_o_auth_providers(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminCreateCustomOAuthProvider` — register a custom OAuth/OIDC provider.
    pub async fn create_custom_oauth_provider(
        &mut self,
        token: &str,
        identifier: &str,
        provider_type: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        issuer: &str,
        scopes: &str,
        enabled: bool,
    ) -> Result<quark_auth_proto::auth::v1::CustomOAuthProvider, AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminCreateCustomOAuthProviderRequest {
                identifier: identifier.to_string(),
                provider_type: provider_type.to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                redirect_uri: redirect_uri.to_string(),
                issuer: issuer.to_string(),
                scopes: scopes.to_string(),
                enabled,
            },
        );
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_create_custom_o_auth_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminGetCustomOAuthProvider` — fetch a custom OAuth provider by identifier.
    pub async fn get_custom_oauth_provider(
        &mut self,
        token: &str,
        identifier: &str,
    ) -> Result<quark_auth_proto::auth::v1::CustomOAuthProvider, AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminGetCustomOAuthProviderRequest {
                identifier: identifier.to_string(),
            },
        );
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_get_custom_o_auth_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminUpdateCustomOAuthProvider` — patch a custom OAuth provider.
    pub async fn update_custom_oauth_provider(
        &mut self,
        token: &str,
        identifier: &str,
        client_id: Option<&str>,
        client_secret: Option<&str>,
        redirect_uri: Option<&str>,
        issuer: Option<&str>,
        scopes: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<quark_auth_proto::auth::v1::CustomOAuthProvider, AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminUpdateCustomOAuthProviderRequest {
                identifier: identifier.to_string(),
                client_id: client_id.map(|s| s.to_string()),
                client_secret: client_secret.map(|s| s.to_string()),
                redirect_uri: redirect_uri.map(|s| s.to_string()),
                issuer: issuer.map(|s| s.to_string()),
                scopes: scopes.map(|s| s.to_string()),
                enabled,
            },
        );
        attach_bearer(&mut req, token);
        let resp = self.inner.admin_update_custom_o_auth_provider(req).await?;
        Ok(resp.into_inner())
    }

    /// `AdminDeleteCustomOAuthProvider` — delete a custom OAuth provider by identifier.
    pub async fn delete_custom_oauth_provider(
        &mut self,
        token: &str,
        identifier: &str,
    ) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(
            quark_auth_proto::auth::v1::AdminDeleteCustomOAuthProviderRequest {
                identifier: identifier.to_string(),
            },
        );
        attach_bearer(&mut req, token);
        self.inner.admin_delete_custom_o_auth_provider(req).await?;
        Ok(())
    }
}
