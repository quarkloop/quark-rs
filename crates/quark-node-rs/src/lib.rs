//! Ergonomic gRPC client SDK for the `quark-noded` daemon.
//!
//! `node-client` is the primary client SDK for talking to a `quark-noded`
//! daemon over gRPC. It wraps the generated tonic client
//! (`quark_node_proto::v1`) with a Supabase-style builder pattern and typed
//! convenience methods covering **all 7 RPCs** of `NodeService` defined in
//! [`proto/node.proto`](../proto/node.proto).
//!
//! The daemon listens on `--grpc-addr` (default `0.0.0.0:50051`) and serves
//! the seven RPCs: `Execute`, `Cancel`, `Health`, `Ready`, `Status`, `Drain`,
//! `Shutdown`. See `docs/content/develop/specs/08-api.mdx` for the full API
//! specification and `docs/content/develop/adr/0001-why-grpc.mdx` for the
//! architectural rationale.
//!
//! # Quick start
//!
//! ```no_run
//! use std::time::Duration;
//! use node_client::NodeClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = NodeClient::builder()
//!     .endpoint("http://127.0.0.1:50051")
//!     .connect_timeout(Duration::from_secs(5))
//!     .request_timeout(Duration::from_secs(30))
//!     .build()
//!     .await?;
//!
//! // Liveness check.
//! let health = client.node().health("").await?;
//! println!("daemon status: {} (uptime {}ms)", health.status, health.uptime_ms);
//!
//! // Execute a node.
//! let resp = client.node()
//!     .execute(
//!         "",
//!         "v1",
//!         "req-1",
//!         "test/echo/native/reverse:1.0.0",
//!         None,
//!         30_000,
//!         "request",
//!         "", "", "", "",
//!     )
//!     .await?;
//! println!("execute status: {}", resp.status);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Service accessors
//!
//! Every proto service has a dedicated accessor on [`NodeClient`] that returns
//! a cheap, owned service client built from a cloned
//! [`tonic::transport::Channel`]:
//!
//! | Accessor               | Service client        | Service       |
//! |------------------------|-----------------------|---------------|
//! | [`NodeClient::node`]   | [`services::NodeService`] | NodeService |
//!
//! Service clients are created on demand and cheap to clone — the underlying
//! gRPC channel is multiplexed (HTTP/2) and shared.

// The SDK deliberately maps every proto field of every RPC to a typed Rust
// parameter (no simplification), so several service methods exceed clippy's
// 7-argument threshold. Boxing every such call site behind a params struct
// would obscure the 1:1 mapping with the proto contract, so we allow the lint
// crate-wide.
#![allow(clippy::too_many_arguments)]
// `NodeClientError` holds a `tonic::Status` (~176 bytes) so its `Result`s
// trip `result_large_err`. This matches the precedent set by the sibling
// `auth-client` crate and is inherent to surfacing full gRPC status details to
// callers.
#![allow(clippy::result_large_err)]

pub mod error;
pub mod services;

pub use error::NodeClientError;

use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

// Re-export the generated proto module so callers don't need to depend on
// `quark-proto-gen` directly to name request/response types.
pub use quark_node_proto::v1 as proto;

pub use services::node::NodeService;

/// Snapshot of the configuration used to build a [`NodeClient`].
///
/// Captured at build time for introspection / logging. Timeouts are applied to
/// the underlying tonic [`Endpoint`] / [`Channel`] and live there for the real
/// enforcement — this struct is a readable record.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The endpoint URL the client is connected to (e.g. `http://127.0.0.1:50051`).
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
    /// An empty config — used when constructing a [`NodeClient`] directly from
    /// an existing channel via [`NodeClient::from_channel`].
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

/// Top-level `quark-noded` client.
///
/// Holds a single multiplexed gRPC [`Channel`] shared by every service client.
/// Each accessor (`node()`, …) clones the channel (cheap) and wraps the
/// generated tonic client.
///
/// Create one with [`NodeClient::builder`] (recommended) or
/// [`NodeClient::connect`], or wrap an existing channel with
/// [`NodeClient::from_channel`].
#[derive(Clone)]
pub struct NodeClient {
    channel: Channel,
    config: ClientConfig,
}

impl std::fmt::Debug for NodeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeClient")
            .field("endpoint", &self.config.endpoint)
            .finish()
    }
}

impl NodeClient {
    /// Start a builder for configuring a new client.
    pub fn builder() -> NodeClientBuilder {
        NodeClientBuilder::default()
    }

    /// Convenience: connect to `endpoint` with default timeouts.
    pub async fn connect(endpoint: &str) -> Result<Self, NodeClientError> {
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

    /// `NodeService` — the gRPC API for the node execution daemon.
    ///
    /// Covers all 7 RPCs: `Execute`, `Cancel`, `Health`, `Ready`, `Status`,
    /// `Drain`, `Shutdown`.
    pub fn node(&self) -> NodeService {
        NodeService::new(self.channel.clone())
    }
}

/// Builder for [`NodeClient`].
///
/// Fluent, consumes-and-returns-self. Finalize with [`NodeClientBuilder::build`]
/// (async, connects eagerly) or [`NodeClientBuilder::build_lazy`] (synchronous,
/// connects on first RPC).
///
/// ```no_run
/// use std::time::Duration;
/// use node_client::NodeClient;
/// # async fn t() -> Result<(), Box<dyn std::error::Error>> {
/// let client = NodeClient::builder()
///     .endpoint("http://127.0.0.1:50051")
///     .connect_timeout(Duration::from_secs(5))
///     .request_timeout(Duration::from_secs(30))
///     .keepalive_interval(Duration::from_secs(30))
///     .tcp_nodelay(true)
///     .build()
///     .await?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Default)]
pub struct NodeClientBuilder {
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

impl NodeClientBuilder {
    /// Set the daemon endpoint URL (e.g. `http://127.0.0.1:50051` or
    /// `https://node.example.com`). Required.
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
    pub async fn build(self) -> Result<NodeClient, NodeClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| NodeClientError::Transport(e.to_string()))?;
        Ok(NodeClient {
            channel,
            config: self.snapshot(),
        })
    }

    /// Build a lazily-connected channel — the TCP connection is established on
    /// the first RPC. Returns synchronously.
    pub fn build_lazy(self) -> Result<NodeClient, NodeClientError> {
        let endpoint = self.build_endpoint()?;
        let channel = endpoint.connect_lazy();
        Ok(NodeClient {
            channel,
            config: self.snapshot(),
        })
    }

    fn build_endpoint(&self) -> Result<Endpoint, NodeClientError> {
        let url = self
            .endpoint
            .as_ref()
            .ok_or_else(|| NodeClientError::Transport("no endpoint configured".into()))?;
        let mut endpoint = Endpoint::from_shared(url.clone())
            .map_err(|e| NodeClientError::Transport(e.to_string()))?;
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
