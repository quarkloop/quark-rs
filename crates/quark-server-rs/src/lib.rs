//! Ergonomic gRPC client SDK for the server server.
//!
//! `server-client` is the primary client SDK for talking to the server
//! server over gRPC. It wraps the generated tonic client (from
//! `quark_server_proto::server::v1`) with a Supabase-style builder pattern and
//! typed convenience methods covering **all 8 RPCs** of `ServerService`
//! defined in [`proto/server.proto`](../proto/server.proto).
//!
//! The server is *not* a gateway: client CRUD traffic never flows
//! through it. It exposes only orchestration (deploy, rollback, provision),
//! service-registry lookup, and admin/operator RPCs. Every RPC is gated by the
//! server-side `AuthInterceptor`, so every method on the SDK takes a `token:
//! &str` first argument and attaches it as `Authorization: Bearer …` metadata.
//!
//! # Quick start
//!
//! ```no_run
//! use std::time::Duration;
//! use quark_server_rs::ServerClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = ServerClient::builder()
//!     .endpoint("http://127.0.0.1:5000")
//!     .connect_timeout(Duration::from_secs(5))
//!     .request_timeout(Duration::from_secs(30))
//!     .build()
//!     .await?;
//!
//! // Fetch the service registry (every RPC requires a bearer token).
//! let registry = client.get_service_registry("admin-token").await?;
//! for svc in &registry.services {
//!     println!("{} -> {} ({})", svc.name, svc.grpc_url, svc.version);
//! }
//!
//! // Provision a new tenant.
//! let tenant = client
//!     .provision_tenant("admin-token", "Acme", "acme", "default")
//!     .await?;
//! println!("provisioned tenant: {}", tenant.id);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Service accessors
//!
//! Every proto service has a dedicated accessor on [`ServerClient`] that
//! returns a cheap, owned service client built from a cloned
//! [`tonic::transport::Channel`]:
//!
//! | Accessor                       | Service client              | Service              |
//! |--------------------------------|-----------------------------|----------------------|
//! | [`ServerClient::server`]| [`services::ServerService`] | ServerService |
//!
//! Service clients are created on demand and cheap to clone — the underlying
//! gRPC channel is multiplexed (HTTP/2) and shared.

// The SDK deliberately maps every proto field of every RPC to a typed Rust
// parameter (no simplification), so several service methods exceed clippy's
// 7-argument threshold. Boxing every such call site behind a params struct
// would obscure the 1:1 mapping with the proto contract, so we allow the lint
// crate-wide.
#![allow(clippy::too_many_arguments)]
// `ServerClientError` holds a `tonic::Status` (~176 bytes) so its `Result`s
// trip `result_large_err`. This matches the precedent set by the sibling
// `auth-client` crate and is inherent to surfacing full gRPC status details to
// callers.
#![allow(clippy::result_large_err)]

pub mod error;
pub mod services;

pub use error::ServerClientError;

use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

// Re-export the generated proto modules so callers don't need to depend on
// `proto-gen` directly to name request/response types.
pub use quark_server_proto::common::v1 as common;
pub use quark_server_proto::server::v1 as proto;

pub use services::organization::OrganizationService;
pub use services::project::ProjectService;
pub use services::server::ServerService;
pub use services::workspace::WorkspaceService;

/// Snapshot of the configuration used to build a [`ServerClient`].
///
/// Captured at build time for introspection / logging. Timeouts are applied to
/// the underlying tonic [`Endpoint`] / [`Channel`] and live there for the real
/// enforcement — this struct is a readable record.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The endpoint URL the client is connected to (e.g. `http://127.0.0.1:5000`).
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
    /// An empty config — used when constructing a [`ServerClient`] directly
    /// from an existing channel via [`ServerClient::from_channel`].
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

/// Top-level server client.
///
/// Holds a single multiplexed gRPC [`Channel`] shared by every service client.
/// Each accessor (`server()`, …) clones the channel (cheap) and wraps
/// the generated tonic client.
///
/// Create one with [`ServerClient::builder`] (recommended) or
/// [`ServerClient::connect`], or wrap an existing channel with
/// [`ServerClient::from_channel`].
#[derive(Clone)]
pub struct ServerClient {
    channel: Channel,
    config: ClientConfig,
    server_service: ServerService,
}

impl std::ops::Deref for ServerClient {
    type Target = ServerService;
    fn deref(&self) -> &Self::Target {
        &self.server_service
    }
}

impl std::ops::DerefMut for ServerClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.server_service
    }
}

impl std::fmt::Debug for ServerClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerClient")
            .field("endpoint", &self.config.endpoint)
            .finish()
    }
}

impl ServerClient {
    /// Start a builder for configuring a new client.
    pub fn builder() -> ServerClientBuilder {
        ServerClientBuilder::default()
    }

    /// Convenience: connect to `endpoint` with default timeouts.
    pub async fn connect(endpoint: &str) -> Result<Self, ServerClientError> {
        Self::builder().endpoint(endpoint).build().await
    }

    /// Wrap an already-connected tonic channel. Useful for sharing a channel
    /// with other clients or for custom transport setups (TLS, middleware, …).
    pub fn from_channel(channel: Channel) -> Self {
        let server_service = ServerService::new(channel.clone());
        Self {
            channel,
            config: ClientConfig::empty(),
            server_service,
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
    // NOTE: ServerService methods (get_service_registry, deploy, rollback, etc.)
    // are available directly on ServerClient via Deref<Target=ServerService>.
    // No need for a separate .server() accessor.

    /// OrganizationService — organizations CRUD + lifecycle.
    ///
    /// Covers all 8 RPCs: `CreateOrganization`, `GetOrganization`,
    /// `ListOrganizations`, `UpdateOrganization`, `ActivateOrganization`,
    /// `DeactivateOrganization`, `ArchiveOrganization`, `DeleteOrganization`.
    pub fn organizations(&self) -> OrganizationService {
        OrganizationService::new(self.channel.clone())
    }

    /// ProjectService — projects CRUD + lifecycle (org-scoped).
    ///
    /// Covers all 8 RPCs: `CreateProject`, `GetProject`, `ListProjects`,
    /// `UpdateProject`, `ActivateProject`, `DeactivateProject`,
    /// `ArchiveProject`, `DeleteProject`.
    pub fn projects(&self) -> ProjectService {
        ProjectService::new(self.channel.clone())
    }

    /// WorkspaceService — workspaces CRUD + lifecycle (project-scoped).
    ///
    /// Covers all 8 RPCs: `CreateWorkspace`, `GetWorkspace`, `ListWorkspaces`,
    /// `UpdateWorkspace`, `ActivateWorkspace`, `DeactivateWorkspace`,
    /// `ArchiveWorkspace`, `DeleteWorkspace`.
    pub fn workspaces(&self) -> WorkspaceService {
        WorkspaceService::new(self.channel.clone())
    }
}

/// Builder for [`ServerClient`].
///
/// Fluent, consumes-and-returns-self. Finalize with
/// [`ServerClientBuilder::build`] (async, connects eagerly) or
/// [`ServerClientBuilder::build_lazy`] (synchronous, connects on first RPC).
///
/// ```no_run
/// use std::time::Duration;
/// use quark_server_rs::ServerClient;
/// # async fn t() -> Result<(), Box<dyn std::error::Error>> {
/// let mut client = ServerClient::builder()
///     .endpoint("http://127.0.0.1:5000")
///     .connect_timeout(Duration::from_secs(5))
///     .request_timeout(Duration::from_secs(30))
///     .keepalive_interval(Duration::from_secs(30))
///     .tcp_nodelay(true)
///     .build()
///     .await?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ServerClientBuilder {
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

impl ServerClientBuilder {
    /// Set the server endpoint URL (e.g. `http://127.0.0.1:5000` or
    /// `https://server.example.com`). Required.
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
    pub async fn build(self) -> Result<ServerClient, ServerClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| ServerClientError::Transport(e.to_string()))?;
        let server_service = ServerService::new(channel.clone());
        Ok(ServerClient {
            channel,
            config: self.snapshot(),
            server_service,
        })
    }

    /// Build a lazily-connected channel — the TCP connection is established on
    /// the first RPC. Returns synchronously.
    pub fn build_lazy(self) -> Result<ServerClient, ServerClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint.connect_lazy();
        let server_service = ServerService::new(channel.clone());
        Ok(ServerClient {
            channel,
            config: self.snapshot(),
            server_service,
        })
    }

    fn build_endpoint(&self) -> Result<Endpoint, ServerClientError> {
        let url = self
            .endpoint
            .as_ref()
            .ok_or_else(|| ServerClientError::Transport("no endpoint configured".into()))?;
        let mut endpoint = Endpoint::from_shared(url.clone())
            .map_err(|e| ServerClientError::Transport(e.to_string()))?;
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
