//! Application configuration structs
//!
//! Loads configuration from environment variables and config files.

use serde::Deserialize;
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub api: ServerConfig,
    pub gateway: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub storage: StorageConfig,
    pub snowflake: SnowflakeConfig,
}

/// General application settings
#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_env")]
    pub env: Environment,
}

/// Environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Staging,
    Production,
}

impl Environment {
    #[must_use]
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }
}

/// Server configuration (for both API and Gateway)
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_redis_max_connections")]
    pub max_connections: u32,
}

/// JWT configuration
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry: i64,
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry: i64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u32,
    #[serde(default = "default_burst")]
    pub burst: u32,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

/// File storage configuration
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_upload_dir")]
    pub upload_dir: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u32,
}

/// Snowflake ID generator configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SnowflakeConfig {
    #[serde(default)]
    pub worker_id: u16,
}

// Default value functions
fn default_app_name() -> String {
    "chat-server".to_string()
}

fn default_env() -> Environment {
    Environment::Development
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_max_connections() -> u32 {
    20
}

fn default_min_connections() -> u32 {
    5
}

fn default_redis_max_connections() -> u32 {
    10
}

fn default_access_token_expiry() -> i64 {
    900 // 15 minutes
}

fn default_refresh_token_expiry() -> i64 {
    604800 // 7 days
}

fn default_requests_per_second() -> u32 {
    10
}

fn default_burst() -> u32 {
    50
}

fn default_upload_dir() -> String {
    "./uploads".to_string()
}

fn default_max_file_size() -> u32 {
    10
}

impl AppConfig {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns an error if required environment variables are missing
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (ignore errors if not found)
        let _ = dotenvy::dotenv();

        Ok(Self {
            app: AppSettings {
                name: env::var("APP_NAME").unwrap_or_else(|_| default_app_name()),
                env: env::var("APP_ENV")
                    .ok()
                    .and_then(|s| match s.to_lowercase().as_str() {
                        "production" => Some(Environment::Production),
                        "staging" => Some(Environment::Staging),
                        "development" => Some(Environment::Development),
                        _ => None,
                    })
                    .unwrap_or_default(),
            },
            api: ServerConfig {
                host: env::var("API_HOST").unwrap_or_else(|_| default_host()),
                port: env::var("API_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .ok_or(ConfigError::MissingVar("API_PORT"))?,
            },
            gateway: ServerConfig {
                host: env::var("GATEWAY_HOST").unwrap_or_else(|_| default_host()),
                port: env::var("GATEWAY_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .ok_or(ConfigError::MissingVar("GATEWAY_PORT"))?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").map_err(|_| ConfigError::MissingVar("DATABASE_URL"))?,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_max_connections),
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_min_connections),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").map_err(|_| ConfigError::MissingVar("REDIS_URL"))?,
                max_connections: env::var("REDIS_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_redis_max_connections),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").map_err(|_| ConfigError::MissingVar("JWT_SECRET"))?,
                access_token_expiry: env::var("JWT_ACCESS_TOKEN_EXPIRY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_access_token_expiry),
                refresh_token_expiry: env::var("JWT_REFRESH_TOKEN_EXPIRY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_refresh_token_expiry),
            },
            rate_limit: RateLimitConfig {
                requests_per_second: env::var("RATE_LIMIT_REQUESTS_PER_SECOND")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_requests_per_second),
                burst: env::var("RATE_LIMIT_BURST")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_burst),
            },
            cors: CorsConfig {
                allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .ok()
                    .map(|s| s.split(',').map(str::trim).map(String::from).collect())
                    .unwrap_or_default(),
            },
            storage: StorageConfig {
                upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| default_upload_dir()),
                max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(default_max_file_size),
            },
            snowflake: SnowflakeConfig {
                worker_id: env::var("WORKER_ID")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            },
        })
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(&'static str),

    #[error("Invalid value for {0}: {1}")]
    InvalidValue(&'static str, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_is_production() {
        assert!(!Environment::Development.is_production());
        assert!(!Environment::Staging.is_production());
        assert!(Environment::Production.is_production());
    }

    #[test]
    fn test_environment_is_development() {
        assert!(Environment::Development.is_development());
        assert!(!Environment::Staging.is_development());
        assert!(!Environment::Production.is_development());
    }

    #[test]
    fn test_server_address() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
        };
        assert_eq!(config.address(), "0.0.0.0:8080");
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_app_name(), "chat-server");
        assert_eq!(default_host(), "127.0.0.1");
        assert_eq!(default_max_connections(), 20);
        assert_eq!(default_access_token_expiry(), 900);
        assert_eq!(default_refresh_token_expiry(), 604800);
    }
}
