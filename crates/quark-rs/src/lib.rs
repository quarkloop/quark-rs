#![allow(clippy::result_large_err)]
//! # quark-rs — Unified Rust client SDK for the Quark platform.
//!
//! This crate provides a single entry point ([`QuarkClient`]) that wraps all
//! platform service clients (auth, server, node, workflow) behind a unified
//! factory + builder pattern.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use quark_rs::QuarkClient;
//!
//! // Build all sub-clients from a single configuration.
//! let client = QuarkClient::builder()
//!     .auth_endpoint("http://127.0.0.1:5001")
//!     .server_endpoint("http://127.0.0.1:3000")
//!     .node_endpoint("http://127.0.0.1:50051")
//!     .workflow_endpoint("http://127.0.0.1:7233")
//!     .build()
//!     .await?;
//!
//! // Access each service client (returns Result; unwrap since endpoints are configured).
//! let login = client.auth()?.auth().login("user", "key").await?;
//! let registry = client.server()?.server().get_service_registry("token").await?;
//! let health = client.node()?.node().health("", "v1").await?;
//! # Ok(())
//! # }
//! ```
//!
//! You can also use individual sub-clients directly without the unified
//! facade — each is re-exported at the crate root:
//!
//! ```no_run
//! use quark_rs::auth::AuthClient;
//! ```

// ─── Re-export individual client crates ─────────────────────────────────

/// Auth service client SDK.
pub mod auth {
    pub use quark_auth_rs::*;
}

/// Server (server) service client SDK.
pub mod server {
    pub use quark_server_rs::*;
}

/// Node service client SDK.
pub mod node {
    pub use quark_node_rs::*;
}

/// Workflow service client SDK.
pub mod workflow {
    pub use quark_workflow_rs::*;
}

// ─── Unified error type ─────────────────────────────────────────────────

use thiserror::Error;

/// Unified error type that wraps all service-specific client errors.
///
/// Returned by [`QuarkClient`] methods and by the unified builder.
/// Individual sub-clients still return their own error types
/// (`AuthClientError`, `ServerClientError`, etc.) — this enum exists so
/// callers that work across services can handle errors in a single `match`.
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum QuarkError {
    /// Error from the auth service client.
    #[error("auth client error: {0}")]
    Auth(#[from] auth::AuthClientError),

    /// Error from the server service client.
    #[error("server client error: {0}")]
    Server(#[from] server::ServerClientError),

    /// Error from the node service client.
    #[error("node client error: {0}")]
    Node(#[from] node::NodeClientError),

    /// Error from the workflow service client.
    #[error("workflow client error: {0}")]
    Workflow(#[from] workflow::SdkError),

    /// Error during client construction (e.g. failed to connect).
    #[error("client construction failed: {0}")]
    Construction(String),
}

// ─── Unified client ─────────────────────────────────────────────────────

use std::time::Duration;

/// Unified Quark platform client.
///
/// Holds pre-built sub-clients for each platform service. Created via
/// [`QuarkClient::builder()`] or [`QuarkClient::from_parts()`].
///
/// Each accessor returns a reference to the sub-client, which in turn
/// provides access to its own service-specific methods.
pub struct QuarkClient {
    auth: Option<auth::AuthClient>,
    server: Option<server::ServerClient>,
    node: Option<node::NodeClient>,
    workflow: Option<workflow::WorkflowClient>,
}

impl QuarkClient {
    /// Create a builder for configuring all sub-clients.
    pub fn builder() -> QuarkClientBuilder {
        QuarkClientBuilder::default()
    }

    /// Assemble from pre-built sub-clients. Any `None` field means that
    /// service is not configured — accessing it via [`auth()`], [`server()`],
    /// etc. will return an error.
    pub fn from_parts(
        auth: Option<auth::AuthClient>,
        server: Option<server::ServerClient>,
        node: Option<node::NodeClient>,
        workflow: Option<workflow::WorkflowClient>,
    ) -> Self {
        Self { auth, server, node, workflow }
    }

    /// Access the auth service client.
    ///
    /// Returns `Err` if no auth endpoint was configured.
    pub fn auth(&self) -> Result<&auth::AuthClient, QuarkError> {
        self.auth.as_ref().ok_or_else(|| {
            QuarkError::Construction("auth endpoint not configured".into())
        })
    }

    /// Access the server service client.
    ///
    /// Returns `Err` if no server endpoint was configured.
    pub fn server(&self) -> Result<&server::ServerClient, QuarkError> {
        self.server.as_ref().ok_or_else(|| {
            QuarkError::Construction("server endpoint not configured".into())
        })
    }

    /// Access the node service client.
    ///
    /// Returns `Err` if no node endpoint was configured.
    pub fn node(&self) -> Result<&node::NodeClient, QuarkError> {
        self.node.as_ref().ok_or_else(|| {
            QuarkError::Construction("node endpoint not configured".into())
        })
    }

    /// Access the workflow service client.
    ///
    /// Returns `Err` if no workflow endpoint was configured.
    pub fn workflow(&self) -> Result<&workflow::WorkflowClient, QuarkError> {
        self.workflow.as_ref().ok_or_else(|| {
            QuarkError::Construction("workflow endpoint not configured".into())
        })
    }
}

// ─── Builder ────────────────────────────────────────────────────────────

/// Builder for [`QuarkClient`].
///
/// Configure each service endpoint individually. Any service left
/// unconfigured will be `None` in the final client (accessing it returns
/// an error). Shared transport options (timeouts, keepalive) are applied
/// to all configured sub-clients.
///
/// ## Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use quark_rs::QuarkClient;
/// use std::time::Duration;
///
/// let client = QuarkClient::builder()
///     .auth_endpoint("http://127.0.0.1:5001")
///     .server_endpoint("http://127.0.0.1:3000")
///     .connect_timeout(Duration::from_secs(5))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct QuarkClientBuilder {
    // Endpoint URLs (None = service not configured)
    auth_endpoint: Option<String>,
    server_endpoint: Option<String>,
    node_endpoint: Option<String>,
    workflow_endpoint: Option<String>,

    // Workflow-specific options
    workflow_namespace: Option<String>,
    workflow_identity: Option<String>,

    // Shared transport options (applied to all configured sub-clients)
    connect_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
}

impl QuarkClientBuilder {
    /// Set the auth service endpoint URL.
    pub fn auth_endpoint(mut self, url: impl Into<String>) -> Self {
        self.auth_endpoint = Some(url.into());
        self
    }

    /// Set the server (server) endpoint URL.
    pub fn server_endpoint(mut self, url: impl Into<String>) -> Self {
        self.server_endpoint = Some(url.into());
        self
    }

    /// Set the node service endpoint URL.
    pub fn node_endpoint(mut self, url: impl Into<String>) -> Self {
        self.node_endpoint = Some(url.into());
        self
    }

    /// Set the workflow service endpoint URL.
    pub fn workflow_endpoint(mut self, url: impl Into<String>) -> Self {
        self.workflow_endpoint = Some(url.into());
        self
    }

    /// Set the workflow namespace (default: "default").
    pub fn workflow_namespace(mut self, ns: impl Into<String>) -> Self {
        self.workflow_namespace = Some(ns.into());
        self
    }

    /// Set the workflow client identity (e.g. "my-app@1.0").
    pub fn workflow_identity(mut self, id: impl Into<String>) -> Self {
        self.workflow_identity = Some(id.into());
        self
    }

    /// Set the connection timeout for all sub-clients.
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = Some(d);
        self
    }

    /// Set the request timeout for all sub-clients.
    pub fn request_timeout(mut self, d: Duration) -> Self {
        self.request_timeout = Some(d);
        self
    }

    /// Build the unified client, connecting to all configured services.
    ///
    /// This is an async operation — each configured sub-client will
    /// establish a gRPC connection to its endpoint. If a sub-client fails
    /// to connect, the entire build fails.
    pub async fn build(self) -> Result<QuarkClient, QuarkError> {
        let auth = if let Some(url) = self.auth_endpoint {
            let mut b = auth::AuthClient::builder().endpoint(&url);
            if let Some(d) = self.connect_timeout {
                b = b.connect_timeout(d);
            }
            if let Some(d) = self.request_timeout {
                b = b.request_timeout(d);
            }
            Some(b.build().await.map_err(QuarkError::Auth)?)
        } else {
            None
        };

        let server = if let Some(url) = self.server_endpoint {
            let mut b = server::ServerClient::builder().endpoint(&url);
            if let Some(d) = self.connect_timeout {
                b = b.connect_timeout(d);
            }
            if let Some(d) = self.request_timeout {
                b = b.request_timeout(d);
            }
            Some(b.build().await.map_err(QuarkError::Server)?)
        } else {
            None
        };

        let node = if let Some(url) = self.node_endpoint {
            let mut b = node::NodeClient::builder().endpoint(&url);
            if let Some(d) = self.connect_timeout {
                b = b.connect_timeout(d);
            }
            if let Some(d) = self.request_timeout {
                b = b.request_timeout(d);
            }
            Some(b.build().await.map_err(QuarkError::Node)?)
        } else {
            None
        };

        let workflow = if let Some(url) = self.workflow_endpoint {
            let mut b = workflow::WorkflowClient::builder().address(&url);
            if let Some(ns) = &self.workflow_namespace {
                b = b.namespace(ns);
            } else {
                b = b.namespace("default");
            }
            if let Some(id) = &self.workflow_identity {
                b = b.identity(id);
            }
            Some(b.build().await.map_err(QuarkError::Workflow)?)
        } else {
            None
        };

        Ok(QuarkClient { auth, server, node, workflow })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults() {
        let b = QuarkClientBuilder::default();
        assert!(b.auth_endpoint.is_none());
        assert!(b.server_endpoint.is_none());
        assert!(b.node_endpoint.is_none());
        assert!(b.workflow_endpoint.is_none());
    }

    #[test]
    fn builder_set_endpoints() {
        let b = QuarkClient::builder()
            .auth_endpoint("http://auth:5001")
            .server_endpoint("http://server:3000")
            .node_endpoint("http://node:50051")
            .workflow_endpoint("http://wf:7233");
        assert_eq!(b.auth_endpoint.as_deref(), Some("http://auth:5001"));
        assert_eq!(b.server_endpoint.as_deref(), Some("http://server:3000"));
        assert_eq!(b.node_endpoint.as_deref(), Some("http://node:50051"));
        assert_eq!(b.workflow_endpoint.as_deref(), Some("http://wf:7233"));
    }

    #[test]
    fn from_parts_none_accessors_error() {
        let client = QuarkClient::from_parts(None, None, None, None);
        assert!(client.auth().is_err());
        assert!(client.server().is_err());
        assert!(client.node().is_err());
        assert!(client.workflow().is_err());
    }

    #[test]
    fn quark_error_from_auth_error() {
        let auth_err = auth::AuthClientError::Transport("test".into());
        let quark_err: QuarkError = auth_err.into();
        assert!(matches!(quark_err, QuarkError::Auth(_)));
    }

    #[test]
    fn quark_error_from_server_error() {
        let server_err = server::ServerClientError::Transport("test".into());
        let quark_err: QuarkError = server_err.into();
        assert!(matches!(quark_err, QuarkError::Server(_)));
    }

    #[test]
    fn quark_error_from_node_error() {
        let node_err = node::NodeClientError::Transport("test".into());
        let quark_err: QuarkError = node_err.into();
        assert!(matches!(quark_err, QuarkError::Node(_)));
    }
}
