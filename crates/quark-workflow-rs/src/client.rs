//! Top-level workflow client.
//!
//! The `WorkflowClient` is the entry point for all SDK operations.
//! It wraps a gRPC `Connection` and provides typed access to
//! workflow and namespace operations via the builder pattern.

use std::sync::Arc;

use quark_workflow_proto::temporal::api::workflowservice::v1 as wfspb;

use crate::builder::WorkflowStartBuilder;
use crate::connection::Connection;
use crate::options::ConnectionOptions as ConnOpts;
use crate::converter::{DataConverter, JsonDataConverter};
use crate::errors::SdkError;
use crate::handle::WorkflowHandle;
use crate::interceptors::Interceptor;
use crate::namespace::NamespaceOperations;
use crate::options::{WorkflowExecutionInfo, TlsConfig};
use crate::workflow_trait::Workflow;

/// The top-level SDK client.
///
/// Created via `WorkflowClient::builder()`. Holds a gRPC connection
/// and a `DataConverter` for serializing values to payloads.
pub struct WorkflowClient {
    connection: Arc<Connection>,
    converter: Arc<dyn DataConverter>,
    namespace: String,
    identity: String,
    interceptors: Vec<Arc<dyn Interceptor>>,
}

impl WorkflowClient {
    /// Returns a builder for constructing a `WorkflowClient`.
    pub fn builder() -> WorkflowClientBuilder {
        WorkflowClientBuilder::default()
    }

    /// Access workflow operations (start, handle, list, count).
    pub fn workflow(&self) -> WorkflowOperations<'_> {
        WorkflowOperations {
            client: self,
        }
    }

    /// Access namespace operations (describe, list, register, delete).
    pub fn namespace(&self) -> NamespaceOperations<'_> {
        NamespaceOperations::new(&self.connection, &self.namespace)
    }

    /// Returns a reference to the underlying gRPC connection.
    ///
    /// This is primarily for testing and advanced use cases where
    /// direct gRPC access is needed (e.g., manually polling WFTs).
    pub fn connection(&self) -> &Arc<Connection> {
        &self.connection
    }
}

/// Builder for `WorkflowClient`. Uses the builder pattern exclusively.
pub struct WorkflowClientBuilder {
    address: Option<String>,
    namespace: String,
    identity: String,
    converter: Option<Box<dyn DataConverter>>,
    interceptors: Vec<Arc<dyn Interceptor>>,
    tls: Option<TlsConfig>,
    timeout: Option<std::time::Duration>,
    keepalive: Option<std::time::Duration>,
}

impl Default for WorkflowClientBuilder {
    fn default() -> Self {
        Self {
            address: None,
            namespace: "default".to_string(),
            identity: format!("{}@workflow-sdk", std::process::id()),
            converter: None,
            interceptors: Vec::new(),
            tls: None,
            timeout: None,
            keepalive: None,
        }
    }
}

impl WorkflowClientBuilder {
    /// Set the server address (e.g., `http://localhost:7233`).
    pub fn address(mut self, addr: impl Into<String>) -> Self {
        self.address = Some(addr.into());
        self
    }

    /// Set the namespace.
    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = ns.into();
        self
    }

    /// Set the client identity reported to the server.
    pub fn identity(mut self, id: impl Into<String>) -> Self {
        self.identity = id.into();
        self
    }

    /// Set a custom data converter.
    pub fn data_converter(mut self, converter: Box<dyn DataConverter>) -> Self {
        self.converter = Some(converter);
        self
    }

    /// Add a request interceptor.
    pub fn interceptor(mut self, interceptor: Arc<dyn Interceptor>) -> Self {
        self.interceptors.push(interceptor);
        self
    }

    /// Set TLS configuration.
    pub fn tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the keepalive interval.
    pub fn keepalive(mut self, keepalive: std::time::Duration) -> Self {
        self.keepalive = Some(keepalive);
        self
    }

    /// Build and connect the client.
    pub async fn build(self) -> Result<WorkflowClient, SdkError> {
        let address = self
            .address
            .ok_or_else(|| SdkError::InvalidArgument("address is required".into()))?;

        let connection = Connection::connect_with_options(ConnOpts {
            address,
            timeout: self.timeout,
            keepalive: self.keepalive,
        })
        .await?;

        let converter: Arc<dyn DataConverter> = match self.converter {
            Some(c) => Arc::from(c),
            None => Arc::new(JsonDataConverter::new()),
        };

        Ok(WorkflowClient {
            connection,
            converter,
            namespace: self.namespace,
            identity: self.identity,
            interceptors: self.interceptors,
        })
    }
}

/// Workflow operations available via `client.workflow()`.
pub struct WorkflowOperations<'a> {
    client: &'a WorkflowClient,
}

impl<'a> WorkflowOperations<'a> {
    /// Start building a new workflow execution.
    ///
    /// Returns a `WorkflowStartBuilder` that must be configured with
    /// at least `workflow_id()` and `task_queue()` before calling `start()`.
    pub fn start<T: Workflow>(&self, workflow_type: impl Into<String>) -> WorkflowStartBuilder<'a, T> {
        WorkflowStartBuilder::new(
            &self.client.connection,
            &self.client.converter,
            &self.client.namespace,
            &self.client.identity,
            workflow_type,
        )
    }

    /// Get a handle to an existing workflow by ID.
    pub fn handle<T: Workflow>(&self, workflow_id: impl Into<String>) -> WorkflowHandle<T> {
        WorkflowHandle::new(
            self.client.connection.clone(),
            self.client.converter.clone(),
            self.client.namespace.clone(),
            self.client.identity.clone(),
            workflow_id.into(),
            None,
        )
    }

    /// Get a handle to an existing workflow by ID and run ID.
    pub fn handle_with_run_id<T: Workflow>(
        &self,
        workflow_id: impl Into<String>,
        run_id: impl Into<String>,
    ) -> WorkflowHandle<T> {
        WorkflowHandle::new(
            self.client.connection.clone(),
            self.client.converter.clone(),
            self.client.namespace.clone(),
            self.client.identity.clone(),
            workflow_id.into(),
            Some(run_id.into()),
        )
    }

    /// List workflow executions in the current namespace.
    pub async fn list(&self, page_size: i32) -> Result<Vec<WorkflowExecutionInfo>, SdkError> {
        let response = self
            .client
            .connection
            .workflow_service()
            .list_workflow_executions(wfspb::ListWorkflowExecutionsRequest {
                namespace: self.client.namespace.clone(),
                page_size,
                ..Default::default()
            })
            .await?
            .into_inner();

        let infos = response
            .executions
            .into_iter()
            .map(|info| WorkflowExecutionInfo {
                workflow_id: info
                    .execution
                    .as_ref()
                    .map(|e| e.workflow_id.clone())
                    .unwrap_or_default(),
                run_id: info
                    .execution
                    .as_ref()
                    .map(|e| e.run_id.clone())
                    .unwrap_or_default(),
                workflow_type: info
                    .r#type
                    .as_ref()
                    .map(|t| t.name.clone())
                    .unwrap_or_default(),
                task_queue: info.task_queue,
                status: info.status,
                history_length: info.history_length,
                start_time: info.start_time,
                close_time: info.close_time,
            })
            .collect();

        Ok(infos)
    }

    /// Count workflow executions matching a query.
    pub async fn count(&self, query: &str) -> Result<u64, SdkError> {
        let response = self
            .client
            .connection
            .workflow_service()
            .count_workflow_executions(wfspb::CountWorkflowExecutionsRequest {
                namespace: self.client.namespace.clone(),
                query: query.to_string(),
                ..Default::default()
            })
            .await?
            .into_inner();

        Ok(response.count as u64)
    }
}
