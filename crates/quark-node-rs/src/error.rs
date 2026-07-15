//! Error type for the `quark-noded` client SDK.

use thiserror::Error;

/// All errors returned by [`crate::NodeClient`] service methods.
///
/// Three broad categories:
/// - [`NodeClientError::Transport`] — connection / transport-level failures
///   (DNS, TCP, TLS, channel closure, malformed endpoint URI, …).
/// - [`NodeClientError::Status`] — a gRPC call reached the daemon but was
///   rejected with a non-OK [`tonic::Status`] (e.g. `Unavailable`,
///   `Internal`, `InvalidArgument`).
/// - [`NodeClientError::InvalidResponse`] — the call succeeded but the
///   response could not be interpreted as expected (decoding / shape mismatch).
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum NodeClientError {
    /// A transport-level failure: the RPC never reached a successful gRPC
    /// exchange. Includes channel connect errors and invalid endpoint URIs.
    #[error("gRPC transport error: {0}")]
    Transport(String),

    /// The daemon returned a gRPC error status.
    #[error("gRPC status: {0}")]
    Status(#[from] tonic::Status),

    /// The response was received but could not be interpreted (e.g. a missing
    /// required field, malformed payload, unexpected empty body).
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl NodeClientError {
    /// True if this error is a transport-level failure.
    pub fn is_transport(&self) -> bool {
        matches!(self, Self::Transport(_))
    }

    /// True if this error is a gRPC status returned by the daemon.
    pub fn is_status(&self) -> bool {
        matches!(self, Self::Status(_))
    }

    /// The gRPC [`tonic::Code`] if this is a [`Self::Status`], else `None`.
    pub fn status_code(&self) -> Option<tonic::Code> {
        match self {
            Self::Status(s) => Some(s.code()),
            _ => None,
        }
    }

    /// Borrow the underlying [`tonic::Status`] if this is a [`Self::Status`].
    pub fn as_status(&self) -> Option<&tonic::Status> {
        match self {
            Self::Status(s) => Some(s),
            _ => None,
        }
    }

    /// Convenience: true if the underlying status code is `Unavailable`.
    ///
    /// The most common transport-equivalent gRPC status for `quark-noded` —
    /// the daemon is down or unreachable.
    pub fn is_unavailable(&self) -> bool {
        self.status_code() == Some(tonic::Code::Unavailable)
    }

    /// Convenience: true if the underlying status code is `NotFound`.
    pub fn is_not_found(&self) -> bool {
        self.status_code() == Some(tonic::Code::NotFound)
    }

    /// Convenience: true if the underlying status code is `InvalidArgument`.
    pub fn is_invalid_argument(&self) -> bool {
        self.status_code() == Some(tonic::Code::InvalidArgument)
    }

    /// Convenience: true if the underlying status code is `DeadlineExceeded`.
    ///
    /// Returned by the daemon when an `Execute` RPC exceeds its `deadline_ms`.
    pub fn is_deadline_exceeded(&self) -> bool {
        self.status_code() == Some(tonic::Code::DeadlineExceeded)
    }
}

impl From<std::convert::Infallible> for NodeClientError {
    fn from(_: std::convert::Infallible) -> Self {
        // Infallible can never be constructed, so this is purely for API
        // ergonomics where a `?` needs a conversion target.
        NodeClientError::InvalidResponse("infallible".into())
    }
}
