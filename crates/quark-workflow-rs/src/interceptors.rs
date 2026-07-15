//! Request interceptor trait.
//!
//! Interceptors allow callers to modify outgoing gRPC requests before
//! they are sent — for example, adding authentication headers, tracing
//! metadata, or rate-limiting.

use crate::errors::SdkError;

/// Trait for intercepting outgoing gRPC requests.
///
/// Implementations can modify request metadata (headers) before the
/// request is sent to the server. Multiple interceptors can be chained.
///
/// This trait is currently used for type-level interception. The
/// `Connection` applies interceptors by passing them to tonic's
/// interceptor mechanism at the client level.
pub trait Interceptor: Send + Sync {
    /// Called before each gRPC request is sent. Return an error to
    /// abort the request.
    fn intercept(&self) -> Result<(), SdkError>;
}

/// An interceptor that adds a static set of metadata headers to every
/// request. Useful for API keys or auth tokens.
pub struct MetadataInterceptor {
    headers: Vec<(String, String)>,
}

impl MetadataInterceptor {
    /// Creates a new `MetadataInterceptor` with the given headers.
    pub fn new(headers: Vec<(String, String)>) -> Self {
        Self { headers }
    }

    /// Adds a header to the interceptor.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Returns the headers to be added to each request.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }
}

impl Interceptor for MetadataInterceptor {
    fn intercept(&self) -> Result<(), SdkError> {
        Ok(())
    }
}
