//! Gateway server setup
//!
//! Provides the main WebSocket server configuration and routes.

mod handler;
mod state;

pub use handler::gateway_handler;
pub use state::GatewayState;

use crate::broadcast::{EventDispatcher, EventDispatcherConfig};
use crate::connection::ConnectionManager;
use axum::{routing::get, Router};
use chat_cache::{RedisPool, RedisPoolConfig};
use chat_common::{AppConfig, AppError};
use chat_service::ServiceContextBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Create the gateway router
pub fn create_router() -> Router<GatewayState> {
    Router::new()
        .route("/gateway", get(gateway_handler))
        .route("/health", get(health_check))
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Build the complete application
pub fn create_app(state: GatewayState) -> Router {
    create_router()
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Initialize all dependencies and create `GatewayState`
pub async fn create_gateway_state(config: AppConfig) -> Result<GatewayState, AppError> {
    // Create database pool
    tracing::info!("Connecting to PostgreSQL...");
    let db_config = chat_db::DatabaseConfig {
        url: config.database.url.clone(),
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        ..Default::default()
    };
    let pool = chat_db::create_pool(&db_config)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    tracing::info!("PostgreSQL connection established");

    // Create Redis pool
    tracing::info!("Connecting to Redis...");
    let redis_config = RedisPoolConfig::from(&config.redis);
    let redis_pool = RedisPool::new(redis_config).map_err(|e| AppError::Cache(e.to_string()))?;
    let shared_redis = Arc::new(redis_pool);
    tracing::info!("Redis connection established");

    // Create JWT service
    let jwt_service = Arc::new(chat_common::JwtService::new(
        &config.jwt.secret,
        config.jwt.access_token_expiry,
        config.jwt.refresh_token_expiry,
    ));

    // Create Snowflake generator
    let snowflake_generator = Arc::new(chat_core::SnowflakeGenerator::new(config.snowflake.worker_id));

    // Create repositories
    let user_repo = Arc::new(chat_db::PgUserRepository::new(pool.clone()));
    let guild_repo = Arc::new(chat_db::PgGuildRepository::new(pool.clone()));
    let channel_repo = Arc::new(chat_db::PgChannelRepository::new(pool.clone()));
    let message_repo = Arc::new(chat_db::PgMessageRepository::new(pool.clone()));
    let role_repo = Arc::new(chat_db::PgRoleRepository::new(pool.clone()));
    let member_repo = Arc::new(chat_db::PgMemberRepository::new(pool.clone()));
    let reaction_repo = Arc::new(chat_db::PgReactionRepository::new(pool.clone()));
    let invite_repo = Arc::new(chat_db::PgInviteRepository::new(pool.clone()));
    let ban_repo = Arc::new(chat_db::PgBanRepository::new(pool.clone()));
    let attachment_repo = Arc::new(chat_db::PgAttachmentRepository::new(pool.clone()));

    // Build service context
    let service_context = ServiceContextBuilder::new()
        .pool(pool)
        .redis_pool(shared_redis.clone())
        .user_repo(user_repo)
        .guild_repo(guild_repo)
        .channel_repo(channel_repo)
        .message_repo(message_repo)
        .role_repo(role_repo)
        .member_repo(member_repo)
        .reaction_repo(reaction_repo)
        .invite_repo(invite_repo)
        .ban_repo(ban_repo)
        .attachment_repo(attachment_repo)
        .jwt_service(jwt_service)
        .snowflake_generator(snowflake_generator)
        .build()
        .map_err(|e| AppError::Config(e.to_string()))?;

    // Create connection manager
    let connection_manager = ConnectionManager::new_shared();

    // Create event dispatcher
    let dispatcher_config = EventDispatcherConfig {
        redis_url: config.redis.url.clone(),
        broadcast_buffer: 1024,
        reconnect_delay_ms: 1000,
    };

    let event_dispatcher = EventDispatcher::new(dispatcher_config, connection_manager.clone())
        .await
        .map_err(|e| AppError::Cache(format!("Failed to create event dispatcher: {e}")))?;

    let event_dispatcher = Arc::new(event_dispatcher);

    // Start the event dispatcher
    event_dispatcher.clone().start();

    Ok(GatewayState::new(
        service_context,
        connection_manager,
        event_dispatcher,
        config,
    ))
}

/// Run the gateway server
pub async fn run_server(app: Router, addr: SocketAddr) -> Result<(), AppError> {
    tracing::info!("Starting Gateway server on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Config(format!("Failed to bind to {addr}: {e}")))?;

    tracing::info!("Gateway listening on ws://{}/gateway", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Config(format!("Server error: {e}")))?;

    Ok(())
}

/// Run the complete gateway server with configuration
pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.gateway.port));

    // Create gateway state
    let state = create_gateway_state(config).await?;

    // Build application
    let app = create_app(state);

    // Run server
    run_server(app, addr).await
}
