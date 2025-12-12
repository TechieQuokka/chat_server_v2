//! Gateway state
//!
//! Application state for the gateway server.

use crate::broadcast::EventDispatcher;
use crate::connection::ConnectionManager;
use chat_common::AppConfig;
use chat_service::ServiceContext;
use std::sync::Arc;

/// Gateway application state
///
/// Holds all shared dependencies for the gateway server.
#[derive(Clone)]
pub struct GatewayState {
    /// Service context with repositories and services
    service_context: Arc<ServiceContext>,
    /// Connection manager for WebSocket connections
    connection_manager: Arc<ConnectionManager>,
    /// Event dispatcher for Redis Pub/Sub
    event_dispatcher: Arc<EventDispatcher>,
    /// Application configuration
    config: Arc<AppConfig>,
}

impl GatewayState {
    /// Create a new gateway state
    pub fn new(
        service_context: ServiceContext,
        connection_manager: Arc<ConnectionManager>,
        event_dispatcher: Arc<EventDispatcher>,
        config: AppConfig,
    ) -> Self {
        Self {
            service_context: Arc::new(service_context),
            connection_manager,
            event_dispatcher,
            config: Arc::new(config),
        }
    }

    /// Get the service context
    pub fn service_context(&self) -> &ServiceContext {
        &self.service_context
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.connection_manager
    }

    /// Get the event dispatcher
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.event_dispatcher
    }

    /// Get the application configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

impl std::fmt::Debug for GatewayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayState")
            .field("connection_manager", &self.connection_manager)
            .field("config", &"AppConfig")
            .finish()
    }
}
