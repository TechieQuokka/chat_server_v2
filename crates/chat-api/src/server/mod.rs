//! Server setup and initialization
//!
//! Provides the main application builder and server runner.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use chat_cache::{RedisPool, RedisPoolConfig};
use chat_common::{AppConfig, AppError, JwtService};
use chat_core::SnowflakeGenerator;
use chat_db::{
    create_pool, PgAttachmentRepository, PgBanRepository, PgChannelRepository, PgGuildRepository,
    PgInviteRepository, PgMemberRepository, PgMessageRepository, PgReactionRepository,
    PgRoleRepository, PgUserRepository,
};
use chat_service::ServiceContextBuilder;
use tokio::net::TcpListener;
use tracing::info;

use crate::middleware::apply_middleware;
use crate::routes::create_router;
use crate::state::AppState;

/// Build the complete Axum application with all routes and middleware
pub fn create_app(state: AppState) -> Router {
    let router = create_router();
    let router = apply_middleware(router);
    router.with_state(state)
}

/// Initialize all dependencies and create AppState
pub async fn create_app_state(config: AppConfig) -> Result<AppState, AppError> {
    // Create database pool
    info!("Connecting to PostgreSQL...");
    let db_config = chat_db::DatabaseConfig {
        url: config.database.url.clone(),
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        ..Default::default()
    };
    let pool = create_pool(&db_config)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    info!("PostgreSQL connection established");

    // Create Redis pool
    info!("Connecting to Redis...");
    let redis_config = RedisPoolConfig::from(&config.redis);
    let redis_pool = RedisPool::new(redis_config)
        .map_err(|e| AppError::Cache(e.to_string()))?;
    let shared_redis = Arc::new(redis_pool);
    info!("Redis connection established");

    // Create JWT service
    let jwt_service = Arc::new(JwtService::new(
        &config.jwt.secret,
        config.jwt.access_token_expiry,
        config.jwt.refresh_token_expiry,
    ));

    // Create Snowflake generator
    let snowflake_generator = Arc::new(SnowflakeGenerator::new(config.snowflake.worker_id));

    // Create repositories
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let guild_repo = Arc::new(PgGuildRepository::new(pool.clone()));
    let channel_repo = Arc::new(PgChannelRepository::new(pool.clone()));
    let message_repo = Arc::new(PgMessageRepository::new(pool.clone()));
    let role_repo = Arc::new(PgRoleRepository::new(pool.clone()));
    let member_repo = Arc::new(PgMemberRepository::new(pool.clone()));
    let reaction_repo = Arc::new(PgReactionRepository::new(pool.clone()));
    let invite_repo = Arc::new(PgInviteRepository::new(pool.clone()));
    let ban_repo = Arc::new(PgBanRepository::new(pool.clone()));
    let attachment_repo = Arc::new(PgAttachmentRepository::new(pool.clone()));

    // Build service context
    let service_context = ServiceContextBuilder::new()
        .pool(pool)
        .redis_pool(shared_redis)
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

    Ok(AppState::new(service_context, config))
}

/// Run the HTTP server
pub async fn run_server(app: Router, addr: SocketAddr) -> Result<(), AppError> {
    info!("Starting HTTP server on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Config(format!("Failed to bind to {}: {}", addr, e)))?;

    info!("Server listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Config(format!("Server error: {}", e)))?;

    Ok(())
}

/// Run the complete server with configuration
pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.api.port));

    // Create app state
    let state = create_app_state(config).await?;

    // Build application
    let app = create_app(state);

    // Run server
    run_server(app, addr).await
}
