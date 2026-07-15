# quark-auth-rs

Ergonomic, Supabase-style builder-pattern gRPC client SDK for the
**auth-service**.

`quark-auth-rs` is the primary client SDK for talking to the auth-service over
gRPC. It wraps the generated tonic clients (`quark_auth_proto::auth::v1`) with
typed convenience methods covering **all 10 services** and **all 91 RPCs**
defined in [`proto/auth.proto`](../../auth/proto/auth.proto).

Organization / Project / Workspace services used to live in `platform.auth.v1`
but have been migrated to the server component (`platform.server.v1`). Use
`quark-server-rs` for those services. Role/Policy services remain here.

## Table of contents

- [Quick start](#quick-start)
- [The builder pattern](#the-builder-pattern)
- [Service clients](#service-clients)
  - [AuthService](#authservice) — login, signup, token, verify, magic link, OTP, JWKS, OIDC, external OAuth, invite (19 RPCs)
  - [UserService](#userservice) — user CRUD + role assignment (7 RPCs)
  - [IdentityService](#identityservice) — OAuth identity management (3 RPCs)
  - [MFAService](#mfaservice) — TOTP / phone / WebAuthn factors (5 RPCs)
  - [PasskeyService](#passkeyservice) — WebAuthn passkey auth + management (7 RPCs)
  - [SSOService](#ssoservice) — SAML SSO (3 RPCs)
  - [OAuthServerService](#oauthserverservice) — auth-service as OAuth2/OIDC provider (8 RPCs)
  - [AdminService](#adminservice) — admin operations (28 RPCs)
  - [RoleService](#roleservice) — roles + permission grants (7 RPCs)
  - [PolicyService](#policyservice) — RBAC policies (4 RPCs)
- [Authentication](#authentication)
- [Error handling](#error-handling)
- [Design notes](#design-notes)

---

## Quick start

```toml
# Cargo.toml
[dependencies]
quark-auth-rs = { path = "../path/to/quark-rs/crates/quark-auth-rs" }
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::time::Duration;
use quark_auth_rs::AuthClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AuthClient::builder()
        .endpoint("http://127.0.0.1:5001")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(30))
        .build()
        .await?;

    // Public auth flow — no bearer token required.
    let login = client.login("alice", "secret-api-key").await?;
    println!("access token: {}", login.access_token);

    // Authenticated call — pass the access token as a bearer.
    let me = client.users().get(&login.access_token, "user-uuid").await?;
    println!("hello, {}", me.email);

    Ok(())
}
```

---

## The builder pattern

[`AuthClient::builder()`] returns an [`AuthClientBuilder`] with a fluent,
consume-and-return-self API. Finalize with either:

- [`build()`](AuthClientBuilder::build) — eagerly connects (async).
- [`build_lazy()`](AuthClientBuilder::build_lazy) — defers the TCP connect until
  the first RPC (synchronous).

```rust,no_run
use std::time::Duration;
use quark_auth_rs::AuthClient;

# async fn t() -> Result<(), Box<dyn std::error::Error>> {
let eager = AuthClient::builder()
    .endpoint("http://127.0.0.1:5001")
    .connect_timeout(Duration::from_secs(5))
    .request_timeout(Duration::from_secs(30))
    .keepalive_interval(Duration::from_secs(30))
    .tcp_nodelay(true)
    .concurrency_limit(64)
    .build()
    .await?;

// Or, connect lazily:
let lazy = AuthClient::builder()
    .endpoint("http://127.0.0.1:5001")
    .build_lazy()?;
# Ok(()) }
```

There are two convenience constructors:

- [`AuthClient::connect(url)`](AuthClient::connect) — shorthand for
  `builder().endpoint(url).build().await`.
- [`AuthClient::from_channel(channel)`](AuthClient::from_channel) — wrap an
  existing tonic channel (e.g. one shared with another client, or a TLS-enabled
  channel you configured yourself).

The [`AuthClient`] holds a single multiplexed HTTP/2
[`tonic::transport::Channel`](tonic::transport::Channel). Every service accessor
(`auth()`, `users()`, …) **clones** the channel (cheap) and wraps the generated
tonic client, so service clients are created on demand and discarded freely.

---

## Service clients

Each service client exposes typed Rust parameters (not raw proto request types),
builds the proto request internally, attaches a `Bearer` token when the RPC
requires authentication, calls the generated tonic client, and returns the proto
response (or `()` for `google.protobuf.Empty` returns).

### AuthService

Authentication entry points — login, signup, token, verify, magic link, OTP,
recovery, resend, reauthenticate, settings, JWKS, OIDC discovery, health,
external OAuth redirect/callback, invite. **19 RPCs.**

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient) -> Result<(), Box<dyn std::error::Error>> {
// API-key login (no bearer).
let login = client.login("alice", "api-key").await?;

// Refresh the access token.
let refreshed = client.refresh(&login.refresh_token).await?;

// Signup with user metadata.
let signup = client.signup("bob@example.com", "", "p@ss", &[("team", "core")], "", "", "")
    .await?;

// Token endpoint — password grant.
let tok = client.token("password", "bob@example.com", "p@ss", "", "", "", "", "", "", "", "", "")
    .await?;

// Verify a signup token.
let verified = client.verify("signup", "the-otp", "", "", "", "")
    .await?;

// Logout (revoke the refresh token). scope = "" → server defaults to "global".
client.logout(&login.refresh_token, "").await?;

// Public endpoints.
let settings = client.get_settings().await?;
let jwks = client.get_jwks().await?;
let oidc = client.get_open_id_configuration().await?;
let health = client.health().await?;

// External OAuth (anonymous — no bearer).
let redir = client.external_provider_redirect("github", "", "", "", "", "")
    .await?;
let tokens = client.external_provider_callback("the-code", "the-state", "github")
    .await?;

// Invite (requires a bearer token).
let invited = client.invite(&login.access_token, "carol@example.com", "{}").await?;
# Ok(()) }
```

### UserService

User CRUD + role assignment. **7 RPCs.** Every RPC requires a bearer token.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let user = client.users()
    .create(token, "org-1", "alice", "alice@example.com", "Alice", Some("api-key"))
    .await?;

let fetched = client.users().get(token, &user.id).await?;

let page = client.users().list(token, 50, 0, "org-1").await?;

let updated = client.users()
    .update(token, &user.id, Some("Alice Q."), None, None, None, None, None, None, None, None)
    .await?;

let with_role = client.users().assign_role(token, &user.id, "role-admin").await?;
let without = client.users().revoke_role(token, &user.id, "role-admin").await?;

client.users().delete(token, &user.id).await?;
# Ok(()) }
```

### IdentityService

OAuth identity management for the authenticated user. **3 RPCs.** All require a
bearer token.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let identities = client.identities().list(token).await?;

let redirect = client.identities()
    .link(token, "github", "", "https://app.example.com/cb", "", "")
    .await?;

client.identities().delete(token, "identity-id").await?;
# Ok(()) }
```

### MFAService

TOTP / phone / WebAuthn factors. **5 RPCs.** All require a bearer token.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let enrolled = client.mfa()
    .enroll(token, "My TOTP", "totp", "auth-service", "")
    .await?;
let factor_id = &enrolled.id;

let challenge = client.mfa().challenge(token, factor_id, "sms").await?;
let verified = client.mfa()
    .verify(token, factor_id, &challenge.id, "123456")
    .await?;

let factors = client.mfa().list(token).await?;
client.mfa().unenroll(token, factor_id).await?;
# Ok(()) }
```

### PasskeyService

WebAuthn passkey authentication + management. **7 RPCs.**

- Anonymous (no bearer): `authentication_options`, `authentication_verify`.
- Authenticated: `registration_options`, `registration_verify`, `list`,
  `update`, `delete`.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
// Login flow (anonymous).
let opts = client.passkeys().authentication_options("alice@example.com").await?;
let tokens = client.passkeys().authentication_verify("{...assertion...}").await?;

// Management (authenticated).
let reg_opts = client.passkeys().registration_options(token).await?;
let passkey = client.passkeys()
    .registration_verify(token, "Yubikey", "{...attestation...}")
    .await?;
let listed = client.passkeys().list(token).await?;
let renamed = client.passkeys().update(token, &passkey.id, "Home key").await?;
client.passkeys().delete(token, &passkey.id).await?;
# Ok(()) }
```

### SSOService

SAML SSO entry points. **3 RPCs.** All anonymous (part of the browser SSO flow).

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient) -> Result<(), Box<dyn std::error::Error>> {
let redirect = client.sso()
    .redirect("", "example.com", "https://app.example.com/cb", "", "")
    .await?;
let tokens = client.sso()
    .acs("base64-saml-response", "relay-state")
    .await?;
let metadata_xml = client.sso().metadata().await?;
# Ok(()) }
```

### OAuthServerService

auth-service acting as an OAuth2/OIDC provider. **8 RPCs.**

- Anonymous: `authorize`, `token`, `user_info`, `client_dynamic_register`.
- Authenticated: `get_authorization`, `consent`, `list_grants`, `revoke_grant`.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
// Public OAuth2 endpoints.
let redirect = client.oauth_server()
    .authorize("code", "client-1", "https://app.example.com/cb", "openid profile", "xyz", "", "", "")
    .await?;
let tok = client.oauth_server()
    .token("authorization_code", "the-code", "https://app.example.com/cb", "client-1", "secret", "verifier", "")
    .await?;
let info = client.oauth_server().user_info().await?;
let client_obj = client.oauth_server()
    .client_dynamic_register(&["https://app.example.com/cb"], "My App", &["openid"], "client_secret_post")
    .await?;

// Authenticated resource-owner endpoints.
let authz = client.oauth_server().get_authorization(token, "authz-id").await?;
let consented = client.oauth_server().consent(token, "authz-id", true).await?;
let grants = client.oauth_server().list_grants(token).await?;
client.oauth_server().revoke_grant(token, "grant-id").await?;
# Ok(()) }
```

### AdminService

Admin-only operations. **28 RPCs.** Every RPC requires an admin bearer token.

Covers user management, factor management, passkey management, audit logs, SSO
provider management, OAuth client management, and custom OAuth provider
management.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, admin_token: &str) -> Result<(), Box<dyn std::error::Error>> {
// Users.
let users = client.admin().list_users(admin_token, 50, 0, "created_at", "").await?;
let created = client.admin()
    .create_user(admin_token, "dave@example.com", "", "p@ss", "member", "{}", "", true, false, "")
    .await?;
let fetched = client.admin().get_user(admin_token, &created.id).await?;
let updated = client.admin()
    .update_user(admin_token, &created.id, None, None, None, Some("admin"), None, None, None, None, None)
    .await?;
client.admin().delete_user(admin_token, &created.id, true).await?;
let link = client.admin()
    .generate_link(admin_token, "invite", "dave@example.com", "", "{}", "https://app.example.com/welcome")
    .await?;

// Factors & passkeys.
let factors = client.admin().list_user_factors(admin_token, "user-id").await?;
client.admin().delete_factor(admin_token, "user-id", "factor-id").await?;
client.admin().update_factor(admin_token, "user-id", "factor-id", true, "Renamed").await?;
let passkeys = client.admin().list_passkeys(admin_token, "user-id").await?;
client.admin().delete_passkey(admin_token, "user-id", "passkey-id").await?;

// Audit logs.
let logs = client.admin().list_audit_logs(admin_token, 100, 0).await?;

// SSO providers.
let sso = client.admin().list_sso_providers(admin_token).await?;
let provider = client.admin()
    .create_sso_provider(admin_token, "Corp IdP", "saml", "https://idp.example.com/metadata", "", "{}", &["example.com"])
    .await?;
client.admin().delete_sso_provider(admin_token, &provider.id).await?;

// OAuth clients.
let oc = client.admin()
    .oquark_auth_rs_register(admin_token, "My OAuth App", &["https://app.example.com/cb"], &["openid"])
    .await?;
let ocs = client.admin().oquark_auth_rs_list(admin_token).await?;
let rotated = client.admin().oquark_auth_rs_regenerate_secret(admin_token, &oc.id).await?;
client.admin().oquark_auth_rs_delete(admin_token, &oc.id).await?;

// Custom OAuth providers.
let custom = client.admin()
    .create_custom_oauth_provider(admin_token, "google", "oidc", "client-id", "secret", "https://app.example.com/cb", "https://accounts.google.com", "openid email", true)
    .await?;
let customs = client.admin().list_custom_oauth_providers(admin_token, "oidc").await?;
client.admin().delete_custom_oauth_provider(admin_token, "google").await?;
# Ok(()) }
```

### RoleService

Roles + permission grants (org-scoped). **7 RPCs.** All require a bearer token.

```rust,no_run
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let role = client.roles()
    .create(token, "org-1", "editor", Some("Can edit content"))
    .await?;
let fetched = client.roles().get(token, &role.id).await?;
let page = client.roles().list(token, 50, 0, "org-1").await?;
let updated = client.roles().update(token, &role.id, Some("Editor"), None).await?;
let granted = client.roles().grant_permission(token, &role.id, "content:write").await?;
let revoked = client.roles().revoke_permission(token, &role.id, "content:write").await?;
client.roles().delete(token, &role.id).await?;
# Ok(()) }
```

### PolicyService

RBAC policies. **4 RPCs.** All require a bearer token.

```rust,no_run
use quark_auth_rs::proto::{Principal, principal::Principal as PrincipalKind};
# use quark_auth_rs::AuthClient;
# async fn t(client: &AuthClient, token: &str) -> Result<(), Box<dyn std::error::Error>> {
let principals = vec![Principal { principal: Some(PrincipalKind::Identity("user-123".into())) }];
let policy = client.policies()
    .create(token, "allow-read", "allow", &["content:read"], &["content/*"], &principals, Some("Read access"))
    .await?;
let fetched = client.policies().get(token, &policy.id).await?;
let page = client.policies().list(token, 50, 0).await?;
client.policies().delete(token, &policy.id).await?;
# Ok(()) }
```

---

## Authentication

RPCs split into two categories, mirroring the auth-service interceptors:

- **Anonymous (public)** — no bearer token. These are the entry-point flows:
  `login`, `refresh`, `logout`, `verify_token`, `signup`, `token`, `verify`,
  `magic_link`, `otp`, `recover`, `resend`, `get_settings`, `get_jwks`,
  `get_open_id_configuration`, `health`, `external_provider_redirect`,
  `external_provider_callback`, `passkey_authentication_options`,
  `passkey_authentication_verify`, `sso_redirect`, `samlacs`, `saml_metadata`,
  `o_auth_server_authorize`, `o_auth_server_token`, `o_auth_server_user_info`,
  `o_auth_server_client_dynamic_register`.
- **Authenticated** — every other RPC. Pass the access token as the first
  `token: &str` argument; the SDK attaches it as `Authorization: Bearer …` gRPC
  metadata.

An empty `token` is silently skipped (the request goes out unauthenticated),
which is a safety net rather than a recommended pattern.

---

## Error handling

Every method returns `Result<T, AuthClientError>`. `AuthClientError` has three
variants:

| Variant                  | When                                                                   |
|--------------------------|------------------------------------------------------------------------|
| `Transport(String)`      | Channel connect / URI parse / transport-level failure.                 |
| `Status(tonic::Status)`  | The server returned a gRPC error status (via `#[from]`).               |
| `InvalidResponse(String)`| The call succeeded but the response couldn't be interpreted.           |

`tonic::Status` converts automatically via `?`, so the common path is
ergonomic. Helper methods make status introspection concise:

```rust,no_run
use quark_auth_rs::{AuthClient, AuthClientError};
# async fn t(client: &AuthClient, token: &str) -> Result<(), AuthClientError> {
match client.users().get(token, "missing-id").await {
    Ok(u) => println!("user: {u:?}"),
    Err(e) if e.is_not_found() => println!("no such user"),
    Err(e) if e.is_unauthenticated() => println!("token expired — re-login"),
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
  typed Rust parameter — nothing is dropped or hardcoded. Optional proto fields
  map to `Option<&str>` / `Option<bool>`; `repeated string` maps to `&[&str]`;
  `map<string,string>` maps to `&[(&str, &str)]`.
- **No raw proto request types at the call site.** Callers pass Rust primitives
  and slices; the SDK builds the proto request internally. (The one exception is
  `Principal` for `create_policy`, which carries a oneof and is necessarily a
  proto message — it's re-exported at `quark_auth_rs::proto::Principal`.)
- **Cheap service clients.** Each accessor clones the shared `Channel`
  (HTTP/2-multiplexed), so `client.users()`, `client.mfa()`, … are essentially
  free. Hold the returned service client for a sequence of calls, or call the
  accessor inline for one-offs.
- **`google.protobuf.Empty`.** RPCs that take `Empty` need no request argument;
  RPCs that return `Empty` yield `()`.
- **Escape hatch.** Each service client exposes `inner()` for direct access to
  the underlying tonic client when you need something the wrapper doesn't
  surface.
