//! ProjectService — projects CRUD + lifecycle (org-scoped).
//!
//! Wraps `quark_auth_proto::auth::v1::project_service_client::ProjectServiceClient`.
//!
//! Covers all 8 RPCs: `CreateProject`, `GetProject`, `ListProjects`,
//! `UpdateProject`, `ActivateProject`, `DeactivateProject`, `ArchiveProject`,
//! `DeleteProject`. Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::project_service_client::ProjectServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `ProjectService`.
pub struct ProjectService {
    inner: ProjectServiceClient<Channel>,
}

impl ProjectService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: ProjectServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut ProjectServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreateProject` — create a project under an organization.
    pub async fn create(
        &mut self,
        token: &str,
        organization_id: &str,
        name: &str,
        slug: &str,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreateProjectRequest {
            organization_id: organization_id.to_string(),
            name: name.to_string(),
            slug: slug.to_string(),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetProject` — fetch a project by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetProjectRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListProjects` — paginated project list scoped to an organization.
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
        organization_id: &str,
    ) -> Result<quark_auth_proto::auth::v1::ListProjectsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListProjectsRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
            organization_id: organization_id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_projects(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdateProject` — patch a project's name/description.
    pub async fn update(
        &mut self,
        token: &str,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdateProjectRequest {
            id: id.to_string(),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `ActivateProject` — transition a project to active lifecycle.
    pub async fn activate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ActivateProjectRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.activate_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeactivateProject` — transition a project to inactive lifecycle.
    pub async fn deactivate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeactivateProjectRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.deactivate_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `ArchiveProject` — archive a project.
    pub async fn archive(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Project, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ArchiveProjectRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.archive_project(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteProject` — delete a project by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteProjectRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_project(req).await?;
        Ok(())
    }
}
