//! Integration tests for the generated `workflow-api` types.
//!
//! These tests verify that the prost-generated types:
//! - can be constructed with expected field values
//! - have correct default values (protobuf defaults)
//! - support round-trip encode/decode
//! - nested messages and maps work correctly
//! - enum variants are accessible

use prost::Message;
use workflow_api::temporal::api::common::v1::{
    ActivityType, Header, Memo, Payload, Payloads, RetryPolicy, SearchAttributes,
    WorkflowExecution, WorkflowType,
};
use workflow_api::temporal::api::enums::v1::{
    EncodingType, TaskQueueKind, WorkflowExecutionStatus, WorkflowIdConflictPolicy,
    WorkflowIdReusePolicy,
};
use workflow_api::temporal::api::failure::v1::Failure;
use workflow_api::temporal::api::taskqueue::v1::TaskQueue;
use workflow_api::temporal::api::workflowservice::v1::{
    StartWorkflowExecutionRequest, StartWorkflowExecutionResponse,
};

// ---------------------------------------------------------------------------
// Message construction tests
// ---------------------------------------------------------------------------

#[test]
fn start_workflow_request_construction() {
    let req = StartWorkflowExecutionRequest {
        namespace: "default".into(),
        workflow_id: "wf-123".into(),
        workflow_type: Some(WorkflowType {
            name: "MyWorkflow".into(),
        }),
        task_queue: Some(TaskQueue {
            name: "my-queue".into(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: String::new(),
        }),
        input: None,
        workflow_execution_timeout: None,
        workflow_run_timeout: None,
        workflow_task_timeout: None,
        identity: "test-client".into(),
        request_id: "req-uuid".into(),
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        workflow_id_conflict_policy: WorkflowIdConflictPolicy::Fail as i32,
        retry_policy: None,
        cron_schedule: String::new(),
        memo: None,
        search_attributes: None,
        header: None,
        request_eager_execution: false,
        continued_failure: None,
        last_completion_result: None,
        workflow_start_delay: None,
        completion_callbacks: vec![],
        user_metadata: None,
        links: vec![],
        versioning_override: None,
        on_conflict_options: None,
        priority: None,
        eager_worker_deployment_options: None,
        time_skipping_config: None,
    };

    assert_eq!(req.namespace, "default");
    assert_eq!(req.workflow_id, "wf-123");
    assert_eq!(req.workflow_type.as_ref().unwrap().name, "MyWorkflow");
    assert!(!req.request_eager_execution);
}

#[test]
fn workflow_execution_construction() {
    let we = WorkflowExecution {
        workflow_id: "wf-abc".into(),
        run_id: "run-xyz".into(),
    };
    assert_eq!(we.workflow_id, "wf-abc");
    assert_eq!(we.run_id, "run-xyz");
}

#[test]
fn workflow_type_construction() {
    let wt = WorkflowType {
        name: "GreetWorld".into(),
    };
    assert_eq!(wt.name, "GreetWorld");
}

#[test]
fn activity_type_construction() {
    let at = ActivityType {
        name: "SendEmail".into(),
    };
    assert_eq!(at.name, "SendEmail");
}

// ---------------------------------------------------------------------------
// Default value tests (protobuf default: 0 for ints, "" for strings, etc.)
// ---------------------------------------------------------------------------

#[test]
fn start_workflow_request_defaults() {
    let req = StartWorkflowExecutionRequest::default();
    assert!(req.namespace.is_empty());
    assert!(req.workflow_id.is_empty());
    assert!(req.workflow_type.is_none());
    assert!(req.task_queue.is_none());
    assert!(req.input.is_none());
    assert!(req.identity.is_empty());
    assert!(req.request_id.is_empty());
    assert_eq!(req.workflow_id_reuse_policy, 0);
    assert_eq!(req.workflow_id_conflict_policy, 0);
    assert!(!req.request_eager_execution);
    assert!(req.cron_schedule.is_empty());
}

#[test]
fn payload_defaults() {
    let p = Payload::default();
    assert!(p.data.is_empty());
    assert!(p.metadata.is_empty());
    assert!(p.external_payloads.is_empty());
}

#[test]
fn retry_policy_defaults() {
    let rp = RetryPolicy::default();
    assert!(rp.initial_interval.is_none());
    assert_eq!(rp.backoff_coefficient, 0.0);
    assert!(rp.maximum_interval.is_none());
    assert_eq!(rp.maximum_attempts, 0);
}

#[test]
fn workflow_execution_defaults() {
    let we = WorkflowExecution::default();
    assert!(we.workflow_id.is_empty());
    assert!(we.run_id.is_empty());
}

// ---------------------------------------------------------------------------
// Round-trip encode/decode tests
// ---------------------------------------------------------------------------

#[test]
fn start_workflow_request_roundtrip() {
    let req = StartWorkflowExecutionRequest {
        namespace: "production".into(),
        workflow_id: "wf-roundtrip".into(),
        workflow_type: Some(WorkflowType {
            name: "OrderProcess".into(),
        }),
        task_queue: Some(TaskQueue {
            name: "orders".into(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: String::new(),
        }),
        input: Some(Payloads {
            payloads: vec![Payload {
                metadata: [(
                    "encoding".into(),
                    prost::bytes::Bytes::from_static(b"json/plain"),
                )]
                .into_iter()
                .collect(),
                data: prost::bytes::Bytes::from_static(b"{\"order_id\": 42}"),
                external_payloads: vec![],
            }],
        }),
        workflow_execution_timeout: Some(prost_types::Duration {
            seconds: 3600,
            nanos: 0,
        }),
        workflow_run_timeout: Some(prost_types::Duration {
            seconds: 600,
            nanos: 0,
        }),
        workflow_task_timeout: Some(prost_types::Duration {
            seconds: 10,
            nanos: 0,
        }),
        identity: "test-client".into(),
        request_id: "req-rt-001".into(),
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        workflow_id_conflict_policy: WorkflowIdConflictPolicy::Fail as i32,
        retry_policy: Some(RetryPolicy {
            initial_interval: Some(prost_types::Duration {
                seconds: 1,
                nanos: 0,
            }),
            backoff_coefficient: 2.0,
            maximum_interval: Some(prost_types::Duration {
                seconds: 60,
                nanos: 0,
            }),
            maximum_attempts: 3,
            non_retryable_error_types: vec![],
        }),
        cron_schedule: "0 0 * * *".into(),
        memo: Some(Memo {
            fields: [(
                "key".into(),
                Payload {
                    metadata: std::collections::HashMap::new(),
                    data: prost::bytes::Bytes::from_static(b"val"),
                    external_payloads: vec![],
                },
            )]
            .into_iter()
            .collect(),
        }),
        search_attributes: Some(SearchAttributes {
            indexed_fields: std::collections::HashMap::new(),
        }),
        header: Some(Header {
            fields: std::collections::HashMap::new(),
        }),
        request_eager_execution: false,
        continued_failure: None,
        last_completion_result: None,
        workflow_start_delay: None,
        completion_callbacks: vec![],
        user_metadata: None,
        links: vec![],
        versioning_override: None,
        on_conflict_options: None,
        priority: None,
        eager_worker_deployment_options: None,
        time_skipping_config: None,
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).expect("encode failed");

    let decoded = StartWorkflowExecutionRequest::decode(&buf[..]).expect("decode failed");

    assert_eq!(decoded.namespace, req.namespace);
    assert_eq!(decoded.workflow_id, req.workflow_id);
    assert_eq!(
        decoded.workflow_type.as_ref().unwrap().name,
        req.workflow_type.as_ref().unwrap().name
    );
    assert_eq!(
        decoded.task_queue.as_ref().unwrap().name,
        req.task_queue.as_ref().unwrap().name
    );
    assert_eq!(decoded.identity, req.identity);
    assert_eq!(decoded.request_id, req.request_id);
    assert_eq!(
        decoded.workflow_id_reuse_policy,
        req.workflow_id_reuse_policy
    );
    assert_eq!(
        decoded.workflow_id_conflict_policy,
        req.workflow_id_conflict_policy
    );
    assert_eq!(decoded.cron_schedule, req.cron_schedule);

    // Verify nested message round-trip.
    let input = decoded.input.as_ref().unwrap();
    assert_eq!(input.payloads.len(), 1);
    assert_eq!(
        input.payloads[0].data,
        prost::bytes::Bytes::from_static(b"{\"order_id\": 42}")
    );
    assert_eq!(
        input.payloads[0].metadata.get("encoding").unwrap(),
        &prost::bytes::Bytes::from_static(b"json/plain")
    );

    // Verify retry policy.
    let rp = decoded.retry_policy.as_ref().unwrap();
    assert_eq!(
        rp.initial_interval,
        Some(prost_types::Duration {
            seconds: 1,
            nanos: 0,
        })
    );
    assert_eq!(rp.backoff_coefficient, 2.0);
    assert_eq!(
        rp.maximum_interval,
        Some(prost_types::Duration {
            seconds: 60,
            nanos: 0,
        })
    );
    assert_eq!(rp.maximum_attempts, 3);

    // Verify durations.
    let exec_timeout = decoded.workflow_execution_timeout.as_ref().unwrap();
    assert_eq!(exec_timeout.seconds, 3600);
    let run_timeout = decoded.workflow_run_timeout.as_ref().unwrap();
    assert_eq!(run_timeout.seconds, 600);
    let task_timeout = decoded.workflow_task_timeout.as_ref().unwrap();
    assert_eq!(task_timeout.seconds, 10);

    // Verify memo.
    let memo = decoded.memo.as_ref().unwrap();
    assert_eq!(memo.fields.len(), 1);
}

#[test]
fn workflow_execution_roundtrip() {
    let we = WorkflowExecution {
        workflow_id: "wf-test".into(),
        run_id: "run-test".into(),
    };
    let mut buf = Vec::new();
    we.encode(&mut buf).unwrap();
    let decoded = WorkflowExecution::decode(&buf[..]).unwrap();
    assert_eq!(we, decoded);
}

#[test]
fn payloads_roundtrip() {
    let payloads = Payloads {
        payloads: vec![
            Payload {
                metadata: std::collections::HashMap::new(),
                data: prost::bytes::Bytes::from_static(b"hello"),
                external_payloads: vec![],
            },
            Payload {
                metadata: [("encoding".into(), prost::bytes::Bytes::from_static(b"json"))]
                    .into_iter()
                    .collect(),
                data: prost::bytes::Bytes::from_static(b"{\"key\":1}"),
                external_payloads: vec![],
            },
        ],
    };
    let mut buf = Vec::new();
    payloads.encode(&mut buf).unwrap();
    let decoded = Payloads::decode(&buf[..]).unwrap();
    assert_eq!(payloads, decoded);
}

#[test]
fn retry_policy_roundtrip() {
    let rp = RetryPolicy {
        initial_interval: Some(prost_types::Duration {
            seconds: 5,
            nanos: 0,
        }),
        backoff_coefficient: 2.5,
        maximum_interval: Some(prost_types::Duration {
            seconds: 120,
            nanos: 0,
        }),
        maximum_attempts: 10,
        non_retryable_error_types: vec!["InvalidInput".into(), "ValidationError".into()],
    };
    let mut buf = Vec::new();
    rp.encode(&mut buf).unwrap();
    let decoded = RetryPolicy::decode(&buf[..]).unwrap();
    assert_eq!(rp, decoded);
}

#[test]
fn search_attributes_roundtrip() {
    let sa = SearchAttributes {
        indexed_fields: [(
            "CustomStringField".into(),
            Payload {
                metadata: std::collections::HashMap::new(),
                data: prost::bytes::Bytes::from_static(b"test-value"),
                external_payloads: vec![],
            },
        )]
        .into_iter()
        .collect(),
    };
    let mut buf = Vec::new();
    sa.encode(&mut buf).unwrap();
    let decoded = SearchAttributes::decode(&buf[..]).unwrap();
    assert_eq!(sa.indexed_fields.len(), decoded.indexed_fields.len());
    assert_eq!(
        sa.indexed_fields["CustomStringField"].data,
        decoded.indexed_fields["CustomStringField"].data
    );
}

// ---------------------------------------------------------------------------
// Enum tests
// ---------------------------------------------------------------------------

#[test]
fn enum_variant_values() {
    assert_eq!(WorkflowIdReusePolicy::AllowDuplicate as i32, 1);
    assert_eq!(WorkflowIdReusePolicy::AllowDuplicateFailedOnly as i32, 2);
    assert_eq!(WorkflowIdReusePolicy::RejectDuplicate as i32, 3);
    assert_eq!(WorkflowIdReusePolicy::TerminateIfRunning as i32, 4);
}

#[test]
fn enum_conflict_policy_values() {
    assert_eq!(WorkflowIdConflictPolicy::Fail as i32, 1);
    assert_eq!(WorkflowIdConflictPolicy::UseExisting as i32, 2);
    assert_eq!(WorkflowIdConflictPolicy::TerminateExisting as i32, 3);
}

#[test]
fn encoding_type_values() {
    assert_eq!(EncodingType::Unspecified as i32, 0);
    assert_eq!(EncodingType::Proto3 as i32, 1);
    assert_eq!(EncodingType::Json as i32, 2);
}

#[test]
fn workflow_execution_status_values() {
    assert_eq!(WorkflowExecutionStatus::Running as i32, 1);
    assert_eq!(WorkflowExecutionStatus::Completed as i32, 2);
    assert_eq!(WorkflowExecutionStatus::Failed as i32, 3);
    assert_eq!(WorkflowExecutionStatus::Canceled as i32, 4);
    assert_eq!(WorkflowExecutionStatus::Terminated as i32, 5);
    assert_eq!(WorkflowExecutionStatus::ContinuedAsNew as i32, 6);
    assert_eq!(WorkflowExecutionStatus::TimedOut as i32, 7);
}

// ---------------------------------------------------------------------------
// Server API type tests
// ---------------------------------------------------------------------------

#[test]
fn history_service_request_construction() {
    use workflow_api::temporal::server::api::historyservice::v1::StartWorkflowExecutionRequest as ServerStartReq;

    let req = ServerStartReq::default();
    // Server request wraps the public request + namespace_id
    assert!(req.namespace_id.is_empty());
}

#[test]
fn server_enums_accessible() {
    use workflow_api::temporal::server::api::enums::v1::WorkflowExecutionState;

    assert_eq!(WorkflowExecutionState::Created as i32, 1);
    assert_eq!(WorkflowExecutionState::Running as i32, 2);
    assert_eq!(WorkflowExecutionState::Completed as i32, 3);
}

// ---------------------------------------------------------------------------
// Map field tests
// ---------------------------------------------------------------------------

#[test]
fn payload_metadata_map() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "encoding".into(),
        prost::bytes::Bytes::from_static(b"json/plain"),
    );
    metadata.insert(
        "compression".into(),
        prost::bytes::Bytes::from_static(b"gzip"),
    );

    let p = Payload {
        metadata,
        data: prost::bytes::Bytes::from_static(b"test-data"),
        external_payloads: vec![],
    };

    assert_eq!(p.metadata.len(), 2);
    assert_eq!(
        p.metadata.get("encoding").unwrap(),
        &prost::bytes::Bytes::from_static(b"json/plain")
    );
    assert_eq!(
        p.metadata.get("compression").unwrap(),
        &prost::bytes::Bytes::from_static(b"gzip")
    );
}

#[test]
fn memo_map_roundtrip() {
    let memo = Memo {
        fields: [
            (
                "customer_id".into(),
                Payload {
                    metadata: std::collections::HashMap::new(),
                    data: prost::bytes::Bytes::from_static(b"cust-42"),
                    external_payloads: vec![],
                },
            ),
            (
                "order_count".into(),
                Payload {
                    metadata: std::collections::HashMap::new(),
                    data: prost::bytes::Bytes::from_static(b"7"),
                    external_payloads: vec![],
                },
            ),
        ]
        .into_iter()
        .collect(),
    };

    let mut buf = Vec::new();
    memo.encode(&mut buf).unwrap();
    let decoded = Memo::decode(&buf[..]).unwrap();
    assert_eq!(decoded.fields.len(), 2);
    assert_eq!(
        decoded.fields["customer_id"].data,
        prost::bytes::Bytes::from_static(b"cust-42")
    );
    assert_eq!(
        decoded.fields["order_count"].data,
        prost::bytes::Bytes::from_static(b"7")
    );
}

// ---------------------------------------------------------------------------
// Large payload / edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_payloads_roundtrip() {
    let payloads = Payloads { payloads: vec![] };
    let mut buf = Vec::new();
    payloads.encode(&mut buf).unwrap();
    let decoded = Payloads::decode(&buf[..]).unwrap();
    assert!(decoded.payloads.is_empty());
}

#[test]
fn large_payload_data() {
    // 10KB payload
    let large_data = vec![0xAB_u8; 10 * 1024];
    let p = Payload {
        metadata: std::collections::HashMap::new(),
        data: prost::bytes::Bytes::from(large_data.clone()),
        external_payloads: vec![],
    };

    let mut buf = Vec::new();
    p.encode(&mut buf).unwrap();
    let decoded = Payload::decode(&buf[..]).unwrap();
    assert_eq!(decoded.data.as_ref(), large_data.as_slice());
}

#[test]
fn zero_duration_roundtrip() {
    let dur = prost_types::Duration {
        seconds: 0,
        nanos: 0,
    };
    let mut buf = Vec::new();
    dur.encode(&mut buf).unwrap();
    let decoded = prost_types::Duration::decode(&buf[..]).unwrap();
    assert_eq!(decoded.seconds, 0);
    assert_eq!(decoded.nanos, 0);
}

#[test]
fn negative_duration_roundtrip() {
    // Protobuf allows negative durations
    let dur = prost_types::Duration {
        seconds: -5,
        nanos: -500_000_000,
    };
    let mut buf = Vec::new();
    dur.encode(&mut buf).unwrap();
    let decoded = prost_types::Duration::decode(&buf[..]).unwrap();
    assert_eq!(decoded.seconds, -5);
    assert_eq!(decoded.nanos, -500_000_000);
}

// ---------------------------------------------------------------------------
// Response type tests
// ---------------------------------------------------------------------------

#[test]
fn start_workflow_response_defaults() {
    let resp = StartWorkflowExecutionResponse::default();
    assert!(resp.run_id.is_empty());
    assert!(!resp.started);
}

// ---------------------------------------------------------------------------
// Failure message tests
// ---------------------------------------------------------------------------

#[test]
fn failure_construction_and_roundtrip() {
    let failure = Failure {
        message: "Something went wrong".into(),
        source: "my-activity".into(),
        stack_trace: "at fn_a\n  at fn_b".into(),
        encoded_attributes: None,
        cause: None,
        failure_info: None,
    };

    let mut buf = Vec::new();
    failure.encode(&mut buf).unwrap();
    let decoded = Failure::decode(&buf[..]).unwrap();
    assert_eq!(decoded.message, "Something went wrong");
    assert_eq!(decoded.source, "my-activity");
    assert_eq!(decoded.stack_trace, "at fn_a\n  at fn_b");
}

#[test]
fn nested_failure_chain() {
    let inner = Failure {
        message: "inner error".into(),
        source: "inner-source".into(),
        stack_trace: String::new(),
        encoded_attributes: None,
        cause: None,
        failure_info: None,
    };

    let outer = Failure {
        message: "outer error".into(),
        source: "outer-source".into(),
        stack_trace: String::new(),
        encoded_attributes: None,
        cause: Some(Box::new(inner.clone())),
        failure_info: None,
    };

    let mut buf = Vec::new();
    outer.encode(&mut buf).unwrap();
    let decoded = Failure::decode(&buf[..]).unwrap();
    assert_eq!(decoded.message, "outer error");

    let cause = decoded.cause.as_ref().unwrap();
    assert_eq!(cause.message, "inner error");
    assert_eq!(cause.source, "inner-source");
}

// ---------------------------------------------------------------------------
// Header tests
// ---------------------------------------------------------------------------

#[test]
fn header_roundtrip() {
    let header = Header {
        fields: [(
            "trace-id".into(),
            Payload {
                metadata: std::collections::HashMap::new(),
                data: prost::bytes::Bytes::from_static(b"abc-123"),
                external_payloads: vec![],
            },
        )]
        .into_iter()
        .collect(),
    };

    let mut buf = Vec::new();
    header.encode(&mut buf).unwrap();
    let decoded = Header::decode(&buf[..]).unwrap();
    assert_eq!(decoded.fields.len(), 1);
    assert_eq!(
        decoded.fields["trace-id"].data,
        prost::bytes::Bytes::from_static(b"abc-123")
    );
}

// ---------------------------------------------------------------------------
// Prost codec encode/decode size tests
// ---------------------------------------------------------------------------

#[test]
fn encode_length_delimited() {
    // Prost uses length-delimited encoding by default for messages
    let req = StartWorkflowExecutionRequest {
        namespace: "test".into(),
        ..Default::default()
    };

    let mut buf = Vec::new();
    req.encode(&mut buf).unwrap();
    assert!(!buf.is_empty());

    // First byte should be the tag for field 1 (wire type 2 = length-delimited)
    assert_eq!(buf[0] >> 3, 1); // field number 1
    assert_eq!(buf[0] & 0x07, 2); // wire type 2 = length-delimited
}
