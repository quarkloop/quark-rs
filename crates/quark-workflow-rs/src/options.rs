//! Options types for workflow operations.
//!
//! These types configure how workflows are started and interacted
//! with. They are used by the builder pattern — no defaults are
//! silently applied except where noted.

use std::collections::HashMap;
use std::time::Duration;

use quark_workflow_proto::temporal::api::common::v1 as commonpb;

/// Options for starting a workflow execution.
///
/// These are set via the `WorkflowStartBuilder`. Fields default to
/// `None` / `false` when not explicitly set.
#[derive(Clone, Debug, Default)]
pub struct WorkflowOptions {
    /// Maximum time the workflow is allowed to run.
    pub workflow_run_timeout: Option<Duration>,

    /// Maximum time a single workflow task is allowed to run.
    pub workflow_task_timeout: Option<Duration>,

    /// Search attributes for advanced visibility.
    pub search_attributes: Option<HashMap<String, commonpb::Payload>>,

    /// Memo (non-indexed key-value pairs attached to the workflow).
    pub memo: Option<HashMap<String, commonpb::Payload>>,

    /// Whether to request eager workflow start (start on a local worker).
    pub request_eager_start: bool,
}

/// Connection options for the gRPC channel.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ConnectionOptions {
    /// Server address in `host:port` format.
    pub address: String,

    /// Request timeout for all gRPC calls.
    pub timeout: Option<Duration>,

    /// Keepalive interval for the gRPC channel.
    pub keepalive: Option<Duration>,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            address: "http://localhost:7233".to_string(),
            timeout: None,
            keepalive: None,
        }
    }
}

/// TLS configuration for the gRPC channel.
#[derive(Clone, Debug, Default)]
pub struct TlsConfig {
    /// Path to the CA certificate file (PEM format).
    pub ca_cert_path: Option<String>,

    /// Path to the client certificate file (PEM format).
    pub client_cert_path: Option<String>,

    /// Path to the client key file (PEM format).
    pub client_key_path: Option<String>,

    /// Server name to verify in the certificate.
    pub server_name: Option<String>,
}

/// Description of a workflow execution returned by `describe()`.
#[derive(Clone, Debug)]
pub struct WorkflowExecutionDescription {
    /// The workflow ID.
    pub workflow_id: String,
    /// The run ID.
    pub run_id: String,
    /// The workflow type name.
    pub workflow_type: String,
    /// The task queue name.
    pub task_queue: String,
    /// Workflow status (Running, Completed, Failed, etc.).
    pub status: i32,
    /// Number of events in the workflow history.
    pub history_length: i64,
    /// Workflow start time (Unix millis).
    pub start_time: Option<prost_types::Timestamp>,
    /// Workflow close time (Unix millis), if closed.
    pub close_time: Option<prost_types::Timestamp>,
    /// Search attributes attached to the workflow.
    pub search_attributes: Option<commonpb::SearchAttributes>,
    /// Memo attached to the workflow.
    pub memo: Option<commonpb::Memo>,
    /// Pending activities (non-terminal activities).
    pub pending_activities: Vec<PendingActivityInfo>,
}

/// Information about a pending activity in a workflow execution.
#[derive(Clone, Debug)]
pub struct PendingActivityInfo {
    /// The activity ID.
    pub activity_id: String,
    /// The activity type name.
    pub activity_type: String,
    /// The activity state (Scheduled, Started, etc.).
    pub state: i32,
    /// The activity's attempt number.
    pub attempt: i32,
}

/// Information about a workflow execution returned by `list()`.
#[derive(Clone, Debug)]
pub struct WorkflowExecutionInfo {
    /// The workflow ID.
    pub workflow_id: String,
    /// The run ID.
    pub run_id: String,
    /// The workflow type name.
    pub workflow_type: String,
    /// The task queue name.
    pub task_queue: String,
    /// Workflow status.
    pub status: i32,
    /// Number of events in history.
    pub history_length: i64,
    /// Start time.
    pub start_time: Option<prost_types::Timestamp>,
    /// Close time, if closed.
    pub close_time: Option<prost_types::Timestamp>,
}

/// Description of a namespace.
#[derive(Clone, Debug)]
pub struct NamespaceDescription {
    /// The namespace name.
    pub name: String,
    /// The namespace ID.
    pub id: String,
    /// Namespace state (Registered, Deprecated, Deleted).
    pub state: i32,
    /// Namespace description.
    pub description: String,
    /// Retention period for workflow executions.
    pub retention_period: Option<prost_types::Duration>,
}

/// Request to register a new namespace.
#[derive(Clone, Debug, Default)]
pub struct RegisterNamespaceRequest {
    /// The namespace name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Retention period for workflow executions.
    pub retention_period: Option<Duration>,
}
