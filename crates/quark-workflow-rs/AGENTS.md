# AGENTS.md — Rust Client SDK

## Project goal

A type-safe Rust client SDK for the workflow-rs server. Communicates
exclusively via gRPC using the tonic crate. Uses the builder pattern
exclusively — no shorthand methods.

## Architecture

The SDK is organized into modules with strict single-responsibility:

| Module | Responsibility |
|--------|---------------|
| `errors.rs` | `SdkError` enum, gRPC status code mapping |
| `converter.rs` | `DataConverter` trait, `JsonDataConverter`, payload helpers |
| `definitions.rs` | `SignalDef`, `QueryDef`, `UpdateDef` type-safe definitions |
| `options.rs` | Options structs, description types |
| `interceptors.rs` | `Interceptor` trait, `MetadataInterceptor` |
| `connection.rs` | `Connection` — tonic gRPC channel management |
| `workflow_trait.rs` | `Workflow` trait (type-safe result type) |
| `builder.rs` | `WorkflowStartBuilder` — fluent builder |
| `handle.rs` | `WorkflowHandle<T>` — workflow interaction |
| `client.rs` | `WorkflowClient`, `WorkflowClientBuilder`, `WorkflowOperations` |
| `namespace.rs` | `NamespaceOperations` — namespace CRUD |

## Rules

1. **Builder pattern only.** No shorthand methods. Every operation
   requires explicit configuration via the builder.
2. **No hacks, no shortcuts.** Every gRPC call maps errors to typed
   `SdkError` variants. No silent defaults.
3. **Type safety.** Workflows are defined via the `Workflow` trait.
   Signals, queries, and updates use typed definition objects with
   generics for compile-time argument checking.
4. **SRP.** Each module has exactly one responsibility. No cross-module
   coupling except through public APIs.
5. **Zero warnings.** The crate must compile with zero warnings.

## Dependencies

- `workflow-api` — proto types (path dependency)
- `tonic` — gRPC client
- `prost` — protobuf
- `serde` / `serde_json` — JSON data converter
- `thiserror` — error types
- `tokio` — async runtime
- `uuid` — workflow ID generation

## Build & test

```bash
cargo build -p workflow-sdk
cargo test  -p workflow-sdk
```

## Examples

Examples live in `../../examples/src/rust/` and are verified against a
running server:

- `start_and_result.rs` — Start a workflow, act as worker, get result
- `signal_and_query.rs` — Signal and query a workflow
- `namespace_ops.rs` — Namespace CRUD operations
