//! Application state
//!
//! Holds the shared state for the Axum application including
//! the service context and configuration.

use std::sync::Arc;

use chat_common::{AppConfig, JwtService};
use chat_service::ServiceContext;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Service context containing all dependencies
    service_context: Arc<ServiceContext>,
    /// Application configuration
    config: Arc<AppConfig>,
}

impl AppState {
    /// Create a new AppState
    pub fn new(service_context: ServiceContext, config: AppConfig) -> Self {
        Self {
            service_context: Arc::new(service_context),
            config: Arc::new(config),
        }
    }

    /// Get the service context
    pub fn service_context(&self) -> &ServiceContext {
        &self.service_context
    }

    /// Get the application configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get the JWT service from the service context
    pub fn jwt_service(&self) -> &JwtService {
        self.service_context.jwt_service()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("service_context", &"ServiceContext")
            .field("config", &"AppConfig")
            .finish()
    }
}
