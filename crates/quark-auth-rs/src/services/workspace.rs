//! WorkspaceService — workspaces CRUD + lifecycle (project-scoped).
//!
//! Wraps `quark_auth_proto::auth::v1::workspace_service_client::WorkspaceServiceClient`.
//!
//! Covers all 8 RPCs: `CreateWorkspace`, `GetWorkspace`, `ListWorkspaces`,
//! `UpdateWorkspace`, `ActivateWorkspace`, `DeactivateWorkspace`,
//! `ArchiveWorkspace`, `DeleteWorkspace`. Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::workspace_service_client::WorkspaceServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `WorkspaceService`.
pub struct WorkspaceService {
    inner: WorkspaceServiceClient<Channel>,
}

impl WorkspaceService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: WorkspaceServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut WorkspaceServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreateWorkspace` — create a workspace under a project.
    pub async fn create(
        &mut self,
        token: &str,
        project_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreateWorkspaceRequest {
            project_id: project_id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetWorkspace` — fetch a workspace by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetWorkspaceRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListWorkspaces` — paginated workspace list scoped to a project.
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
        project_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListWorkspacesResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListWorkspacesRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
            project_id: project_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_workspaces(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdateWorkspace` — patch a workspace's name/description.
    pub async fn update(
        &mut self,
        token: &str,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdateWorkspaceRequest {
            id: id.to_string(),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `ActivateWorkspace` — transition a workspace to active lifecycle.
    pub async fn activate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ActivateWorkspaceRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.activate_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeactivateWorkspace` — transition a workspace to inactive lifecycle.
    pub async fn deactivate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeactivateWorkspaceRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.deactivate_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `ArchiveWorkspace` — archive a workspace.
    pub async fn archive(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Workspace, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ArchiveWorkspaceRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.archive_workspace(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteWorkspace` — delete a workspace by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteWorkspaceRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_workspace(req).await?;
        Ok(())
    }
}
