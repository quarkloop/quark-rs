//! Type-safe Rust client SDK for the workflow-rs server.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use workflow_sdk::{WorkflowClient, Workflow};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct OrderResult { order_id: String, status: String }
//!
//! struct OrderWorkflow;
//! impl Workflow for OrderWorkflow {
//!     type Result = OrderResult;
//!     const WORKFLOW_TYPE: &'static str = "processOrder";
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = WorkflowClient::builder()
//!         .address("http://localhost:7233")
//!         .build()
//!         .await?;
//!
//!     let handle = client.workflow()
//!         .start::<OrderWorkflow>("processOrder")
//!         .workflow_id("order-123")
//!         .task_queue("orders")
//!         .args(vec![serde_json::json!({ "orderId": "123" })])
//!         .start()
//!         .await?;
//!
//!     let result: OrderResult = handle.result().await?;
//!     println!("Result: {:?}", result);
//!     Ok(())
//! }
//! ```

mod builder;
mod client;
mod connection;
mod converter;
mod definitions;
mod errors;
mod handle;
mod interceptors;
mod namespace;
mod options;
mod workflow_trait;

pub use builder::WorkflowStartBuilder;
pub use client::{WorkflowClient, WorkflowClientBuilder, WorkflowOperations};
pub use connection::Connection;
pub use converter::{DataConverter, JsonDataConverter};
pub use definitions::{signal, query, update, SignalDef, QueryDef, UpdateDef};
pub use errors::SdkError;
pub use handle::WorkflowHandle;
pub use interceptors::{Interceptor, MetadataInterceptor};
pub use namespace::NamespaceOperations;
pub use options::{
    NamespaceDescription, PendingActivityInfo, RegisterNamespaceRequest, TlsConfig,
    WorkflowExecutionDescription, WorkflowExecutionInfo, WorkflowOptions,
};
pub use workflow_trait::Workflow;
