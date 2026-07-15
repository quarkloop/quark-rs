//! Example: authenticate with the auth service and list users.
//!
//! Run: cargo run -p auth-example -- http://127.0.0.1:5001

use quark_auth_rs::AuthClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:5001".to_string());

    let mut client = AuthClient::builder()
        .endpoint(&endpoint)
        .build()
        .await?;

    println!("Connected to auth service at {endpoint}");

    // Login
    let login = client
        
        .login("admin", "admin-api-key")
        .await
        .map_err(|e| {
            eprintln!("Login failed: {e}");
            e
        })?;

    println!("Login successful!");
    let token_preview = &login.access_token[..20.min(login.access_token.len())];
    println!("  access_token: {token_preview}...");

    // List users (authenticated)
    let users = client
        .users()
        .list(&login.access_token, 10, 0, "")
        .await?;

    println!("Found {} users:", users.users.len());
    for user in &users.users {
        println!("  - {} ({})", user.display_name, user.email);
    }

    Ok(())
}
