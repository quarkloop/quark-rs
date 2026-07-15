# Quark Rust SDK

Unified Rust client SDK for the Quark platform.

Licensed under the MIT License.

## Overview

This workspace contains proto definitions and ergonomic builder-pattern gRPC client SDKs for all Quark platform services: auth, server, node, and workflow. A unified facade crate (`quark-rs`) wraps all sub-clients behind a single `QuarkClient` with a `QuarkClientBuilder`.

## Crate Structure

```
quark-rs/
├── crates/
│   ├── quark-auth-proto/         Proto files + tonic/prost codegen for auth
│   ├── quark-auth-rs/            Builder-pattern gRPC client for auth (13 services, 115 RPCs)
│   ├── quark-server-proto/       Proto files + codegen for server
│   ├── quark-server-rs/          Builder-pattern gRPC client for server (1 service, 8 RPCs)
│   ├── quark-node-proto/         Proto files + codegen for node
│   ├── quark-node-rs/            Builder-pattern gRPC client for node (1 service, 7 RPCs)
│   ├── quark-workflow-proto/     Proto files + codegen for workflow (Temporal API)
│   ├── quark-workflow-rs/        Builder-pattern gRPC client for workflow
│   └── quark-rs/                 Unified facade (QuarkClient + QuarkClientBuilder)
├── examples/
│   ├── auth-example/             Auth client demo (login + list users)
│   ├── server-example/           Server client demo (service registry + health)
│   ├── node-example/             Node client demo (health + ready + status)
│   ├── workflow-example/         Workflow client demo (list namespaces)
│   └── unified-example/          Unified QuarkClient demo
├── Cargo.toml
├── LICENSE
└── README.md
```

## Naming Convention

| Pattern | Purpose |
|---|---|
| `quark-{service}-proto` | Proto definitions + tonic/prost codegen (no client logic) |
| `quark-{service}-rs` | Ergonomic builder-pattern gRPC client SDK |
| `quark-rs` | Unified facade wrapping all sub-clients |

## Design Principles

1. **Builder pattern** — Every client uses `Client::builder().endpoint(...).build().await?`
2. **1:1 proto coverage** — Every RPC has a typed Rust method, every proto field is exposed
3. **Bearer token attachment** — `attach_bearer` helper, no-op on empty token for anonymous RPCs
4. **Unified error type** — Each client has `{Service}ClientError { Transport, Status, InvalidResponse }`
5. **Single multiplexed channel** — tonic `Channel` cloned per service accessor (HTTP/2 multiplexed)
6. **SRP** — One crate per service, one module per proto service, one file per concern

## Service Coverage

| Service | Proto Crate | Client Crate | Services | RPCs |
|---|---|---|---|---|
| auth | `quark-auth-proto` | `quark-auth-rs` | 13 | 115 |
| server | `quark-server-proto` | `quark-server-rs` | 1 | 8 |
| node | `quark-node-proto` | `quark-node-rs` | 1 | 7 |
| workflow | `quark-workflow-proto` | `quark-workflow-rs` | 3 | ~96 |

## Usage

### Unified Client

```rust
use quark_rs::QuarkClient;
use std::time::Duration;

let client = QuarkClient::builder()
    .auth_endpoint("http://127.0.0.1:5001")
    .server_endpoint("http://127.0.0.1:3000")
    .node_endpoint("http://127.0.0.1:50051")
    .workflow_endpoint("http://127.0.0.1:7233")
    .connect_timeout(Duration::from_secs(5))
    .build()
    .await?;

// Access each service client
let login = client.auth()?.auth().login("user", "api-key").await?;
let registry = client.server()?.control_plane().get_service_registry(token).await?;
let health = client.node()?.node().health("", "v1").await?;
```

### Individual Clients

**Auth:**

```rust
use quark_auth_rs::AuthClient;

let client = AuthClient::builder()
    .endpoint("http://127.0.0.1:5001")
    .build()
    .await?;

let login = client.auth().login("user", "api-key").await?;
let users = client.users().list(&login.access_token, 50, 0, "").await?;
let factor = client.mfa().enroll_factor(&login.access_token, "totp").await?;
```

**Server:**

```rust
use quark_server_rs::ServerClient;

let client = ServerClient::builder()
    .endpoint("http://127.0.0.1:3000")
    .build()
    .await?;

let registry = client.control_plane().get_service_registry(token).await?;
let health = client.control_plane().get_system_health(token).await?;
```

**Node:**

```rust
use quark_node_rs::NodeClient;

let client = NodeClient::builder()
    .endpoint("http://127.0.0.1:50051")
    .build()
    .await?;

let health = client.node().health("", "v1").await?;
let status = client.node().status("", "v1").await?;
```

**Workflow:**

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
cargo run -p unified-example
```

## License

[MIT](LICENSE)
