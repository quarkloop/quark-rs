//! Example: connect to the workflow service and list namespaces.
//!
//! Run: cargo run -p workflow-example -- http://127.0.0.1:7233

use quark_workflow_rs::WorkflowClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:7233".to_string());

    let client = WorkflowClient::builder()
        .address(&endpoint)
        .namespace("default")
        .identity("workflow-example")
        .build()
        .await?;

    println!("Connected to workflow service at {endpoint}");

    // List namespaces
    let namespaces = client.namespace().list().await?;
    println!("Namespaces: {:?}", namespaces);

    Ok(())
}
