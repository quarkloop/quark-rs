//! Workflow handle — a typed reference to a running or completed workflow.
//!
//! The handle provides methods for interacting with the workflow:
//! `result()`, `signal()`, `query()`, `cancel()`, `terminate()`,
//! `describe()`, and `fetch_history()`.

use std::marker::PhantomData;
use std::sync::Arc;

use quark_workflow_proto::temporal::api::common::v1 as commonpb;
use quark_workflow_proto::temporal::api::enums::v1 as enumpb;
use quark_workflow_proto::temporal::api::history::v1 as historypb;
use quark_workflow_proto::temporal::api::workflowservice::v1 as wfspb;

use crate::connection::Connection;
use crate::converter::DataConverter;
use crate::definitions::{QueryDef, SignalDef};
use crate::errors::SdkError;
use crate::options::WorkflowExecutionDescription;
use crate::workflow_trait::Workflow;

/// A handle to a workflow execution.
///
/// Created by `WorkflowStartBuilder::start()` or
/// `WorkflowOperations::handle()`. Provides type-safe interaction
/// with the workflow.
pub struct WorkflowHandle<T: Workflow> {
    connection: Arc<Connection>,
    converter: Arc<dyn DataConverter>,
    namespace: String,
    identity: String,
    /// The workflow ID.
    pub workflow_id: String,
    /// The run ID (optional — if not set, the current run is used).
    pub run_id: Option<String>,
    _phantom: PhantomData<T>,
}

impl<T: Workflow> WorkflowHandle<T> {
    pub(crate) fn new(
        connection: Arc<Connection>,
        converter: Arc<dyn DataConverter>,
        namespace: String,
        identity: String,
        workflow_id: String,
        run_id: Option<String>,
    ) -> Self {
        Self {
            connection,
            converter,
            namespace,
            identity,
            workflow_id,
            run_id,
            _phantom: PhantomData,
        }
    }

    /// Wait for the workflow to complete and return its result.
    ///
    /// Polls `GetWorkflowExecutionHistory` until the workflow reaches
    /// a terminal state, then decodes the result payload.
    pub async fn result(&self) -> Result<T::Result, SdkError> {
        let mut page_token: Vec<u8> = Vec::new();

        loop {
            let response = self
                .connection
                .workflow_service()
                .get_workflow_execution_history(wfspb::GetWorkflowExecutionHistoryRequest {
                    namespace: self.namespace.clone(),
                    execution: Some(commonpb::WorkflowExecution {
                        workflow_id: self.workflow_id.clone(),
                        run_id: self.run_id.clone().unwrap_or_default(),
                    }),
                    next_page_token: page_token.clone().into(),
                    ..Default::default()
                })
                .await?
                .into_inner();

            if let Some(history) = &response.history {
                for event in &history.events {
                    if let Some(attrs) = &event.attributes {
                        match attrs {
                            historypb::history_event::Attributes::WorkflowExecutionCompletedEventAttributes(
                                a,
                            ) => {
                                if let Some(result_payloads) = &a.result {
                                    let result =
                                        crate::converter::from_single_payload::<T::Result>(
                                            &self.converter,
                                            result_payloads,
                                        )?;
                                    return Ok(result);
                                }
                                return serde_json::from_value(serde_json::Value::Null)
                                    .map_err(SdkError::from);
                            }
                            historypb::history_event::Attributes::WorkflowExecutionFailedEventAttributes(
                                a,
                            ) => {
                                let msg = a
                                    .failure
                                    .as_ref()
                                    .map(|f| f.message.clone())
                                    .unwrap_or_default();
                                return Err(SdkError::WorkflowFailed(msg));
                            }
                            historypb::history_event::Attributes::WorkflowExecutionTimedOutEventAttributes(_) => {
                                return Err(SdkError::Timeout);
                            }
                            historypb::history_event::Attributes::WorkflowExecutionCanceledEventAttributes(
                                _,
                            ) => {
                                return Err(SdkError::Cancelled);
                            }
                            historypb::history_event::Attributes::WorkflowExecutionTerminatedEventAttributes(
                                a,
                            ) => {
                                return Err(SdkError::WorkflowFailed(format!(
                                    "terminated: {}",
                                    a.reason
                                )));
                            }
                            _ => {}
                        }
                    }
                }
            }

            page_token = response.next_page_token.to_vec();
            if page_token.is_empty() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                page_token = Vec::new();
            }
        }
    }

    /// Send a signal to the workflow with type-checked arguments.
    pub async fn signal<A: serde::Serialize>(
        &self,
        def: SignalDef<A>,
        args: A,
    ) -> Result<(), SdkError> {
        let args_value = serde_json::to_value(&args)?;
        let input_payloads = self.converter.to_payloads(&[args_value])?;

        self.connection
            .workflow_service()
            .signal_workflow_execution(wfspb::SignalWorkflowExecutionRequest {
                namespace: self.namespace.clone(),
                workflow_execution: Some(commonpb::WorkflowExecution {
                    workflow_id: self.workflow_id.clone(),
                    run_id: self.run_id.clone().unwrap_or_default(),
                }),
                signal_name: def.name,
                input: Some(input_payloads),
                identity: self.identity.clone(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    /// Query the workflow with type-checked arguments and return type.
    pub async fn query<R: serde::de::DeserializeOwned, A: serde::Serialize>(
        &self,
        def: QueryDef<R, A>,
        args: A,
    ) -> Result<R, SdkError> {
        let args_value = serde_json::to_value(&args)?;
        let args_payloads = self.converter.to_payloads(&[args_value])?;

        let response = self
            .connection
            .workflow_service()
            .query_workflow(wfspb::QueryWorkflowRequest {
                namespace: self.namespace.clone(),
                execution: Some(commonpb::WorkflowExecution {
                    workflow_id: self.workflow_id.clone(),
                    run_id: self.run_id.clone().unwrap_or_default(),
                }),
                query: Some(quark_workflow_proto::temporal::api::query::v1::WorkflowQuery {
                    query_type: def.name,
                    query_args: Some(args_payloads),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?
            .into_inner();

        if let Some(rejected) = response.query_rejected {
            return Err(SdkError::QueryRejected(format!(
                "status: {:?}",
                enumpb::WorkflowExecutionStatus::try_from(rejected.status)
            )));
        }

        let result_payloads = response
            .query_result
            .ok_or_else(|| SdkError::Unexpected("query returned no result".into()))?;

        crate::converter::from_single_payload::<R>(&self.converter, &result_payloads)
    }

    /// Request cancellation of the workflow execution.
    pub async fn cancel(&self) -> Result<(), SdkError> {
        self.connection
            .workflow_service()
            .request_cancel_workflow_execution(
                wfspb::RequestCancelWorkflowExecutionRequest {
                    namespace: self.namespace.clone(),
                    workflow_execution: Some(commonpb::WorkflowExecution {
                        workflow_id: self.workflow_id.clone(),
                        run_id: self.run_id.clone().unwrap_or_default(),
                    }),
                    identity: self.identity.clone(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    /// Terminate the workflow execution with a reason.
    pub async fn terminate(&self, reason: impl Into<String>) -> Result<(), SdkError> {
        self.connection
            .workflow_service()
            .terminate_workflow_execution(wfspb::TerminateWorkflowExecutionRequest {
                namespace: self.namespace.clone(),
                workflow_execution: Some(commonpb::WorkflowExecution {
                    workflow_id: self.workflow_id.clone(),
                    run_id: self.run_id.clone().unwrap_or_default(),
                }),
                reason: reason.into(),
                identity: self.identity.clone(),
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    /// Describe the workflow execution.
    pub async fn describe(&self) -> Result<WorkflowExecutionDescription, SdkError> {
        let response = self
            .connection
            .workflow_service()
            .describe_workflow_execution(wfspb::DescribeWorkflowExecutionRequest {
                namespace: self.namespace.clone(),
                execution: Some(commonpb::WorkflowExecution {
                    workflow_id: self.workflow_id.clone(),
                    run_id: self.run_id.clone().unwrap_or_default(),
                }),
            })
            .await?
            .into_inner();

        let info = response
            .workflow_execution_info
            .ok_or_else(|| SdkError::Unexpected("missing workflow_execution_info".into()))?;

        let execution = info
            .execution
            .as_ref()
            .ok_or_else(|| SdkError::Unexpected("missing execution".into()))?;

        let workflow_type = info
            .r#type
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_default();

        let pending_activities: Vec<_> = response
            .pending_activities
            .iter()
            .map(|a| crate::options::PendingActivityInfo {
                activity_id: a.activity_id.clone(),
                activity_type: a
                    .activity_type
                    .as_ref()
                    .map(|t| t.name.clone())
                    .unwrap_or_default(),
                state: a.state,
                attempt: a.attempt,
            })
            .collect();

        Ok(WorkflowExecutionDescription {
            workflow_id: execution.workflow_id.clone(),
            run_id: execution.run_id.clone(),
            workflow_type,
            task_queue: info.task_queue.clone(),
            status: info.status,
            history_length: info.history_length,
            start_time: info.start_time,
            close_time: info.close_time,
            search_attributes: info.search_attributes,
            memo: info.memo,
            pending_activities,
        })
    }

    /// Fetch the complete workflow history.
    pub async fn fetch_history(&self) -> Result<historypb::History, SdkError> {
        let mut events = Vec::new();
        let mut page_token: Vec<u8> = Vec::new();

        loop {
            let response = self
                .connection
                .workflow_service()
                .get_workflow_execution_history(wfspb::GetWorkflowExecutionHistoryRequest {
                    namespace: self.namespace.clone(),
                    execution: Some(commonpb::WorkflowExecution {
                        workflow_id: self.workflow_id.clone(),
                        run_id: self.run_id.clone().unwrap_or_default(),
                    }),
                    next_page_token: page_token.clone().into(),
                    ..Default::default()
                })
                .await?
                .into_inner();

            if let Some(history) = response.history {
                events.extend(history.events);
            }

            page_token = response.next_page_token.to_vec();
            if page_token.is_empty() {
                break;
            }
        }

        Ok(historypb::History { events })
    }
}
