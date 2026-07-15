//! Ergonomic gRPC client SDK for the auth-service.
//!
//! `auth-client` is the primary client SDK for talking to the auth-service over
//! gRPC. It wraps the generated tonic clients (from `quark_auth_proto::auth::v1`) with
//! a Supabase-style builder pattern and typed convenience methods covering
//! **all 13 services** and **all 115 RPCs** defined in `proto/auth.proto`.
//!
//! # Quick start
//!
//! ```no_run
//! use std::time::Duration;
//! use auth_client::AuthClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = AuthClient::builder()
//!     .endpoint("http://127.0.0.1:5001")
//!     .connect_timeout(Duration::from_secs(5))
//!     .request_timeout(Duration::from_secs(30))
//!     .build()
//!     .await?;
//!
//! // Public auth flow — no bearer token required.
//! let login = client.auth().login("alice", "secret-api-key").await?;
//! println!("access token: {}", login.access_token);
//!
//! // Authenticated call — pass the access token.
//! let me = client.users().get(&login.access_token, "user-uuid").await?;
//! println!("hello, {}", me.email);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Service accessors
//!
//! Every proto service has a dedicated accessor on [`AuthClient`] that returns
//! a cheap, owned service client built from a cloned [`tonic::transport::Channel`]:
//!
//! | Accessor              | Service client        | Service            |
//! |-----------------------|-----------------------|--------------------|
//! | [`AuthClient::auth`]  | [`services::AuthService`]    | AuthService    |
//! | [`AuthClient::users`] | [`services::UserService`]   | UserService     |
//! | ...                   | ...                   | ...                |
//!
//! Service clients are created on demand and cheap to clone — the underlying
//! gRPC channel is multiplexed (HTTP/2) and shared.

// The SDK deliberately maps every proto field of every RPC to a typed Rust
// parameter (no simplification), so several service methods exceed clippy's
// 7-argument threshold. Boxing every such call site behind a params struct
// would obscure the 1:1 mapping with the proto contract, so we allow the lint
// crate-wide.
#![allow(clippy::too_many_arguments)]
// `AuthClientError` holds a `tonic::Status` (~176 bytes) so its `Result`s trip
// `result_large_err`. This matches the precedent set by the sibling
// `auth-admin-client` crate and is inherent to surfacing full gRPC status
// details to callers.
#![allow(clippy::result_large_err)]

pub mod error;
pub mod services;

pub use error::AuthClientError;

use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

// Re-export the generated proto modules so callers don't need to depend on
// `proto-gen` directly to name request/response types.
pub use quark_auth_proto::auth::v1 as proto;
pub use quark_auth_proto::common::v1 as common;

pub use services::{
    admin::AdminService, auth::AuthService, identity::IdentityService, mfa::MfaService,
    oauth_server::OAuthServerService, organization::OrganizationService, passkey::PasskeyService,
    policy::PolicyService, project::ProjectService, role::RoleService, sso::SsoService,
    user::UserService, workspace::WorkspaceService,
};

/// Snapshot of the configuration used to build an [`AuthClient`].
///
/// Captured at build time for introspection / logging. Timeouts are applied to
/// the underlying tonic [`Endpoint`] / [`Channel`] and live there for the real
/// enforcement — this struct is a readable record.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The endpoint URL the client is connected to (e.g. `http://127.0.0.1:5001`).
    pub endpoint: String,
    /// Optional TCP-connect timeout applied to the channel.
    pub connect_timeout: Option<Duration>,
    /// Optional per-request timeout applied to every RPC issued through the channel.
    pub request_timeout: Option<Duration>,
    /// Optional TCP keepalive timeout.
    pub keepalive_timeout: Option<Duration>,
    /// Optional HTTP/2 keepalive interval.
    pub keepalive_interval: Option<Duration>,
}

impl ClientConfig {
    /// An empty config — used when constructing an [`AuthClient`] directly from
    /// an existing channel via [`AuthClient::from_channel`].
    pub fn empty() -> Self {
        Self {
            endpoint: String::new(),
            connect_timeout: None,
            request_timeout: None,
            keepalive_timeout: None,
            keepalive_interval: None,
        }
    }
}

/// Top-level auth-service client.
///
/// Holds a single multiplexed gRPC [`Channel`] shared by every service client.
/// Each accessor (`auth()`, `users()`, …) clones the channel (cheap) and wraps
/// the generated tonic client.
///
/// Create one with [`AuthClient::builder`] (recommended) or
/// [`AuthClient::connect`], or wrap an existing channel with
/// [`AuthClient::from_channel`].
#[derive(Clone)]
pub struct AuthClient {
    channel: Channel,
    config: ClientConfig,
}

impl std::fmt::Debug for AuthClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthClient")
            .field("endpoint", &self.config.endpoint)
            .finish()
    }
}

impl AuthClient {
    /// Start a builder for configuring a new client.
    pub fn builder() -> AuthClientBuilder {
        AuthClientBuilder::default()
    }

    /// Convenience: connect to `endpoint` with default timeouts.
    pub async fn connect(endpoint: &str) -> Result<Self, AuthClientError> {
        Self::builder().endpoint(endpoint).build().await
    }

    /// Wrap an already-connected tonic channel. Useful for sharing a channel
    /// with other clients or for custom transport setups (TLS, middleware, …).
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            channel,
            config: ClientConfig::empty(),
        }
    }

    /// Clone of the underlying gRPC channel.
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// The configuration this client was built with.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    // ─── service accessors ───────────────────────────────────────────────────

    /// AuthService — authentication entry points (login, signup, token, verify, …).
    pub fn auth(&self) -> AuthService {
        AuthService::new(self.channel.clone())
    }

    /// UserService — user CRUD + role assignment.
    pub fn users(&self) -> UserService {
        UserService::new(self.channel.clone())
    }

    /// IdentityService — OAuth identity management for the authenticated user.
    pub fn identities(&self) -> IdentityService {
        IdentityService::new(self.channel.clone())
    }

    /// MFAService — multi-factor authentication factors (TOTP, phone, WebAuthn).
    pub fn mfa(&self) -> MfaService {
        MfaService::new(self.channel.clone())
    }

    /// PasskeyService — WebAuthn passkey authentication + management.
    pub fn passkeys(&self) -> PasskeyService {
        PasskeyService::new(self.channel.clone())
    }

    /// SSOService — SAML SSO entry points.
    pub fn sso(&self) -> SsoService {
        SsoService::new(self.channel.clone())
    }

    /// OAuthServerService — auth-service acting as an OAuth2/OIDC provider.
    pub fn oauth_server(&self) -> OAuthServerService {
        OAuthServerService::new(self.channel.clone())
    }

    /// AdminService — admin-only operations (user/factor/passkey/SSO/OAuth-client management, audit logs).
    pub fn admin(&self) -> AdminService {
        AdminService::new(self.channel.clone())
    }

    /// OrganizationService — organizations CRUD + lifecycle.
    pub fn organizations(&self) -> OrganizationService {
        OrganizationService::new(self.channel.clone())
    }

    /// ProjectService — projects CRUD + lifecycle (org-scoped).
    pub fn projects(&self) -> ProjectService {
        ProjectService::new(self.channel.clone())
    }

    /// WorkspaceService — workspaces CRUD + lifecycle (project-scoped).
    pub fn workspaces(&self) -> WorkspaceService {
        WorkspaceService::new(self.channel.clone())
    }

    /// RoleService — roles CRUD + permission grants (org-scoped).
    pub fn roles(&self) -> RoleService {
        RoleService::new(self.channel.clone())
    }

    /// PolicyService — RBAC policies CRUD.
    pub fn policies(&self) -> PolicyService {
        PolicyService::new(self.channel.clone())
    }
}

/// Builder for [`AuthClient`].
///
/// Fluent, consumes-and-returns-self. Finalize with [`AuthClientBuilder::build`]
/// (async, connects eagerly) or [`AuthClientBuilder::build_lazy`] (synchronous,
/// connects on first RPC).
///
/// ```no_run
/// use std::time::Duration;
/// use auth_client::AuthClient;
/// # async fn t() -> Result<(), Box<dyn std::error::Error>> {
/// let client = AuthClient::builder()
///     .endpoint("http://127.0.0.1:5001")
///     .connect_timeout(Duration::from_secs(5))
///     .request_timeout(Duration::from_secs(30))
///     .keepalive_interval(Duration::from_secs(30))
///     .tcp_nodelay(true)
///     .build()
///     .await?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Default)]
pub struct AuthClientBuilder {
    endpoint: Option<String>,
    connect_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
    keepalive_timeout: Option<Duration>,
    keepalive_interval: Option<Duration>,
    http2_keepalive_interval: Option<Duration>,
    tcp_nodelay: Option<bool>,
    tcp_keepalive: Option<Duration>,
    concurrency_limit: Option<usize>,
}

impl AuthClientBuilder {
    /// Set the auth-service endpoint URL (e.g. `http://127.0.0.1:5001` or
    /// `https://auth.example.com`). Required.
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Timeout for the initial TCP connect (and TLS handshake).
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Per-request timeout applied to every RPC.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Timeout for the TCP keepalive.
    pub fn keepalive_timeout(mut self, timeout: Duration) -> Self {
        self.keepalive_timeout = Some(timeout);
        self
    }

    /// Interval for HTTP/2 PING frames ("connection keepalive").
    pub fn keepalive_interval(mut self, interval: Duration) -> Self {
        self.keepalive_interval = Some(interval);
        self
    }

    /// Alias for [`Self::keepalive_interval`] — maps to tonic's
    /// `http2_keepalive_interval`.
    pub fn http2_keepalive_interval(mut self, interval: Duration) -> Self {
        self.http2_keepalive_interval = Some(interval);
        self
    }

    /// Set the `TCP_NODELAY` flag on the socket (disables Nagle's algorithm).
    pub fn tcp_nodelay(mut self, enabled: bool) -> Self {
        self.tcp_nodelay = Some(enabled);
        self
    }

    /// Set the OS-level TCP keepalive duration.
    pub fn tcp_keepalive(mut self, interval: Duration) -> Self {
        self.tcp_keepalive = Some(interval);
        self
    }

    /// Cap the number of in-flight concurrent requests per channel.
    pub fn concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = Some(limit);
        self
    }

    /// Build and eagerly connect the channel.
    pub async fn build(self) -> Result<AuthClient, AuthClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| AuthClientError::Transport(e.to_string()))?;
        Ok(AuthClient {
            channel,
            config: self.snapshot(),
        })
    }

    /// Build a lazily-connected channel — the TCP connection is established on
    /// the first RPC. Returns synchronously.
    pub fn build_lazy(self) -> Result<AuthClient, AuthClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint.connect_lazy();
        Ok(AuthClient {
            channel,
            config: self.snapshot(),
        })
    }

    fn build_endpoint(&self) -> Result<Endpoint, AuthClientError> {
        let url = self
            .endpoint
            .as_ref()
            .ok_or_else(|| AuthClientError::Transport("no endpoint configured".into()))?;
        let mut endpoint = Endpoint::from_shared(url.clone())
            .map_err(|e| AuthClientError::Transport(e.to_string()))?;
        if let Some(d) = self.connect_timeout {
            endpoint = endpoint.connect_timeout(d);
        }
        if let Some(d) = self.request_timeout {
            endpoint = endpoint.timeout(d);
        }
        if let Some(d) = self.keepalive_timeout {
            endpoint = endpoint.keep_alive_timeout(d);
        }
        // `keepalive_interval` and `http2_keepalive_interval` are aliases that
        // both map to tonic's `http2_keep_alive_interval`; apply whichever was
        // set (explicit http2 name wins if both supplied).
        if let Some(d) = self.http2_keepalive_interval.or(self.keepalive_interval) {
            endpoint = endpoint.http2_keep_alive_interval(d);
        }
        if let Some(b) = self.tcp_nodelay {
            endpoint = endpoint.tcp_nodelay(b);
        }
        if let Some(d) = self.tcp_keepalive {
            endpoint = endpoint.tcp_keepalive(Some(d));
        }
        if let Some(n) = self.concurrency_limit {
            endpoint = endpoint.concurrency_limit(n);
        }
        Ok(endpoint)
    }

    fn snapshot(&self) -> ClientConfig {
        ClientConfig {
            endpoint: self.endpoint.clone().unwrap_or_default(),
            connect_timeout: self.connect_timeout,
            request_timeout: self.request_timeout,
            keepalive_timeout: self.keepalive_timeout,
            keepalive_interval: self.keepalive_interval.or(self.http2_keepalive_interval),
        }
    }
}
