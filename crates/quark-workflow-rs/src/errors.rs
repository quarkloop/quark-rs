//! SDK error types.
//!
//! Maps gRPC status codes and workflow-specific failures to typed
//! SDK errors so callers can match on them without string parsing.

use thiserror::Error;

/// The top-level SDK error. All SDK operations return `Result<T, SdkError>`.
#[derive(Debug, Error)]
pub enum SdkError {
    /// The requested workflow was not found on the server.
    #[error("workflow not found: {0}")]
    NotFound(String),

    /// A workflow with the given ID is already running.
    #[error("workflow already started: {0}")]
    AlreadyStarted(String),

    /// The workflow completed with a failure.
    #[error("workflow failed: {0}")]
    WorkflowFailed(String),

    /// The workflow or operation timed out.
    #[error("timeout")]
    Timeout,

    /// The workflow was cancelled.
    #[error("cancelled")]
    Cancelled,

    /// A query was rejected by the server (e.g., workflow is closed).
    #[error("query rejected: {0}")]
    QueryRejected(String),

    /// An invalid argument was provided.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Failed to serialize or deserialize a payload.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// A transport-level error (connection refused, network, etc.).
    #[error("transport error: {0}")]
    Transport(String),

    /// An unexpected error that doesn't fit the above categories.
    #[error("unexpected: {0}")]
    Unexpected(String),
}

impl SdkError {
    /// Converts a `tonic::Status` into an `SdkError` by matching on
    /// the gRPC status code.
    pub(crate) fn from_status(status: tonic::Status) -> Self {
        let message = status.message().to_string();
        match status.code() {
            tonic::Code::NotFound => SdkError::NotFound(message),
            tonic::Code::AlreadyExists => SdkError::AlreadyStarted(message),
            tonic::Code::InvalidArgument => SdkError::InvalidArgument(message),
            tonic::Code::Cancelled => SdkError::Cancelled,
            tonic::Code::DeadlineExceeded => SdkError::Timeout,
            tonic::Code::FailedPrecondition => SdkError::QueryRejected(message),
            _ => SdkError::Transport(format!("{}: {}", status.code(), message)),
        }
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self {
        SdkError::Serialization(e.to_string())
    }
}

impl From<tonic::Status> for SdkError {
    fn from(status: tonic::Status) -> Self {
        Self::from_status(status)
    }
}
