//! # chat-common
//!
//! Shared utilities including configuration, error handling, authentication, and telemetry.

pub mod auth;
pub mod config;
pub mod error;
pub mod telemetry;

// Re-export commonly used types at crate root
pub use auth::{
    hash_password, validate_password_strength, verify_password, Claims, JwtService,
    PasswordService, TokenPair, TokenType,
};
pub use config::{
    AppConfig, AppSettings, ConfigError, CorsConfig, DatabaseConfig, Environment, JwtConfig,
    RateLimitConfig, RedisConfig, ServerConfig, SnowflakeConfig, StorageConfig,
};
pub use error::{AppError, AppResult, ErrorResponse};
pub use telemetry::{
    init_tracing, init_tracing_with_config, try_init_tracing, try_init_tracing_with_config,
    TracingConfig, TracingError,
};
