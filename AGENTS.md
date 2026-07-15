# AGENTS.md — Operating Contract for AI Agents Working on quark-rs

This file is the authoritative source of truth for any AI agent (or human
contributor) that opens this repository with the intent to modify code. Read it
before touching anything; re-read it before committing.

## 1. The Hard Red Line

**NO SIMPLIFICATION. NO COSMETICS. NO HACKS. NO SHORTCUTS. NO MOCK. NO MVP.**

Every RPC method must be a faithful wrapper of its proto counterpart. Every
proto field must be exposed as a typed parameter. No fields dropped, no
branches skipped, no "close enough" approximations.

## 2. What This Repository Is

quark-rs is the **unified Rust client SDK** for the Quarkloop platform. It
provides ergonomic, builder-pattern gRPC client crates for four services:

| Service | Proto crate | Client crate | Services | RPCs |
|---|---|---|---|---|
| auth | `quark-auth-proto` | `quark-auth-rs` | 13 | 115 |
| server | `quark-server-proto` | `quark-server-rs` | 1 | 8 |
| node | `quark-node-proto` | `quark-node-rs` | 1 | 7 |
| workflow | `quark-workflow-proto` | `quark-workflow-rs` | 3 | ~96 |

The unified facade crate `quark-rs` wraps all four sub-clients behind a single
`QuarkClient` with a `QuarkClientBuilder`.

## 3. Workspace Layout

```
quark-rs/
├── crates/
│   ├── quark-auth-proto/         proto files + tonic/prost codegen for auth
│   ├── quark-auth-rs/            builder-pattern gRPC client for auth
│   ├── quark-server-proto/       proto files + codegen for server
│   ├── quark-server-rs/          builder-pattern gRPC client for server
│   ├── quark-node-proto/         proto files + codegen for node
│   ├── quark-node-rs/            builder-pattern gRPC client for node
│   ├── quark-workflow-proto/     Temporal API protos + codegen for workflow
│   ├── quark-workflow-rs/        builder-pattern gRPC client for workflow
│   └── quark-rs/                 unified facade (QuarkClient + QuarkClientBuilder)
├── examples/
│   ├── auth-example/             demonstrate auth client
│   ├── server-example/           demonstrate server client
│   ├── node-example/             demonstrate node client
│   ├── workflow-example/         demonstrate workflow client
│   └── unified-example/          demonstrate unified QuarkClient
├── Cargo.toml                    workspace manifest
├── LICENSE                       MIT
└── README.md
```

## 4. Crate Naming Convention

| Pattern | Purpose |
|---|---|
| `quark-{service}-proto` | Proto definitions + tonic/prost codegen (no client logic) |
| `quark-{service}-rs` | Ergonomic builder-pattern gRPC client SDK |
| `quark-rs` | Unified facade wrapping all sub-clients |

## 5. Design Principles

1. **Builder pattern everywhere** — `Client::builder().endpoint(...).build().await?`
2. **1:1 proto coverage** — every RPC has a typed Rust method, every proto field is exposed
3. **Bearer token attachment** — `attach_bearer` helper, no-op on empty token
4. **Unified error type** — `{Service}ClientError { Transport, Status, InvalidResponse }`
5. **Single multiplexed channel** — tonic `Channel` cloned per service accessor
6. **SRP** — one crate per service, one module per proto service, one file per concern
7. **Re-exports** — proto types re-exported as `pub use ... as proto;` so callers never depend on proto crates directly

## 6. Build & Test Commands

```bash
# Prerequisites: Rust 1.75+, protoc 25.x
export PROTOC="$HOME/.local/bin/protoc"
export PROTOC_INCLUDE="/tmp/protoc-install/include"

# Sanity check — must finish clean
cargo check --workspace

# Lint — must be 0 warnings
cargo clippy --workspace --no-deps

# Tests
cargo test --workspace

# Run an example
cargo run -p auth-example -- http://127.0.0.1:5001
```

## 7. Audit Workflow

When asked to audit:

1. Read the proto file(s) for the service in question.
2. Read the corresponding client crate's service module(s).
3. Compare every RPC method signature against the proto definition.
4. Verify every proto field is exposed as a typed parameter.
5. Verify bearer token attachment on authenticated RPCs.
6. Verify error handling maps `tonic::Status` correctly.
7. Fix every discrepancy — no exceptions, no "good enough".

## 8. Commit Conventions

- Start with a verb in imperative mood: "Add ...", "Fix ...", "Audit ...".
- Reference the proto service/RPC being ported or audited.
- State test results: "cargo check clean, 0 clippy warnings, N tests passed."

## 9. What You May NOT Do

- Do not skip an RPC method — all must be wrapped.
- Do not drop a proto field from a method signature.
- Do not use `unwrap()` or `expect()` in client code — return `Result`.
- Do not add `#[ignore]` to a passing test.
- Do not add `// TODO` instead of implementing — the code is the implementation.
- Do not simplify the builder — all 9 transport knobs must be present.

## 10. Commit Message Format

All commit messages must follow this format:

```
{type}: {message}
```

- All lowercase.
- `{type}` is one of: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `build`, `ci`.
- `{message}` is a concise, imperative-mood description.
- No period at the end.
- Example: `feat: add unified quark-rs facade crate`
- Example: `fix: rename proto-gen crates to avoid lockfile collisions`
- Example: `docs: add agents.md with commit message format`
