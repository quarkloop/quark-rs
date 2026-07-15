//! Fluent builder for starting workflow executions.
//!
//! Uses the builder pattern exclusively — callers must set
//! `workflow_id` and `task_queue` before calling `start()`.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use quark_workflow_proto::temporal::api::common::v1 as commonpb;
use quark_workflow_proto::temporal::api::workflowservice::v1 as wfspb;

use crate::connection::Connection;
use crate::converter::DataConverter;
use crate::errors::SdkError;
use crate::handle::WorkflowHandle;
use crate::options::WorkflowOptions;
use crate::workflow_trait::Workflow;

/// Builder for starting a workflow execution.
///
/// Created via `client.workflow().start::<T>(workflow_type)`.
/// Requires `workflow_id()` and `task_queue()` to be set before `start()`.
pub struct WorkflowStartBuilder<'a, T: Workflow> {
    connection: &'a Arc<Connection>,
    converter: &'a Arc<dyn DataConverter>,
    namespace: &'a str,
    identity: &'a str,
    workflow_type: String,
    workflow_id: Option<String>,
    task_queue: Option<String>,
    args: Vec<serde_json::Value>,
    options: WorkflowOptions,
    _phantom: PhantomData<T>,
}

impl<'a, T: Workflow> WorkflowStartBuilder<'a, T> {
    pub(crate) fn new(
        connection: &'a Arc<Connection>,
        converter: &'a Arc<dyn DataConverter>,
        namespace: &'a str,
        identity: &'a str,
        workflow_type: impl Into<String>,
    ) -> Self {
        Self {
            connection,
            converter,
            namespace,
            identity,
            workflow_type: workflow_type.into(),
            workflow_id: None,
            task_queue: None,
            args: Vec::new(),
            options: WorkflowOptions::default(),
            _phantom: PhantomData,
        }
    }

    /// Set the workflow ID. Required.
    pub fn workflow_id(mut self, id: impl Into<String>) -> Self {
        self.workflow_id = Some(id.into());
        self
    }

    /// Set the task queue. Required.
    pub fn task_queue(mut self, queue: impl Into<String>) -> Self {
        self.task_queue = Some(queue.into());
        self
    }

    /// Set the workflow arguments. These are serialized via the
    /// `DataConverter` and passed to the workflow function.
    pub fn args(mut self, args: Vec<serde_json::Value>) -> Self {
        self.args = args;
        self
    }

    /// Set the workflow run timeout.
    pub fn workflow_run_timeout(mut self, timeout: Duration) -> Self {
        self.options.workflow_run_timeout = Some(timeout);
        self
    }

    /// Set the workflow task timeout.
    pub fn workflow_task_timeout(mut self, timeout: Duration) -> Self {
        self.options.workflow_task_timeout = Some(timeout);
        self
    }

    /// Set search attributes for the workflow.
    pub fn search_attributes(
        mut self,
        attrs: std::collections::HashMap<String, commonpb::Payload>,
    ) -> Self {
        self.options.search_attributes = Some(attrs);
        self
    }

    /// Set a memo for the workflow.
    pub fn memo(mut self, memo: std::collections::HashMap<String, commonpb::Payload>) -> Self {
        self.options.memo = Some(memo);
        self
    }

    /// Request eager workflow start.
    pub fn request_eager_start(mut self, eager: bool) -> Self {
        self.options.request_eager_start = eager;
        self
    }

    /// Start the workflow and return a handle.
    ///
    /// Returns an error if `workflow_id` or `task_queue` is not set.
    pub async fn start(self) -> Result<WorkflowHandle<T>, SdkError> {
        let workflow_id = self
            .workflow_id
            .ok_or_else(|| SdkError::InvalidArgument("workflow_id is required".into()))?;
        let task_queue = self
            .task_queue
            .ok_or_else(|| SdkError::InvalidArgument("task_queue is required".into()))?;

        let input_payloads = self.converter.to_payloads(&self.args)?;

        let request = wfspb::StartWorkflowExecutionRequest {
            namespace: self.namespace.to_string(),
            workflow_id: workflow_id.clone(),
            workflow_type: Some(commonpb::WorkflowType {
                name: self.workflow_type,
            }),
            task_queue: Some(quark_workflow_proto::temporal::api::taskqueue::v1::TaskQueue {
                name: task_queue,
                kind: 0,
                normal_name: String::new(),
            }),
            input: Some(input_payloads),
            workflow_run_timeout: self
                .options
                .workflow_run_timeout
                .map(|d| prost_types::Duration {
                    seconds: d.as_secs() as i64,
                    nanos: 0,
                }),
            workflow_task_timeout: self
                .options
                .workflow_task_timeout
                .map(|d| prost_types::Duration {
                    seconds: d.as_secs() as i64,
                    nanos: 0,
                }),
            search_attributes: self.options.search_attributes.as_ref().map(|fields| {
                commonpb::SearchAttributes {
                    indexed_fields: fields.clone(),
                }
            }),
            memo: self.options.memo.as_ref().map(|fields| commonpb::Memo {
                fields: fields.clone(),
            }),
            request_eager_execution: self.options.request_eager_start,
            identity: self.identity.to_string(),
            request_id: uuid::Uuid::now_v7().to_string(),
            ..Default::default()
        };

        let response = self
            .connection
            .workflow_service()
            .start_workflow_execution(request)
            .await?
            .into_inner();

        Ok(WorkflowHandle::new(
            self.connection.clone(),
            self.converter.clone(),
            self.namespace.to_string(),
            self.identity.to_string(),
            workflow_id,
            Some(response.run_id),
        ))
    }

    /// Start the workflow and wait for its result.
    ///
    /// Equivalent to calling `start().await?` then `handle.result().await?`.
    pub async fn execute(self) -> Result<T::Result, SdkError> {
        let handle = self.start().await?;
        handle.result().await
    }
}
