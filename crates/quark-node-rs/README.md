# quark (internal: quark-node-rs)

Ergonomic, Supabase-style builder-pattern gRPC client SDK for the
**`quark-noded`** daemon.

`quark (internal: quark-node-rs)` is the primary client SDK. It wraps the generated tonic client
(`quark_node_proto::v1`) with typed convenience methods covering **all 7 RPCs**
of `NodeService` defined in [`proto/node.proto`](../../quark-rs/crates/quark-node-proto/proto/node.proto).

The daemon listens on `--grpc-addr` (default `0.0.0.0:50051`) and serves the
seven RPCs: `Execute`, `Cancel`, `Health`, `Ready`, `Status`, `Drain`,
`Shutdown`. See `docs/content/develop/specs/08-api.mdx` for the full API
specification and `docs/content/develop/adr/0001-why-grpc.mdx` for the
architectural rationale.

## Table of contents

- [Quick start](#quick-start)
- [The builder pattern](#the-builder-pattern)
- [Service clients](#service-clients)
  - [NodeService](#nodeservice) — execute, cancel, health, ready, status, drain, shutdown (7 RPCs)
- [Authentication](#authentication)
- [Error handling](#error-handling)
- [Design notes](#design-notes)

---

## Quick start

```toml
# Cargo.toml
[dependencies]
quark (internal: quark-node-rs) = { path = "../path/to/quark-rs/crates/quark (internal: quark-node-rs)" }
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::time::Duration;
use quark::node::NodeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = NodeClient::builder()
        .endpoint("http://127.0.0.1:50051")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(30))
        .build()
        .await?;

    // Liveness check.
    let health = client.node().health("", "v1").await?;
    println!("daemon status: {} (uptime {}ms)", health.status, health.uptime_ms);

    // Readiness check.
    let ready = client.node().ready("", "v1").await?;
    if !ready.ready {
        println!("daemon not ready: {}", ready.reason);
        return Ok(());
    }

    // Execute a node.
    let resp = client.node()
        .execute(
            "",
            "v1",
            "req-1",
            "test/echo/native/reverse:1.0.0",
            None,
            30_000,
            "request",
            "", "", "", "",
        )
        .await?;
    println!("execute status: {}", resp.status);

    Ok(())
}
```

---

## The builder pattern

[`NodeClient::builder()`] returns a [`NodeClientBuilder`] with a fluent,
consume-and-return-self API. Finalize with either:

- [`build()`](NodeClientBuilder::build) — eagerly connects (async).
- [`build_lazy()`](NodeClientBuilder::build_lazy) — defers the TCP connect
  until the first RPC (synchronous).

```rust,no_run
use std::time::Duration;
use quark::node::NodeClient;

# async fn t() -> Result<(), Box<dyn std::error::Error>> {
let eager = NodeClient::builder()
    .endpoint("http://127.0.0.1:50051")
    .connect_timeout(Duration::from_secs(5))
    .request_timeout(Duration::from_secs(30))
    .keepalive_interval(Duration::from_secs(30))
    .tcp_nodelay(true)
    .concurrency_limit(64)
    .build()
    .await?;

// Or, connect lazily:
let lazy = NodeClient::builder()
    .endpoint("http://127.0.0.1:50051")
    .build_lazy()?;
# Ok(()) }
```

There are two convenience constructors:

- [`NodeClient::connect(url)`](NodeClient::connect) — shorthand for
  `builder().endpoint(url).build().await`.
- [`NodeClient::from_channel(channel)`](NodeClient::from_channel) — wrap an
  existing tonic channel (e.g. one shared with another client, or a TLS-enabled
  channel you configured yourself).

The [`NodeClient`] holds a single multiplexed HTTP/2
[`tonic::transport::Channel`](tonic::transport::Channel). Every service
accessor (`node()`, …) **clones** the channel (cheap) and wraps the generated
tonic client, so service clients are created on demand and discarded freely.

---

## Service clients

Each service client exposes typed Rust parameters (not raw proto request
types), builds the proto request internally, attaches a `Bearer` token when the
RPC requires authentication, calls the generated tonic client, and returns the
proto response.

### NodeService

The gRPC API for the node execution daemon. **7 RPCs.**

```rust,no_run
# use quark::node::NodeClient;
# async fn t(client: &NodeClient) -> Result<(), Box<dyn std::error::Error>> {
let node = client.node();

// Liveness / readiness / status.
let health = node.health("", "v1").await?;
let ready = node.ready("", "v1").await?;
let status = node.status("", "v1").await?;
println!("host_id: {}, uptime: {}ms", status.host_id, status.uptime_ms);

// Execute a node. `input` is a `prost_types::Value`; pass `None` for no input.
let resp = node
    .execute(
        "",
        "v1",
        "req-1",
        "test/echo/native/reverse:1.0.0",
        None,
        30_000,
        "request",
        "", "", "", "",
    )
    .await?;
if resp.status == "ok" {
    println!("output: {:?}", resp.output);
} else if let Some(err) = &resp.error {
    eprintln!("error: {}: {}", err.code, err.message);
}

// Cancel an in-flight execution.
let cancel = node.cancel("", "v1", "req-1", "user-requested").await?;
println!("cancelled: {}", cancel.cancelled);

// Drain in-flight requests (wait up to 5 seconds).
let drained = node.drain("", "v1", 5_000).await?;
println!("drained {} requests", drained.drained);

// Shut down the daemon (force = false for graceful).
node.shutdown("", "v1", false).await?;
# Ok(()) }
```

---

## Authentication

`quark-noded` does **not** install a server-side auth interceptor — every RPC
is anonymous at the gRPC layer (the daemon relies on network ACLs / Unix
socket permissions for access control). The methods on this client still take a
`token: &str` first argument for two reasons:

1. **Forward-compatibility.** If the daemon (or a fronting gateway) ever
   enforces bearer auth, callers won't need to change their code — just pass a
   non-empty token.
2. **Gateway deployments.** When the daemon sits behind a gateway that injects
   identity, callers can pass their token through.

An empty `token` is silently skipped (no `Authorization` header is attached),
which is the default for direct daemon access.

---

## Error handling

Every method returns `Result<T, NodeClientError>`. `NodeClientError` has three
variants:

| Variant                  | When                                                                   |
|--------------------------|------------------------------------------------------------------------|
| `Transport(String)`      | Channel connect / URI parse / transport-level failure.                 |
| `Status(tonic::Status)`  | The daemon returned a gRPC error status (via `#[from]`).               |
| `InvalidResponse(String)`| The call succeeded but the response couldn't be interpreted.           |

`tonic::Status` converts automatically via `?`, so the common path is
ergonomic. Helper methods make status introspection concise:

```rust,no_run
use quark::node::{NodeClient, NodeClientError};
# async fn t(client: &NodeClient) -> Result<(), NodeClientError> {
match client.node().health("", "v1").await {
    Ok(h) => println!("status: {}", h.status),
    Err(e) if e.is_unavailable() => println!("daemon is down or unreachable"),
    Err(e) if e.is_deadline_exceeded() => println!("daemon timed out"),
    Err(e) => return Err(e),
}
# Ok(()) }
```

Available helpers: `is_transport`, `is_status`, `status_code`, `as_status`,
`is_unavailable`, `is_not_found`, `is_invalid_argument`,
`is_deadline_exceeded`.

---

## Design notes

- **1:1 proto coverage.** Every field of every request message is exposed as a
  typed Rust parameter — nothing is dropped or hardcoded. The proto's optional
  `input` (`google.protobuf.Value`) maps to `Option<prost_types::Value>`; the
  proto's optional `metadata` (`RequestMetadata`) maps to four `&str`
  parameters (`trace_id`, `span_id`, `caller_id`, `caller_ip`) — when all four
  are empty, no metadata is sent, preserving the proto's optional semantics.
- **No raw proto request types at the call site.** Callers pass Rust
  primitives, slices, and (for the input Value) `prost_types::Value`; the SDK
  builds the proto request internally.
- **Cheap service clients.** The accessor clones the shared `Channel`
  (HTTP/2-multiplexed), so `client.node()` is essentially free. Hold the
  returned service client for a sequence of calls, or call the accessor inline
  for one-offs.
- **Escape hatch.** `NodeService::inner()` exposes the underlying tonic client
  for direct access when you need something the wrapper doesn't surface.
