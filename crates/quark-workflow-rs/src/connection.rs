//! gRPC connection management.
//!
//! Manages the tonic `Channel` and provides typed gRPC service clients
//! for `WorkflowService` and `OperatorService`.

use std::sync::Arc;
use std::time::Duration;

use tonic::transport::Channel;
use quark_workflow_proto::temporal::api::operatorservice::v1::operator_service_client::OperatorServiceClient;
use quark_workflow_proto::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;

use crate::errors::SdkError;
use crate::options::ConnectionOptions as ConnOpts;

/// A gRPC connection to the workflow-rs server.
///
/// Connections are expensive to create and should be reused. A single
/// `Connection` can be shared by multiple `WorkflowClient` instances.
pub struct Connection {
    channel: Channel,
    workflow_service: WorkflowServiceClient<Channel>,
    operator_service: OperatorServiceClient<Channel>,
}

impl Connection {
    /// Connect to the server at the given address.
    ///
    /// `address` should be in `http://host:port` or `https://host:port` format.
    pub async fn connect(address: impl Into<String>) -> Result<Arc<Self>, SdkError> {
        Self::connect_with_options(ConnOpts {
            address: address.into(),
            ..Default::default()
        })
        .await
    }

    /// Connect to the server using the given options.
    pub async fn connect_with_options(opts: ConnOpts) -> Result<Arc<Self>, SdkError> {
        let mut endpoint = Channel::from_shared(opts.address.clone())
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        if let Some(timeout) = opts.timeout {
            endpoint = endpoint.timeout(timeout);
        }

        if let Some(keepalive) = opts.keepalive {
            endpoint = endpoint
                .keep_alive_while_idle(true)
                .http2_keep_alive_interval(keepalive);
        }

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        let workflow_service = WorkflowServiceClient::new(channel.clone());
        let operator_service = OperatorServiceClient::new(channel.clone());

        Ok(Arc::new(Self {
            channel,
            workflow_service,
            operator_service,
        }))
    }

    /// Returns a cloned `WorkflowServiceClient` for direct gRPC access.
    ///
    /// This is intended for advanced use cases and testing where the
    /// SDK's higher-level API doesn't provide what's needed (e.g.,
    /// manually polling workflow tasks).
    pub fn workflow_service(&self) -> WorkflowServiceClient<Channel> {
        self.workflow_service.clone()
    }

    /// Returns a cloned `OperatorServiceClient`.
    pub(crate) fn operator_service(&self) -> OperatorServiceClient<Channel> {
        self.operator_service.clone()
    }
}
