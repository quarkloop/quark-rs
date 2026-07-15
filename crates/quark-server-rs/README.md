# server-client

Ergonomic, Supabase-style builder-pattern gRPC client SDK for the
**server server**.

`server-client` is the primary client SDK. It wraps the generated tonic client
(`proto_gen::server::v1`) with typed convenience methods covering **all 8
RPCs** of `ServerService` defined in
[`proto/server.proto`](../proto/server.proto).

The server is *not* a gateway — client CRUD traffic never flows through
it. It exposes only orchestration (deploy, rollback, provision),
service-registry lookup, and admin/operator RPCs.

## Table of contents

- [Quick start](#quick-start)
- [The builder pattern](#the-builder-pattern)
- [Service clients](#service-clients)
  - [ServerService](#serverclient) — registry, deployments, provisioning, admin (8 RPCs)
- [Authentication](#authentication)
- [Error handling](#error-handling)
- [Design notes](#design-notes)

---

## Quick start

```toml
# Cargo.toml
[dependencies]
server-client = { path = "../path/to/server/client" }
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::time::Duration;
use server_client::ServerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ServerClient::builder()
        .endpoint("http://127.0.0.1:5000")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(30))
        .build()
        .await?;

    // Every server RPC requires a bearer token.
    let registry = client.server().get_service_registry("admin-token").await?;
    for svc in &registry.services {
        println!("{} -> {} ({})", svc.name, svc.grpc_url, svc.version);
    }

    // Provision a new tenant.
    let tenant = client
        .server()
        .provision_tenant("admin-token", "Acme", "acme", "default")
        .await?;
    println!("provisioned tenant: {}", tenant.id);

    Ok(())
}
```

---

## The builder pattern

[`ServerClient::builder()`] returns a [`ServerClientBuilder`] with a fluent,
consume-and-return-self API. Finalize with either:

- [`build()`](ServerClientBuilder::build) — eagerly connects (async).
- [`build_lazy()`](ServerClientBuilder::build_lazy) — defers the TCP connect
  until the first RPC (synchronous).

```rust,no_run
use std::time::Duration;
use server_client::ServerClient;

# async fn t() -> Result<(), Box<dyn std::error::Error>> {
let eager = ServerClient::builder()
    .endpoint("http://127.0.0.1:5000")
    .connect_timeout(Duration::from_secs(5))
    .request_timeout(Duration::from_secs(30))
    .keepalive_interval(Duration::from_secs(30))
    .tcp_nodelay(true)
    .concurrency_limit(64)
    .build()
    .await?;

// Or, connect lazily:
let lazy = ServerClient::builder()
    .endpoint("http://127.0.0.1:5000")
    .build_lazy()?;
# Ok(()) }
```

There are two convenience constructors:

- [`ServerClient::connect(url)`](ServerClient::connect) — shorthand for
  `builder().endpoint(url).build().await`.
- [`ServerClient::from_channel(channel)`](ServerClient::from_channel) — wrap an
  existing tonic channel (e.g. one shared with another client, or a TLS-enabled
  channel you configured yourself).

The [`ServerClient`] holds a single multiplexed HTTP/2
[`tonic::transport::Channel`](tonic::transport::Channel). Every service
accessor (`server()`, …) **clones** the channel (cheap) and wraps the
generated tonic client, so service clients are created on demand and discarded
freely.

---

## Service clients

Each service client exposes typed Rust parameters (not raw proto request
types), builds the proto request internally, attaches a `Bearer` token when the
RPC requires authentication, calls the generated tonic client, and returns the
proto response (or `()` for `google.protobuf.Empty` returns).

### ServerService

Orchestration, service registry, and admin/operator RPCs. **8 RPCs.** Every
RPC requires a bearer token (the server's `AuthInterceptor` is installed on
the entire service).

```rust,no_run
# use server_client::ServerClient;
# async fn t(client: &ServerClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let cp = client.server();

// Service registry.
let registry = cp.get_service_registry(token).await?;
for svc in &registry.services {
    println!("{} -> {} ({})", svc.name, svc.grpc_url, svc.version);
}

// Deploy a release version through a workflow.
let deployment = cp
    .deploy(token, "release-v1.2.3", "deploy-workflow", b"{\"region\":\"us-east\"}")
    .await?;
println!("deployment {} status {:?}", deployment.id, deployment.status);

// Roll it back if something went wrong.
let rolled = cp.rollback(token, &deployment.id).await?;

// Inspect / list.
let fetched = cp.get_deployment(token, &deployment.id).await?;
let page = cp.list_deployments(token, Some(50), Some(0)).await?;
println!("{} deployments total", page.deployments.len());

// Provision a new tenant.
let tenant = cp
    .provision_tenant(token, "Acme", "acme", "default")
    .await?;

// Admin operations — the token must carry the "admin" role.
let tenants = cp.list_tenants(token, Some(100), Some(0)).await?;
let health = cp.get_system_health(token).await?;
for svc in &health.services {
    println!("{}: healthy={} detail={:?}", svc.name, svc.healthy, svc.detail);
}
# Ok(()) }
```

---

## Authentication

Unlike `auth-service` (which mixes public and authenticated RPCs), the
server installs a single `AuthInterceptor` on the *entire*
`ServerService`. **Every RPC requires a valid bearer token.** Pass the
access token as the first `token: &str` argument to every method; the SDK
attaches it as `Authorization: Bearer …` gRPC metadata.

`ListTenants` and `GetSystemHealth` further require the `admin` role on the
token (enforced server-side by the handler / interceptor).

An empty `token` is silently skipped (the request goes out unauthenticated),
which is a safety net rather than a recommended pattern — the server will
reject it with `Unauthenticated`.

---

## Error handling

Every method returns `Result<T, ServerClientError>`. `ServerClientError` has
three variants:

| Variant                  | When                                                                   |
|--------------------------|------------------------------------------------------------------------|
| `Transport(String)`      | Channel connect / URI parse / transport-level failure.                 |
| `Status(tonic::Status)`  | The server returned a gRPC error status (via `#[from]`).               |
| `InvalidResponse(String)`| The call succeeded but the response couldn't be interpreted.           |

`tonic::Status` converts automatically via `?`, so the common path is
ergonomic. Helper methods make status introspection concise:

```rust,no_run
use server_client::{ServerClient, ServerClientError};
# async fn t(client: &ServerClient, token: &str) -> Result<(), ServerClientError> {
match client.server().get_deployment(token, "missing-id").await {
    Ok(d) => println!("deployment: {d:?}"),
    Err(e) if e.is_not_found() => println!("no such deployment"),
    Err(e) if e.is_unauthenticated() => println!("token expired — re-login"),
    Err(e) if e.is_permission_denied() => println!("need admin role"),
    Err(e) => return Err(e),
}
# Ok(()) }
```

Available helpers: `is_transport`, `is_status`, `status_code`, `as_status`,
`is_unauthenticated`, `is_not_found`, `is_permission_denied`,
`is_already_exists`.

---

## Design notes

- **1:1 proto coverage.** Every field of every request message is exposed as a
  typed Rust parameter — nothing is dropped or hardcoded. The proto's optional
  `PageQuery` on the `List*` RPCs maps to `Option<u32>` for `limit` and
  `offset`; passing `None, None` omits the page query entirely, preserving the
  proto's optional semantics.
- **No raw proto request types at the call site.** Callers pass Rust
  primitives and slices; the SDK builds the proto request internally.
- **Cheap service clients.** The accessor clones the shared `Channel`
  (HTTP/2-multiplexed), so `client.server()` is essentially free. Hold
  the returned service client for a sequence of calls, or call the accessor
  inline for one-offs.
- **`google.protobuf.Empty`.** RPCs that take `Empty` (`GetServiceRegistry`,
  `GetSystemHealth`) need no request argument; the SDK passes `()` and attaches
  the bearer metadata to an empty-body request.
- **Escape hatch.** `ServerService::inner()` exposes the underlying tonic
  client for direct access when you need something the wrapper doesn't surface.
