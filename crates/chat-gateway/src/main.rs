//! Chat Gateway Server entry point
//!
//! Run with:
//! ```bash
//! cargo run -p chat-gateway
//! ```
//!
//! Configuration is loaded from environment variables.

use chat_common::{try_init_tracing, AppConfig};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize tracing
    if let Err(e) = try_init_tracing() {
        eprintln!("Warning: Failed to initialize tracing: {e}");
    }

    // Run the server
    if let Err(e) = run().await {
        error!(error = %e, "Gateway failed to start");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Chat Gateway Server...");

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| {
        error!(error = %e, "Failed to load configuration");
        e
    })?;

    info!(
        env = ?config.app.env,
        port = config.gateway.port,
        "Configuration loaded"
    );

    // Run the gateway server
    chat_gateway::run(config).await?;

    Ok(())
}
