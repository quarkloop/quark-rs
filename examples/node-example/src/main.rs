//! Example: check node health and status.
//!
//! Run: cargo run -p node-example -- http://127.0.0.1:50051

use quark_node_rs::NodeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:50051".to_string());

    let client = NodeClient::builder()
        .endpoint(&endpoint)
        .build()
        .await?;

    println!("Connected to node at {endpoint}");

    // Health check (no token needed — node has no auth interceptor)
    let health = client.node().health("", "v1").await?;
    println!("Health: {:?}", health);

    // Ready check
    let ready = client.node().ready("", "v1").await?;
    println!("Ready: {:?}", ready);

    // Status
    let status = client.node().status("", "v1").await?;
    println!("Status: {:?}", status);

    Ok(())
}
