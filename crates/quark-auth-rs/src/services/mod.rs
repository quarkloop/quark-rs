//! Per-service client wrappers.
//!
//! Each module wraps one generated tonic client (e.g. `AuthServiceClient`)
//! with typed, ergonomic convenience methods. The pattern is uniform across
//! every service:
//!
//! 1. Accept typed Rust parameters (not raw proto request types).
//! 2. Build the proto request internally.
//! 3. Attach a `Bearer` token to the gRPC metadata when the RPC requires
//!    authentication.
//! 4. Call the generated tonic client.
//! 5. Return the proto response (or `()` for `google.protobuf.Empty` returns),
//!    mapping `tonic::Status` → [`crate::AuthClientError`].

pub mod admin;
pub mod auth;
pub mod identity;
pub mod mfa;
pub mod oauth_server;
pub mod organization;
pub mod passkey;
pub mod policy;
pub mod project;
pub mod role;
pub mod sso;
pub mod user;
pub mod workspace;

use tonic::metadata::MetadataValue;
use tonic::Request;

/// Attach a `Bearer` token to a gRPC request's `authorization` metadata key.
///
/// No-op when `token` is empty — guards callers that don't have a token for a
/// nominally-authenticated RPC.
pub(crate) fn attach_bearer<T>(req: &mut Request<T>, token: &str) {
    if token.is_empty() {
        return;
    }
    if let Ok(value) = format!("Bearer {token}").parse::<MetadataValue<_>>() {
        req.metadata_mut().insert("authorization", value);
    }
}
