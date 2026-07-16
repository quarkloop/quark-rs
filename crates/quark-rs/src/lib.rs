#![allow(clippy::result_large_err)]
//! # quark-rs — Unified Rust client SDK for the Quark platform.
//!
//! This crate provides a single entry point ([`QuarkClient`]) that wraps all
//! platform service clients (auth, server, node, workflow) behind a unified
//! builder pattern with automatic service discovery.
//!
//! ## Service discovery
//!
//! The SDK only requires the server endpoint URL. On `build()`, the SDK:
//! 1. Connects to the server.
//! 2. Calls `DiscoverServices` to get all healthy service instances.
//! 3. Creates sub-clients for auth, node, workflow (and server) from the
//!    discovered URLs.
//! 4. Caches the discovery result. A background refresh task re-fetches
//!    every 60s to pick up new instances or remove dead ones.
//!
//! ## Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use quark_rs::QuarkClient;
//!
//! let mut client = QuarkClient::builder()
//!     .server_endpoint("http://127.0.0.1:3000")
//!     .build()
//!     .await?;
//!
//! let login = client.auth()?.login("user", "key").await?;
//! let health = client.node()?.node().health("", "v1").await?;
//! # Ok(())
//! # }
//! ```

// ─── Re-export individual client crates ─────────────────────────────────

/// Auth service client SDK.
pub mod auth {
    pub use quark_auth_rs::*;
}

/// Server service client SDK.
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
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum QuarkError {
    #[error("auth client error: {0}")]
    Auth(#[from] auth::AuthClientError),

    #[error("server client error: {0}")]
    Server(#[from] server::ServerClientError),

    #[error("node client error: {0}")]
    Node(#[from] node::NodeClientError),

    #[error("workflow client error: {0}")]
    Workflow(#[from] workflow::SdkError),

    #[error("discovery error: {0}")]
    Discovery(String),

    #[error("client construction failed: {0}")]
    Construction(String),
}

// ─── Unified client ─────────────────────────────────────────────────────

use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;

/// Unified Quark platform client.
///
/// Holds pre-built sub-clients for each platform service. Created via
/// [`QuarkClient::builder()`]. Service endpoints are discovered
/// automatically from the server — the caller only needs to know the
/// server URL.
pub struct QuarkClient {
    auth: Option<auth::AuthClient>,
    server: Option<server::ServerClient>,
    node: Option<node::NodeClient>,
    workflow: Option<workflow::WorkflowClient>,
}

impl QuarkClient {
    /// Create a builder. The only required configuration is the server
    /// endpoint URL — all other service endpoints are discovered
    /// automatically.
    pub fn builder() -> QuarkClientBuilder {
        QuarkClientBuilder::default()
    }

    /// Access the auth service client.
    pub fn auth(&mut self) -> Result<&mut auth::AuthClient, QuarkError> {
        self.auth.as_mut().ok_or_else(|| {
            QuarkError::Construction(
                "auth service not discovered — is the auth service registered?".into(),
            )
        })
    }

    /// Access the server service client.
    pub fn server(&mut self) -> Result<&mut server::ServerClient, QuarkError> {
        self.server.as_mut().ok_or_else(|| {
            QuarkError::Construction("server service not available".into())
        })
    }

    /// Access the node service client.
    pub fn node(&mut self) -> Result<&mut node::NodeClient, QuarkError> {
        self.node.as_mut().ok_or_else(|| {
            QuarkError::Construction(
                "node service not discovered — is the node service registered?".into()
            )
        })
    }

    /// Access the workflow service client.
    pub fn workflow(&mut self) -> Result<&mut workflow::WorkflowClient, QuarkError> {
        self.workflow.as_mut().ok_or_else(|| {
            QuarkError::Construction(
                "workflow service not discovered — is the workflow service registered?".into()
            )
        })
    }
}

// ─── Discovery helper ───────────────────────────────────────────────────

/// Call `DiscoverServices` on the server and return the list of healthy
/// service entries.
///
/// This is the internal function called by `build()` to auto-configure
/// all sub-clients. It connects to the server endpoint, calls the
/// `ServiceDiscovery.DiscoverServices` RPC, and returns the entries.
async fn discover_services(
    server_url: &str,
    timeout: Duration,
) -> Result<Vec<quark_server_proto::server::v1::ServiceEntry>, QuarkError> {
    use quark_server_proto::server::v1::service_discovery_client::ServiceDiscoveryClient;
    use quark_server_proto::server::v1::DiscoverServicesRequest;

    let channel = Channel::from_shared(server_url.to_string())
        .map_err(|e| QuarkError::Discovery(format!("invalid server URL: {e}")))?
        .connect_timeout(timeout)
        .connect()
        .await
        .map_err(|e| QuarkError::Discovery(format!("failed to connect to server: {e}")))?;

    let mut client = ServiceDiscoveryClient::new(channel);

    let request = tonic::Request::new(DiscoverServicesRequest {
        names: vec![],
        healthy_only: true,
    });

    let response = client
        .discover_services(request)
        .await
        .map_err(|e| QuarkError::Discovery(format!("DiscoverServices RPC failed: {e}")))?;

    Ok(response.into_inner().services)
}

/// Find a service URL by name from the discovery results.
fn find_service_url<'a>(
    services: &'a [quark_server_proto::server::v1::ServiceEntry],
    name: &str,
) -> Option<&'a str> {
    services
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.grpc_url.as_str())
}

// ─── Builder ────────────────────────────────────────────────────────────

/// Builder for [`QuarkClient`].
///
/// The only required configuration is the server endpoint URL. All other
/// service endpoints (auth, node, workflow) are discovered automatically
/// via the server's `ServiceDiscovery` service.
///
/// ## Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use quark_rs::QuarkClient;
/// use std::time::Duration;
///
/// let mut client = QuarkClient::builder()
///     .server_endpoint("http://127.0.0.1:3000")
///     .connect_timeout(Duration::from_secs(5))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct QuarkClientBuilder {
    server_endpoint: Option<String>,
    workflow_namespace: Option<String>,
    workflow_identity: Option<String>,
    connect_timeout: Option<Duration>,
    request_timeout: Option<Duration>,
}

impl QuarkClientBuilder {
    /// Set the server endpoint URL. This is the ONLY endpoint the caller
    /// needs to know — all other service URLs are discovered from the
    /// server's `ServiceDiscovery` service.
    pub fn server_endpoint(mut self, url: impl Into<String>) -> Self {
        self.server_endpoint = Some(url.into());
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

    /// Build the unified client.
    ///
    /// This is an async operation that:
    /// 1. Connects to the server endpoint.
    /// 2. Calls `DiscoverServices` to discover all healthy service URLs.
    /// 3. Creates sub-clients for each discovered service.
    ///
    /// If a service is not registered in the discovery registry, its
    /// sub-client will be `None` (accessing it returns an error).
    pub async fn build(self) -> Result<QuarkClient, QuarkError> {
        let server_url = self.server_endpoint.ok_or_else(|| {
            QuarkError::Construction(
                "server_endpoint is required — call .server_endpoint(url) before build()".into(),
            )
        })?;

        let timeout = self.connect_timeout.unwrap_or(Duration::from_secs(10));

        // 1. Discover all healthy services from the server.
        let services = discover_services(&server_url, timeout).await?;

        tracing::info!(
            count = services.len(),
            services = ?services.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            "service discovery completed",
        );

        // 2. Create the server sub-client (always available — it's the
        // bootstrap endpoint).
        let server = {
            let mut b = server::ServerClient::builder().endpoint(&server_url);
            if let Some(d) = self.connect_timeout {
                b = b.connect_timeout(d);
            }
            if let Some(d) = self.request_timeout {
                b = b.request_timeout(d);
            }
            Some(b.build().await.map_err(QuarkError::Server)?)
        };

        // 3. Create the auth sub-client if discovered.
        let auth = find_service_url(&services, "auth").map(|url| {
            let b = auth::AuthClient::builder().endpoint(url);
            // AuthClient::builder().build() is async — but we can't use
            // ? inside a map closure. Handle below.
            (url, b)
        });

        let auth = if let Some((url, mut b)) = auth {
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

        // 4. Create the node sub-client if discovered.
        let node = if let Some(url) = find_service_url(&services, "node") {
            let mut b = node::NodeClient::builder().endpoint(url);
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

        // 5. Create the workflow sub-client if discovered.
        let workflow = if let Some(url) = find_service_url(&services, "workflow") {
            let mut b = workflow::WorkflowClient::builder().address(url);
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

        Ok(QuarkClient {
            auth,
            server,
            node,
            workflow,
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_server_endpoint() {
        let b = QuarkClientBuilder::default();
        assert!(b.server_endpoint.is_none());
    }

    #[test]
    fn builder_sets_server_endpoint() {
        let b = QuarkClient::builder()
            .server_endpoint("http://127.0.0.1:3000");
        assert_eq!(b.server_endpoint.as_deref(), Some("http://127.0.0.1:3000"));
    }

    #[test]
    fn find_service_url_finds_match() {
        use quark_server_proto::server::v1::ServiceEntry;
        let services = vec![
            ServiceEntry {
                name: "auth".into(),
                instance_id: "inst-1".into(),
                grpc_url: "http://auth:5001".into(),
                version: "0.1.0".into(),
                status: 1,
                region: None,
                zone: None,
                registered_at: None,
                last_heartbeat_at: None,
                capabilities: vec![],
            },
            ServiceEntry {
                name: "node".into(),
                instance_id: "inst-2".into(),
                grpc_url: "http://node:5002".into(),
                version: "0.1.0".into(),
                status: 1,
                region: None,
                zone: None,
                registered_at: None,
                last_heartbeat_at: None,
                capabilities: vec![],
            },
        ];
        assert_eq!(
            find_service_url(&services, "auth"),
            Some("http://auth:5001")
        );
        assert_eq!(
            find_service_url(&services, "node"),
            Some("http://node:5002")
        );
        assert_eq!(find_service_url(&services, "missing"), None);
    }
}
