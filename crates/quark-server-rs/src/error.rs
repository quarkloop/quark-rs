//! Error type for the server client SDK.

use thiserror::Error;

/// All errors returned by [`crate::ServerClient`] service methods.
///
/// Three broad categories:
/// - [`ServerClientError::Transport`] — connection / transport-level failures
///   (DNS, TCP, TLS, channel closure, malformed endpoint URI, …).
/// - [`ServerClientError::Status`] — a gRPC call reached the server but was
///   rejected with a non-OK [`tonic::Status`] (e.g. `Unauthenticated`,
///   `NotFound`, `PermissionDenied`).
/// - [`ServerClientError::InvalidResponse`] — the call succeeded but the
///   response could not be interpreted as expected (decoding / shape mismatch).
#[derive(Debug, Error)]
#[allow(clippy::result_large_err)]
pub enum ServerClientError {
    /// A transport-level failure: the RPC never reached a successful gRPC
    /// exchange. Includes channel connect errors and invalid endpoint URIs.
    #[error("gRPC transport error: {0}")]
    Transport(String),

    /// The server returned a gRPC error status.
    #[error("gRPC status: {0}")]
    Status(#[from] tonic::Status),

    /// The response was received but could not be interpreted (e.g. a missing
    /// required field, malformed payload, unexpected empty body).
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

impl ServerClientError {
    /// True if this error is a transport-level failure.
    pub fn is_transport(&self) -> bool {
        matches!(self, Self::Transport(_))
    }

    /// True if this error is a gRPC status returned by the server.
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

    /// Convenience: true if the underlying status code is `Unauthenticated`.
    pub fn is_unauthenticated(&self) -> bool {
        self.status_code() == Some(tonic::Code::Unauthenticated)
    }

    /// Convenience: true if the underlying status code is `NotFound`.
    pub fn is_not_found(&self) -> bool {
        self.status_code() == Some(tonic::Code::NotFound)
    }

    /// Convenience: true if the underlying status code is `PermissionDenied`.
    pub fn is_permission_denied(&self) -> bool {
        self.status_code() == Some(tonic::Code::PermissionDenied)
    }

    /// Convenience: true if the underlying status code is `AlreadyExists`.
    pub fn is_already_exists(&self) -> bool {
        self.status_code() == Some(tonic::Code::AlreadyExists)
    }
}

impl From<std::convert::Infallible> for ServerClientError {
    fn from(_: std::convert::Infallible) -> Self {
        // Infallible can never be constructed, so this is purely for API
        // ergonomics where a `?` needs a conversion target.
        ServerClientError::InvalidResponse("infallible".into())
    }
}
