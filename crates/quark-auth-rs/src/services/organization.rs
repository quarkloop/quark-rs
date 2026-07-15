//! OrganizationService — organizations CRUD + lifecycle.
//!
//! Wraps `quark_auth_proto::auth::v1::organization_service_client::OrganizationServiceClient`.
//!
//! Covers all 8 RPCs: `CreateOrganization`, `GetOrganization`,
//! `ListOrganizations`, `UpdateOrganization`, `ActivateOrganization`,
//! `DeactivateOrganization`, `ArchiveOrganization`, `DeleteOrganization`.
//! Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::organization_service_client::OrganizationServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `OrganizationService`.
pub struct OrganizationService {
    inner: OrganizationServiceClient<Channel>,
}

impl OrganizationService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: OrganizationServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut OrganizationServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreateOrganization` — create a new organization.
    pub async fn create(
        &mut self,
        token: &str,
        name: &str,
        slug: &str,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreateOrganizationRequest {
            name: name.to_string(),
            slug: slug.to_string(),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetOrganization` — fetch an organization by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetOrganizationRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListOrganizations` — paginated organization list.
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
    ) -> Result<quark_auth_proto::auth::v1::ListOrganizationsResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListOrganizationsRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_organizations(req).await?;
        Ok(resp.into_inner())
    }

    /// `UpdateOrganization` — patch an organization's name/description.
    pub async fn update(
        &mut self,
        token: &str,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::UpdateOrganizationRequest {
            id: id.to_string(),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.update_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `ActivateOrganization` — transition an organization to active lifecycle.
    pub async fn activate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ActivateRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.activate_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeactivateOrganization` — transition an organization to inactive lifecycle.
    pub async fn deactivate(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeactivateRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.deactivate_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `ArchiveOrganization` — archive an organization.
    pub async fn archive(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Organization, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ArchiveRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.archive_organization(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeleteOrganization` — delete an organization by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeleteOrganizationRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_organization(req).await?;
        Ok(())
    }
}
