//! `NodeService` — the gRPC API for the node execution daemon.
//!
//! Wraps `quark_node_proto::v1::node_service_client::NodeServiceClient`.
//!
//! Covers all 7 RPCs of `NodeService`:
//! `Execute`, `Cancel`, `Health`, `Ready`, `Status`, `Drain`, `Shutdown`.
//!
//! The daemon does not currently enforce bearer auth at the gRPC layer (there
//! is no server-side interceptor — see `noded/src/main.rs`). Every method on
//! this client still takes a `token: &str` first argument for
//! forward-compatibility and gateway passthrough; an empty token is a no-op.

use prost_types::Value;
use quark_node_proto::v1::node_service_client::NodeServiceClient;
use quark_node_proto::v1::RequestMetadata;
use tonic::transport::Channel;

use crate::error::NodeClientError;
use crate::services::attach_bearer;

/// Client for `NodeService`.
pub struct NodeService {
    inner: NodeServiceClient<Channel>,
}

impl NodeService {
    /// Wrap a generated `NodeServiceClient` over a shared channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: NodeServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch for advanced use).
    pub fn inner(&mut self) -> &mut NodeServiceClient<Channel> {
        &mut self.inner
    }

    // ─── Execute ─────────────────────────────────────────────────────────────

    /// `Execute` — execute a node and return the result.
    ///
    /// `input` is the optional input payload as a protobuf `Value`; pass
    /// `None` for a no-input execution. The four metadata fields (`trace_id`,
    /// `span_id`, `caller_id`, `caller_ip`) populate the optional
    /// `RequestMetadata`; if all four are empty, no metadata is sent.
    ///
    /// `mode` is the execution mode: `"request"`, `"stream"`, `"transform"`,
    /// `"serve"`, or `"batch"`. `deadline_ms` bounds the execution; the daemon
    /// returns `DeadlineExceeded` if it is exceeded.
    pub async fn execute(
        &mut self,
        token: &str,
        api_version: &str,
        request_id: &str,
        node_uri: &str,
        input: Option<Value>,
        deadline_ms: u64,
        mode: &str,
        trace_id: &str,
        span_id: &str,
        caller_id: &str,
        caller_ip: &str,
    ) -> Result<quark_node_proto::v1::ExecuteResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::ExecuteRequest {
            api_version: api_version.to_string(),
            request_id: request_id.to_string(),
            node_uri: node_uri.to_string(),
            input,
            deadline_ms,
            mode: mode.to_string(),
            metadata: request_metadata(trace_id, span_id, caller_id, caller_ip),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.execute(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Cancel ──────────────────────────────────────────────────────────────

    /// `Cancel` — cancel an in-flight execution by `request_id`.
    pub async fn cancel(
        &mut self,
        token: &str,
        api_version: &str,
        request_id: &str,
        reason: &str,
    ) -> Result<quark_node_proto::v1::CancelResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::CancelRequest {
            api_version: api_version.to_string(),
            request_id: request_id.to_string(),
            reason: reason.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.cancel(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Health / Ready / Status ─────────────────────────────────────────────

    /// `Health` — liveness check. Is the daemon process alive?
    pub async fn health(
        &mut self,
        token: &str,
        api_version: &str,
    ) -> Result<quark_node_proto::v1::HealthResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::HealthRequest {
            api_version: api_version.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.health(req).await?;
        Ok(resp.into_inner())
    }

    /// `Ready` — readiness check. Is the daemon ready to accept requests?
    pub async fn ready(
        &mut self,
        token: &str,
        api_version: &str,
    ) -> Result<quark_node_proto::v1::ReadyResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::ReadyRequest {
            api_version: api_version.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.ready(req).await?;
        Ok(resp.into_inner())
    }

    /// `Status` — detailed runtime status (host ID, uptime, catalog size,
    /// execution counters).
    pub async fn status(
        &mut self,
        token: &str,
        api_version: &str,
    ) -> Result<quark_node_proto::v1::StatusResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::StatusRequest {
            api_version: api_version.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.status(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Drain / Shutdown ────────────────────────────────────────────────────

    /// `Drain` — stop accepting new requests, finish in-flight ones (up to
    /// `timeout_ms`), then return the count drained.
    pub async fn drain(
        &mut self,
        token: &str,
        api_version: &str,
        timeout_ms: u64,
    ) -> Result<quark_node_proto::v1::DrainResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::DrainRequest {
            api_version: api_version.to_string(),
            timeout_ms,
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.drain(req).await?;
        Ok(resp.into_inner())
    }

    /// `Shutdown` — stop the daemon. Pass `force = true` to abort in-flight
    /// requests immediately; `false` for a graceful shutdown.
    pub async fn shutdown(
        &mut self,
        token: &str,
        api_version: &str,
        force: bool,
    ) -> Result<quark_node_proto::v1::ShutdownResponse, NodeClientError> {
        let mut req = tonic::Request::new(quark_node_proto::v1::ShutdownRequest {
            api_version: api_version.to_string(),
            force,
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.shutdown(req).await?;
        Ok(resp.into_inner())
    }
}

/// Build the optional `RequestMetadata` for `Execute`.
///
/// Returns `None` when all four fields are empty, preserving the proto's
/// `optional` semantics. If any field is non-empty, a `RequestMetadata` is
/// sent with the remaining fields defaulted to empty strings.
fn request_metadata(
    trace_id: &str,
    span_id: &str,
    caller_id: &str,
    caller_ip: &str,
) -> Option<RequestMetadata> {
    if trace_id.is_empty()
        && span_id.is_empty()
        && caller_id.is_empty()
        && caller_ip.is_empty()
    {
        None
    } else {
        Some(RequestMetadata {
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            caller_id: caller_id.to_string(),
            caller_ip: caller_ip.to_string(),
        })
    }
}
