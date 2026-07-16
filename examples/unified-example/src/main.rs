//! Example: use the unified QuarkClient with automatic service discovery.
//!
//! Only the server endpoint is needed — all other service URLs are
//! discovered automatically via the server's ServiceDiscovery service.
//!
//! Run: cargo run -p unified-example

use quark_rs::QuarkClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a unified client — only the server endpoint is required.
    // The SDK calls DiscoverServices on the server to find all other
    // services (auth, node, workflow) automatically.
    let mut client = QuarkClient::builder()
        .server_endpoint("http://127.0.0.1:3000")
        .workflow_namespace("default")
        .workflow_identity("unified-example")
        .connect_timeout(Duration::from_secs(5))
        .build()
        .await?;

    println!("Connected to server — services discovered automatically!");

    // Access auth service (discovered from server)
    match client.auth() {
        Ok(_auth) => {
            println!("Auth client ready");
            // let login = _auth.login("user", "key").await?;
        }
        Err(e) => println!("Auth not discovered: {e}"),
    }

    // Access server service (always available — it's the bootstrap endpoint)
    match client.server() {
        Ok(_server) => {
            println!("Server client ready");
        }
        Err(e) => println!("Server not configured: {e}"),
    }

    // Access node service (discovered from server)
    match client.node() {
        Ok(_node) => {
            println!("Node client ready");
            // let health = _node.health("", "v1").await?;
        }
        Err(e) => println!("Node not discovered: {e}"),
    }

    // Access workflow service (discovered from server)
    match client.workflow() {
        Ok(_workflow) => {
            println!("Workflow client ready");
            // let namespaces = _workflow.namespace().list().await?;
        }
        Err(e) => println!("Workflow not discovered: {e}"),
    }

    Ok(())
}
