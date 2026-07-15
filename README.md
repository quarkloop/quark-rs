# Quark Rust SDK

Unified Rust client SDK for the Quarkloop platform.

**Licensed under the MIT License.**

## What's in this repo

```
quark-rs/
├── Cargo.toml                          workspace manifest
├── crates/
│   ├── quark-auth-proto/               proto files + tonic/prost codegen for auth service
│   ├── quark-auth-rs/                  ergonomic builder-pattern gRPC client for auth (13 services, 115 RPCs)
│   ├── quark-server-proto/             proto files + codegen for server service
│   ├── quark-server-rs/                builder-pattern gRPC client for server (1 service, 8 RPCs)
│   ├── quark-node-proto/               proto files + codegen for node service
│   ├── quark-node-rs/                  builder-pattern gRPC client for node (1 service, 7 RPCs)
│   ├── quark-workflow-proto/           proto files + codegen for workflow service (Temporal API)
│   └── quark-workflow-rs/              builder-pattern gRPC client for workflow
├── examples/
│   ├── auth-example/                   demonstrate auth client (login + list users)
│   ├── server-example/                 demonstrate server client (service registry + health)
│   ├── node-example/                   demonstrate node client (health + ready + status)
│   └── workflow-example/              demonstrate workflow client (list namespaces)
└── README.md
```

## Crate naming convention

| Crate | Purpose |
|---|---|
| `quark-{service}-proto` | Proto definitions + tonic/prost codegen (no client logic) |
| `quark-{service}-rs` | Ergonomic builder-pattern gRPC client SDK |

## Design principles

1. **Builder pattern everywhere** — every client uses `Client::builder().endpoint(...).build().await?`
2. **1:1 proto coverage** — every RPC has a typed Rust method, every proto field is exposed as a parameter
3. **Bearer token attachment** — `attach_bearer` helper, no-op on empty token (anonymous RPCs)
4. **Unified error type** — each client has `{Service}ClientError { Transport, Status, InvalidResponse }`
5. **Single multiplexed channel** — tonic `Channel` cloned per service accessor (cheap, HTTP/2 multiplexed)
6. **SRP** — one crate per service, one module per proto service, one file per concern

## Usage

### Auth client

```rust
use quark_auth_rs::AuthClient;

let client = AuthClient::builder()
    .endpoint("http://127.0.0.1:5001")
    .build()
    .await?;

// Login (anonymous RPC)
let login = client.auth().login("user", "api-key").await?;

// List users (authenticated)
let users = client.users().list(&login.access_token, 50, 0, "").await?;

// MFA enrollment
let factor = client.mfa().enroll_factor(&login.access_token, "totp").await?;
```

### Server client

```rust
use quark_server_rs::ServerClient;

let client = ServerClient::builder()
    .endpoint("http://127.0.0.1:3000")
    .build()
    .await?;

let registry = client.control_plane().get_service_registry(token).await?;
let health = client.control_plane().get_system_health(token).await?;
```

### Node client

```rust
use quark_node_rs::NodeClient;

let client = NodeClient::builder()
    .endpoint("http://127.0.0.1:50051")
    .build()
    .await?;

let health = client.node().health("", "v1").await?;
let status = client.node().status("", "v1").await?;
```

### Workflow client

```rust
use quark_workflow_rs::WorkflowClient;

let client = WorkflowClient::builder()
    .address("http://127.0.0.1:7233")
    .namespace("default")
    .identity("my-app")
    .build()
    .await?;

let handle = client.workflow()
    .start::<MyWorkflow>("my-workflow")
    .workflow_id("wf-123")
    .task_queue("tasks")
    .args(vec![serde_json::json!({"key": "value"})])
    .start()
    .await?;

let result = handle.result().await?;
```

## Build

```bash
# Prerequisites: Rust 1.75+, protoc 25.x
cargo build --workspace
```

## Examples

```bash
cargo run -p auth-example -- http://127.0.0.1:5001
cargo run -p server-example -- http://127.0.0.1:3000
cargo run -p node-example -- http://127.0.0.1:50051
cargo run -p workflow-example -- http://127.0.0.1:7233
```

## Service coverage

| Service | Proto crate | Client crate | Services | RPCs |
|---|---|---|---|---|
| auth | `quark-auth-proto` | `quark-auth-rs` | 13 | 115 |
| server | `quark-server-proto` | `quark-server-rs` | 1 | 8 |
| node | `quark-node-proto` | `quark-node-rs` | 1 | 7 |
| workflow | `quark-workflow-proto` | `quark-workflow-rs` | 3 | ~96 |
