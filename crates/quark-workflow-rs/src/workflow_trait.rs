//! The `Workflow` trait and associated types.
//!
//! Users implement the `Workflow` trait to define a workflow's type
//! name and result type. This provides compile-time type safety for
//! workflow handles.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// A trait representing a workflow type.
///
/// Users define workflows as zero-sized types that implement this trait.
/// The `Result` associated type provides type safety for `handle.result()`.
///
/// # Example
///
/// ```rust,ignore
/// struct OrderWorkflow;
///
/// impl Workflow for OrderWorkflow {
///     type Result = OrderResult;
///     const WORKFLOW_TYPE: &'static str = "processOrder";
/// }
/// ```
pub trait Workflow: Send + Sync {
    /// The result type returned by the workflow.
    type Result: DeserializeOwned + Serialize;

    /// The workflow type name as registered on the server.
    const WORKFLOW_TYPE: &'static str;
}
