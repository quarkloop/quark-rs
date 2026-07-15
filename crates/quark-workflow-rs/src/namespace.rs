//! Namespace operations.
//!
/// Provides CRUD operations for namespaces via the `WorkflowService`
/// and `OperatorService` gRPC APIs.

use std::sync::Arc;
use std::time::Duration;

use quark_workflow_proto::temporal::api::operatorservice::v1 as ospb;
use quark_workflow_proto::temporal::api::workflowservice::v1 as wfspb;

use crate::connection::Connection;
use crate::errors::SdkError;
use crate::options::{NamespaceDescription, RegisterNamespaceRequest};

/// Namespace management operations.
pub struct NamespaceOperations<'a> {
    connection: &'a Arc<Connection>,
    namespace: &'a str,
}

impl<'a> NamespaceOperations<'a> {
    pub(crate) fn new(connection: &'a Arc<Connection>, namespace: &'a str) -> Self {
        Self {
            connection,
            namespace,
        }
    }

    /// Describe the current namespace.
    pub async fn describe(&self) -> Result<NamespaceDescription, SdkError> {
        self.describe_by_name(self.namespace).await
    }

    /// Describe a namespace by name.
    pub async fn describe_by_name(&self, name: &str) -> Result<NamespaceDescription, SdkError> {
        let response = self
            .connection
            .workflow_service()
            .describe_namespace(wfspb::DescribeNamespaceRequest {
                namespace: name.to_string(),
                ..Default::default()
            })
            .await?
            .into_inner();

        let info = response
            .namespace_info
            .ok_or_else(|| SdkError::Unexpected("missing namespace_info".into()))?;

        Ok(NamespaceDescription {
            name: info.name,
            id: info.id,
            state: info.state,
            description: info.description,
            retention_period: response.config.and_then(|c| c.workflow_execution_retention_ttl),
        })
    }

    /// List all namespaces.
    pub async fn list(&self) -> Result<Vec<NamespaceDescription>, SdkError> {
        let response = self
            .connection
            .workflow_service()
            .list_namespaces(wfspb::ListNamespacesRequest {
                page_size: 100,
                ..Default::default()
            })
            .await?
            .into_inner();

        let namespaces = response
            .namespaces
            .into_iter()
            .map(|desc| {
                let info = desc.namespace_info.unwrap_or_default();
                let config = desc.config.unwrap_or_default();
                NamespaceDescription {
                    name: info.name,
                    id: info.id,
                    state: info.state,
                    description: info.description,
                    retention_period: config.workflow_execution_retention_ttl,
                }
            })
            .collect();

        Ok(namespaces)
    }

    /// Register a new namespace.
    pub async fn register(&self, request: RegisterNamespaceRequest) -> Result<(), SdkError> {
        self.connection
            .workflow_service()
            .register_namespace(wfspb::RegisterNamespaceRequest {
                namespace: request.name,
                description: request.description,
                workflow_execution_retention_period: request.retention_period.map(|d| {
                    prost_types::Duration {
                        seconds: d.as_secs() as i64,
                        nanos: 0,
                    }
                }),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    /// Delete a namespace by name.
    pub async fn delete(&self, name: &str) -> Result<(), SdkError> {
        self.connection
            .operator_service()
            .delete_namespace(ospb::DeleteNamespaceRequest {
                namespace: name.to_string(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}
