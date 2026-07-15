//! PolicyService — RBAC policies CRUD.
//!
//! Wraps `quark_auth_proto::auth::v1::policy_service_client::PolicyServiceClient`.
//!
//! Covers all 4 RPCs: `CreatePolicy`, `GetPolicy`, `ListPolicies`,
//! `DeletePolicy`. Every RPC requires a bearer token.

use quark_auth_proto::auth::v1::policy_service_client::PolicyServiceClient;
use tonic::transport::Channel;

use crate::error::AuthClientError;
use crate::services::attach_bearer;

/// Client for `PolicyService`.
pub struct PolicyService {
    inner: PolicyServiceClient<Channel>,
}

impl PolicyService {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: PolicyServiceClient::new(channel),
        }
    }

    /// Borrow the underlying tonic client (escape hatch).
    pub fn inner(&mut self) -> &mut PolicyServiceClient<Channel> {
        &mut self.inner
    }

    /// `CreatePolicy` — create an RBAC policy.
    ///
    /// `effect` is `"allow"` or `"deny"` (passed as a string per the proto
    /// request shape). `principals` is a slice of [`quark_auth_proto::auth::v1::Principal`]
    /// — construct them with the nested oneof, e.g.:
    ///
    /// ```no_run
    /// use quark_auth_rs::proto::Principal;
    /// use quark_auth_rs::proto::principal::Principal as PrincipalKind;
    /// let p = Principal {
    ///     principal: Some(PrincipalKind::Identity("user-123".into())),
    /// };
    /// # let _ = p;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &mut self,
        token: &str,
        name: &str,
        effect: &str,
        actions: &[&str],
        resources: &[&str],
        principals: &[quark_auth_proto::auth::v1::Principal],
        description: Option<&str>,
    ) -> Result<quark_auth_proto::auth::v1::Policy, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::CreatePolicyRequest {
            name: name.to_string(),
            effect: effect.to_string(),
            actions: actions.iter().map(|s| s.to_string()).collect(),
            resources: resources.iter().map(|s| s.to_string()).collect(),
            principals: principals.to_vec(),
            description: description.map(|s| s.to_string()),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.create_policy(req).await?;
        Ok(resp.into_inner())
    }

    /// `GetPolicy` — fetch a policy by ID.
    pub async fn get(
        &mut self,
        token: &str,
        id: &str,
    ) -> Result<quark_auth_proto::auth::v1::Policy, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::GetPolicyRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.get_policy(req).await?;
        Ok(resp.into_inner())
    }

    /// `ListPolicies` — paginated policy list.
    pub async fn list(
        &mut self,
        token: &str,
        limit: u32,
        offset: u32,
    ) -> Result<quark_auth_proto::auth::v1::ListPoliciesResponse, AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::ListPoliciesRequest {
            query: Some(quark_auth_proto::common::v1::PageQuery { limit, offset }),
        });
        attach_bearer(&mut req, token);
        let resp = self.inner.list_policies(req).await?;
        Ok(resp.into_inner())
    }

    /// `DeletePolicy` — delete a policy by ID.
    pub async fn delete(&mut self, token: &str, id: &str) -> Result<(), AuthClientError> {
        let mut req = tonic::Request::new(quark_auth_proto::auth::v1::DeletePolicyRequest {
            id: id.to_string(),
        });
        attach_bearer(&mut req, token);
        self.inner.delete_policy(req).await?;
        Ok(())
    }
}
