//! Example: query the server's service registry and system health.
//!
//! Run: cargo run -p server-example -- http://127.0.0.1:3000

use quark_server_rs::ServerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string());

    let client = ServerClient::builder()
        .endpoint(&endpoint)
        .build()
        .await?;

    println!("Connected to server at {endpoint}");

    let token = std::env::var("AUTH_TOKEN").unwrap_or_else(|_| "test-token".to_string());

    // Get service registry
    let registry = client.server().get_service_registry(&token).await?;
    println!("Service registry:");
    for service in &registry.services {
        println!("  - {} at {}", service.name, service.grpc_url);
    }

    // Get system health
    let health = client.server().get_system_health(&token).await?;
    println!("\nSystem health:");
    for service in &health.services {
        println!("  - {} (healthy: {})", service.name, service.healthy);
    }

    Ok(())
}
