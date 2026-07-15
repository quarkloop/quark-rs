//! Example: use the unified QuarkClient to connect to all services.
//!
//! Run: cargo run -p unified-example

use quark_rs::QuarkClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a unified client connecting to all platform services.
    // Only configure the services you need — unconfigured services return
    // an error when accessed.
    let mut client = QuarkClient::builder()
        .auth_endpoint("http://127.0.0.1:5001")
        .server_endpoint("http://127.0.0.1:3000")
        .node_endpoint("http://127.0.0.1:50051")
        .workflow_endpoint("http://127.0.0.1:7233")
        .workflow_namespace("default")
        .workflow_identity("unified-example")
        .connect_timeout(Duration::from_secs(5))
        .build()
        .await?;

    println!("Connected to all platform services!");

    // Access auth service
    match client.auth() {
        Ok(_auth) => {
            println!("Auth client ready");
            // let login = auth.auth().login("user", "key").await?;
        }
        Err(e) => println!("Auth not configured: {e}"),
    }

    // Access server service
    match client.server() {
        Ok(_server) => {
            println!("Server client ready");
            // let registry = server.server().get_service_registry(token).await?;
        }
        Err(e) => println!("Server not configured: {e}"),
    }

    // Access node service
    match client.node() {
        Ok(_node) => {
            println!("Node client ready");
            // let health = node.node().health("", "v1").await?;
        }
        Err(e) => println!("Node not configured: {e}"),
    }

    // Access workflow service
    match client.workflow() {
        Ok(_workflow) => {
            println!("Workflow client ready");
            // let namespaces = workflow.namespace().list().await?;
        }
        Err(e) => println!("Workflow not configured: {e}"),
    }

    Ok(())
}
