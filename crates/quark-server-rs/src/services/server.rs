//! ServerService — orchestration, service registry, admin API.
//!
//! Wraps `quark_server_proto::server::v1::server_service_client::ServerServiceClient`.
//!
//! Covers all 8 RPCs of `ServerService`:
//! `GetServiceRegistry`, `Deploy`, `Rollback`, `GetDeployment`,
//! `ListDeployments`, `ProvisionTenant`, `ListTenants`, `GetSystemHealth`.
//!
//! The server server installs a single [`AuthInterceptor`] on the entire
//! service (see `server/src/main.rs`), so **every RPC requires a valid bearer
//! token**. Each method on this client takes a `token: &str` first argument
//! and attaches it as `Authorization: Bearer …` gRPC metadata.
//!
//! [`AuthInterceptor`]: ../../server/src/interceptors/auth.rs

use quark_server_proto::common::v1::PageQuery;
use quark_server_proto::server::v1::server_service_client::ServerServiceClient;
use tonic::transport::Channel;

use crate::error::ServerClientError;
use crate::services::attach_bearer;

/// Client for `ServerService`.
pub struct ServerService {
    inner: ServerServiceClient<Channel>,
}

impl ServerService {
    /// Wrap a generated `ServerServiceClient` over a shared channel.
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: ServerServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch for advanced use).
    pub fn inner(&mut self) -> &mut ServerServiceClient<Channel> {
        &mut self.inner
    }

    // ─── Service registry ────────────────────────────────────────────────────

    /// `GetServiceRegistry` — fetch the cached service registry. Clients fetch
    /// once and cache locally; this is *not* a per-request discovery call.
    pub async fn get_service_registry(
        &mut self,
        token: &str,
    ) -> Result<quark_server_proto::server::v1::ServiceRegistry, ServerClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.get_service_registry(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Orchestration: deployments ──────────────────────────────────────────

    /// `Deploy` — start a deployment for the given release version, driving it
    /// through the named workflow. `input` is the optional workflow input
    /// payload (raw bytes — typically a serialized JSON or protobuf).
    pub async fn deploy(
        &mut self,
        token: &str,
        version_id: &str,
        workflow_id: &str,
        input: &[u8],
    ) -> Result<quark_server_proto::server::v1::Deployment, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::DeployRequest {
            version_id: version_id.to_string(),
            workflow_id: workflow_id.to_string(),
            input: input.to_vec(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.deploy(req).await?;
        Ok(resp.into_inner())
    }

    /// `Rollback` — roll back a deployment by ID. Returns the resulting
    /// `Deployment` (its status will move to `RolledBack`).
    pub async fn rollback(
        &mut self,
        token: &str,
        deployment_id: &str,
    ) -> Result<quark_server_proto::server::v1::Deployment, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::RollbackRequest {
            deployment_id: deployment_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.rollback(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetDeployment` — fetch a single deployment by ID.
    pub async fn get_deployment(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_server_proto::server::v1::Deployment, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::GetDeploymentRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_deployment(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListDeployments` — paginated list of deployments.
    ///
    /// Pass `Some(limit)` / `Some(offset)` to paginate; pass `None, None` to
    /// omit the page query entirely (the server decides what to return).
    pub async fn list_deployments(
        &mut self,
        token: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<quark_server_proto::server::v1::ListDeploymentsResponse, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::ListDeploymentsRequest {
            query: page_query(limit, offset),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_deployments(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Orchestration: provisioning ─────────────────────────────────────────

    /// `ProvisionTenant` — provision a brand-new tenant. Creates the
    /// corresponding auth-service organization, a default release-service
    /// artifact, and a bootstrap secrets-service secret. The returned `Tenant`
    /// carries the IDs of all three.
    pub async fn provision_tenant(
        &mut self,
        token: &str,
        org_name: &str,
        org_slug: &str,
        artifact_name: &str,
    ) -> Result<quark_server_proto::server::v1::Tenant, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::ProvisionTenantRequest {
            org_name: org_name.to_string(),
            org_slug: org_slug.to_string(),
            artifact_name: artifact_name.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.provision_tenant(req).await?;
        Ok(resp.into_inner())
    }

    // ─── Admin (requires "admin" role) ───────────────────────────────────────

    /// `ListTenants` — paginated list of all tenants. Requires the `admin`
    /// role on the bearer token.
    ///
    /// Pass `Some(limit)` / `Some(offset)` to paginate; pass `None, None` to
    /// omit the page query entirely.
    pub async fn list_tenants(
        &mut self,
        token: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<quark_server_proto::server::v1::ListTenantsResponse, ServerClientError> {
        let mut req = tonic::Request::new(quark_server_proto::server::v1::ListTenantsRequest {
            query: page_query(limit, offset),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_tenants(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetSystemHealth` — aggregate health of every platform service. Requires
    /// the `admin` role on the bearer token.
    pub async fn get_system_health(
        &mut self,
        token: &str,
    ) -> Result<quark_server_proto::server::v1::SystemHealth, ServerClientError> {
        let mut req = tonic::Request::new(());
        attach_bearer(&mut req, token);
        let resp = self.inner.get_system_health(req).await?;
        Ok(resp.into_inner())
    }
}

/// Build the optional `PageQuery` for the `List*` RPCs.
///
/// Returns `None` when neither limit nor offset is supplied, preserving the
/// proto's `optional` semantics. If either is `Some`, a `PageQuery` is sent
/// with the missing field defaulted to `0`.
fn page_query(limit: Option<u32>, offset: Option<u32>) -> Option<PageQuery> {
    match (limit, offset) {
        (None, None) => None,
        (l, o) => Some(PageQuery {
            limit: l.unwrap_or(0),
            offset: o.unwrap_or(0),
        }),
    }
}
