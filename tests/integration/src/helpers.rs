//! Test helpers for integration tests
//!
//! Provides utilities for spawning test servers, making HTTP requests,
//! and managing test data cleanup.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use anyhow::Result;
use chat_api::{create_app, create_app_state};
use chat_common::AppConfig;
use reqwest::{Client, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Counter for unique test ports
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);

/// Get a unique port for testing
pub fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Test server instance that manages lifecycle
pub struct TestServer {
    pub addr: SocketAddr,
    pub client: Client,
    _handle: JoinHandle<()>,
}

impl TestServer {
    /// Start a new test server
    pub async fn start() -> Result<Self> {
        let config = test_config()?;
        Self::start_with_config(config).await
    }

    /// Start a test server with custom config
    pub async fn start_with_config(config: AppConfig) -> Result<Self> {
        let port = get_test_port();
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Create app state
        let state = create_app_state(config).await?;

        // Build application
        let app = create_app(state);

        // Bind to port
        let listener = TcpListener::bind(addr).await?;
        let actual_addr = listener.local_addr()?;

        // Spawn server task
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            addr: actual_addr,
            client,
            _handle: handle,
        })
    }

    /// Get base URL for the server
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self.client.get(&url).send().await?)
    }

    /// Make a GET request with auth token
    pub async fn get_auth(&self, path: &str, token: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?)
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self.client.post(&url).json(body).send().await?)
    }

    /// Make a POST request with auth token
    pub async fn post_auth<T: Serialize>(
        &self,
        path: &str,
        token: &str,
        body: &T,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await?)
    }

    /// Make a PATCH request with auth token
    pub async fn patch_auth<T: Serialize>(
        &self,
        path: &str,
        token: &str,
        body: &T,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await?)
    }

    /// Make a DELETE request with auth token
    pub async fn delete_auth(&self, path: &str, token: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?)
    }

    /// Make a PUT request with auth token
    pub async fn put_auth<T: Serialize>(
        &self,
        path: &str,
        token: &str,
        body: &T,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url(), path);
        Ok(self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await?)
    }
}

/// Create a test configuration
pub fn test_config() -> Result<AppConfig> {
    // Load from environment or use defaults
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("Config error: {}", e))?;

    Ok(config)
}

/// Helper to check if test environment is available
pub async fn check_test_env() -> bool {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("Skipping test: DATABASE_URL not set");
        return false;
    }

    if std::env::var("REDIS_URL").is_err() {
        eprintln!("Skipping test: REDIS_URL not set");
        return false;
    }

    true
}

/// Assert response status and parse JSON body
pub async fn assert_json<T: DeserializeOwned>(response: Response, expected_status: StatusCode) -> Result<T> {
    let status = response.status();
    if status != expected_status {
        let body = response.text().await?;
        anyhow::bail!(
            "Expected status {}, got {}. Body: {}",
            expected_status,
            status,
            body
        );
    }
    Ok(response.json().await?)
}

/// Assert response status without parsing body
pub async fn assert_status(response: Response, expected_status: StatusCode) -> Result<()> {
    let status = response.status();
    if status != expected_status {
        let body = response.text().await?;
        anyhow::bail!(
            "Expected status {}, got {}. Body: {}",
            expected_status,
            status,
            body
        );
    }
    Ok(())
}
