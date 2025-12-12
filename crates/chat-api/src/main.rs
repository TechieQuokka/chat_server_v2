//! Chat API Server entry point
//!
//! Run with:
//! ```bash
//! cargo run -p chat-api
//! ```
//!
//! Configuration is loaded from environment variables or config files.

use chat_common::{try_init_tracing, AppConfig};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize tracing
    if let Err(e) = try_init_tracing() {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    // Run the server
    if let Err(e) = run().await {
        error!(error = %e, "Server failed to start");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Chat API Server...");

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| {
        error!(error = %e, "Failed to load configuration");
        e
    })?;

    info!(
        env = ?config.app.env,
        port = config.api.port,
        "Configuration loaded"
    );

    // Run the server
    chat_api::run(config).await?;

    Ok(())
}
