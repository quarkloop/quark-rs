# quark (internal: quark-workflow-rs)

A type-safe Rust client SDK for the [workflow-rs](https://github.com/quarkloop/workflow.rs) server.

## Features

- **Builder pattern only** — no shorthand methods, no convenience aliases
- **Type-safe workflow definitions** — `Workflow` trait with typed `Result`
- **Type-safe signals/queries/updates** — `SignalDef<Args>`, `QueryDef<Ret, Args>`
- **JSON DataConverter** with pluggable `DataConverter` trait
- **Full gRPC connection management** via tonic
- **Core operations**: start, signal, query, result, cancel, terminate, describe, fetchHistory, list, count
- **Namespace operations**: describe, list, register, delete
- **Error types** mapping gRPC status codes to typed SDK errors
- **Interceptor support** for auth/tracing

## Installation

```toml
[dependencies]
quark (internal: quark-workflow-rs) = { path = "../path/to/quark-rs/crates/quark (internal: quark-workflow-rs)" }
```

## Quick Start

```rust
use serde::{Deserialize, Serialize};
use quark::workflow::{Workflow, WorkflowClient};

#[derive(Serialize, Deserialize, Debug)]
struct OrderResult { order_id: String, status: String }

struct OrderWorkflow;
impl Workflow for OrderWorkflow {
    type Result = OrderResult;
    const WORKFLOW_TYPE: &'static str = "processOrder";
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WorkflowClient::builder()
        .address("http://localhost:7233")
        .build()
        .await?;

    let handle = client
        .workflow()
        .start::<OrderWorkflow>("processOrder")
        .workflow_id("order-123")
        .task_queue("orders")
        .args(vec![serde_json::json!({ "orderId": "123" })])
        .start()
        .await?;

    let result: OrderResult = handle.result().await?;
    println!("Result: {:?}", result);
    Ok(())
}
```

## Signals and Queries

```rust
use quark::workflow::{signal, query};

let increment = signal::<i64>("increment");
let get_value = query::<i64, ()>("getValue");

// Signal
handle.signal(increment, 42).await?;

// Query
let value: i64 = handle.query(get_value, ()).await?;
```

## Namespace Operations

```rust
use quark::workflow::{RegisterNamespaceRequest, WorkflowClient};
use std::time::Duration;

let client = WorkflowClient::builder()
    .address("http://localhost:7233")
    .build()
    .await?;

// Register
client.namespace().register(RegisterNamespaceRequest {
    name: "my-namespace".into(),
    description: "My namespace".into(),
    retention_period: Some(Duration::from_secs(86400)),
}).await?;

// Describe
let desc = client.namespace().describe_by_name("my-namespace").await?;

// List
let namespaces = client.namespace().list().await?;

// Delete
client.namespace().delete("my-namespace").await?;
```

## API Reference

### `WorkflowClient`

Top-level client. Created via `WorkflowClient::builder()`.

| Method | Returns |
|--------|---------|
| `builder()` | `WorkflowClientBuilder` |
| `workflow()` | `WorkflowOperations` |
| `namespace()` | `NamespaceOperations` |
| `connection()` | `&Connection` (for advanced use) |

### `WorkflowClientBuilder`

Fluent builder for `WorkflowClient`.

| Method | Type |
|--------|------|
| `address(addr)` | `String` |
| `namespace(ns)` | `String` |
| `identity(id)` | `String` |
| `data_converter(c)` | `Box<dyn DataConverter>` |
| `interceptor(i)` | `Arc<dyn Interceptor>` |
| `tls(t)` | `TlsConfig` |
| `timeout(d)` | `Duration` |
| `keepalive(d)` | `Duration` |
| `build()` | `Result<WorkflowClient>` |

### `WorkflowOperations`

| Method | Returns |
|--------|---------|
| `start::<T>(type)` | `WorkflowStartBuilder<T>` |
| `handle::<T>(id)` | `WorkflowHandle<T>` |
| `handle_with_run_id::<T>(id, run)` | `WorkflowHandle<T>` |
| `list(page_size)` | `Result<Vec<WorkflowExecutionInfo>>` |
| `count(query)` | `Result<u64>` |

### `WorkflowStartBuilder<T>`

| Method | Type |
|--------|------|
| `workflow_id(id)` | `impl Into<String>` (required) |
| `task_queue(q)` | `impl Into<String>` (required) |
| `args(values)` | `Vec<serde_json::Value>` |
| `workflow_run_timeout(d)` | `Duration` |
| `workflow_task_timeout(d)` | `Duration` |
| `search_attributes(a)` | `HashMap<String, Payload>` |
| `memo(m)` | `HashMap<String, Payload>` |
| `request_eager_start(b)` | `bool` |
| `start()` | `Result<WorkflowHandle<T>>` |
| `execute()` | `Result<T::Result>` |

### `WorkflowHandle<T>`

| Method | Returns |
|--------|---------|
| `result()` | `Result<T::Result>` |
| `signal(def, args)` | `Result<()>` |
| `query(def, args)` | `Result<R>` |
| `cancel()` | `Result<()>` |
| `terminate(reason)` | `Result<()>` |
| `describe()` | `Result<WorkflowExecutionDescription>` |
| `fetch_history()` | `Result<History>` |

### `NamespaceOperations`

| Method | Returns |
|--------|---------|
| `describe()` | `Result<NamespaceDescription>` |
| `describe_by_name(name)` | `Result<NamespaceDescription>` |
| `list()` | `Result<Vec<NamespaceDescription>>` |
| `register(req)` | `Result<()>` |
| `delete(name)` | `Result<()>` |

## License

MIT
