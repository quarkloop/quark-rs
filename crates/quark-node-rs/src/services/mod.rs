//! Per-service client wrappers.
//!
//! Each module wraps one generated tonic client (e.g. `NodeServiceClient`)
//! with typed, ergonomic convenience methods. The pattern is uniform across
//! every service:
//!
//! 1. Accept typed Rust parameters (not raw proto request types).
//! 2. Build the proto request internally.
//! 3. Attach a `Bearer` token to the gRPC metadata when the RPC requires
//!    authentication.
//! 4. Call the generated tonic client.
//! 5. Return the proto response, mapping `tonic::Status` →
//!    [`crate::NodeClientError`].
//!
//! # Authentication
//!
//! `quark-noded` does not currently install a server-side auth interceptor —
//! every RPC is anonymous at the gRPC layer. The methods on this client still
//! take a `token: &str` first argument for two reasons:
//!
//! 1. **Forward-compatibility.** If the daemon (or a fronting gateway) ever
//!    enforces bearer auth, callers won't need to change their code — just
//!    pass a non-empty token.
//! 2. **Gateway deployments.** When the daemon sits behind a gateway that
//!    injects identity, callers can pass their token through; an empty `token`
//!    is a no-op (no `Authorization` header is attached).

pub mod node;

use tonic::metadata::MetadataValue;
use tonic::Request;

/// Attach a `Bearer` token to a gRPC request's `authorization` metadata key.
///
/// No-op when `token` is empty — `quark-noded` does not enforce bearer auth,
/// so this is purely a passthrough for callers routing through a gateway (or
/// for future daemon builds that add an auth interceptor).
pub(crate) fn attach_bearer<T>(req: &mut Request<T>, token: &str) {
    if token.is_empty() {
        return;
    }
    if let Ok(value) = format!("Bearer {token}").parse::<MetadataValue<_>>() {
        req.metadata_mut().insert("authorization", value);
    }
}
