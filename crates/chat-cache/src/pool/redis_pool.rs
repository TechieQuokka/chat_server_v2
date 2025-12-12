//! Redis connection pool using deadpool-redis.
//!
//! Provides a managed pool of Redis connections for efficient resource usage.

use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use std::sync::Arc;

/// Redis pool configuration
#[derive(Debug, Clone)]
pub struct RedisPoolConfig {
    /// Redis connection URL (e.g., `redis://localhost:6379`)
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: usize,
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 16,
        }
    }
}

impl From<&chat_common::RedisConfig> for RedisPoolConfig {
    fn from(config: &chat_common::RedisConfig) -> Self {
        Self {
            url: config.url.clone(),
            max_connections: config.max_connections as usize,
        }
    }
}

/// Error type for Redis pool operations
#[derive(Debug, thiserror::Error)]
pub enum RedisPoolError {
    #[error("Failed to create Redis pool: {0}")]
    CreatePool(String),

    #[error("Failed to get connection from pool: {0}")]
    GetConnection(#[from] deadpool_redis::PoolError),

    #[error("Redis command error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Connection not available")]
    ConnectionNotAvailable,
}

/// Result type for Redis pool operations
pub type RedisResult<T> = Result<T, RedisPoolError>;

/// Managed Redis connection pool
#[derive(Clone)]
pub struct RedisPool {
    pool: Pool,
}

impl std::fmt::Debug for RedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPool")
            .field("status", &self.pool.status())
            .finish()
    }
}

impl RedisPool {
    /// Create a new Redis pool with the given configuration
    pub fn new(config: RedisPoolConfig) -> RedisResult<Self> {
        let cfg = Config::from_url(&config.url);
        let pool = cfg
            .builder()
            .map_err(|e| RedisPoolError::CreatePool(e.to_string()))?
            .max_size(config.max_connections)
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| RedisPoolError::CreatePool(e.to_string()))?;

        // Redact credentials from URL for logging
        let safe_url = config.url.split('@').next_back().unwrap_or(&config.url);
        tracing::info!(
            url = %safe_url,
            max_connections = config.max_connections,
            "Redis pool created"
        );

        Ok(Self { pool })
    }

    /// Create a new Redis pool from chat-common config
    pub fn from_config(config: &chat_common::RedisConfig) -> RedisResult<Self> {
        Self::new(RedisPoolConfig::from(config))
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> RedisResult<deadpool_redis::Connection> {
        self.pool.get().await.map_err(RedisPoolError::GetConnection)
    }

    /// Get the current pool status
    #[must_use]
    pub fn status(&self) -> deadpool_redis::Status {
        self.pool.status()
    }

    /// Check if the pool is healthy by pinging Redis
    pub async fn health_check(&self) -> RedisResult<()> {
        let mut conn = self.get().await?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await?;
        Ok(())
    }

    /// Set a key-value pair with optional TTL
    pub async fn set<V: serde::Serialize>(
        &self,
        key: &str,
        value: &V,
        ttl_seconds: Option<u64>,
    ) -> RedisResult<()> {
        let mut conn = self.get().await?;
        let serialized = serde_json::to_string(value)?;

        match ttl_seconds {
            Some(ttl) => {
                conn.set_ex::<_, _, ()>(key, &serialized, ttl).await?;
            }
            None => {
                conn.set::<_, _, ()>(key, &serialized).await?;
            }
        }

        Ok(())
    }

    /// Get a value by key
    pub async fn get_value<V: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> RedisResult<Option<V>> {
        let mut conn = self.get().await?;
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    /// Delete a key
    pub async fn delete(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.get().await?;
        let deleted: i32 = conn.del(key).await?;
        Ok(deleted > 0)
    }

    /// Delete multiple keys
    pub async fn delete_many(&self, keys: &[&str]) -> RedisResult<i32> {
        if keys.is_empty() {
            return Ok(0);
        }
        let mut conn = self.get().await?;
        let deleted: i32 = conn.del(keys).await?;
        Ok(deleted)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> RedisResult<bool> {
        let mut conn = self.get().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Set TTL for an existing key
    ///
    /// # Errors
    ///
    /// Returns an error if the TTL value exceeds i64::MAX or the Redis command fails.
    pub async fn expire(&self, key: &str, ttl_seconds: u64) -> RedisResult<bool> {
        let ttl = i64::try_from(ttl_seconds)
            .map_err(|_| RedisPoolError::CreatePool("TTL value too large".to_string()))?;
        let mut conn = self.get().await?;
        let result: bool = conn.expire(key, ttl).await?;
        Ok(result)
    }

    /// Get remaining TTL for a key
    pub async fn ttl(&self, key: &str) -> RedisResult<Option<i64>> {
        let mut conn = self.get().await?;
        let ttl: i64 = conn.ttl(key).await?;
        // Redis returns -2 if key doesn't exist, -1 if no TTL
        if ttl == -2 {
            Ok(None)
        } else {
            Ok(Some(ttl))
        }
    }

    /// Get all keys matching a pattern.
    ///
    /// # Warning
    ///
    /// This uses the Redis `KEYS` command which is O(N) and blocks Redis.
    /// **Do not use in production** for large key spaces. Consider using
    /// `scan_keys` instead for production workloads.
    #[deprecated(
        since = "0.1.0",
        note = "Use scan_keys for production workloads. KEYS command blocks Redis."
    )]
    pub async fn keys(&self, pattern: &str) -> RedisResult<Vec<String>> {
        let mut conn = self.get().await?;
        let keys: Vec<String> = conn.keys(pattern).await?;
        Ok(keys)
    }

    /// Scan keys matching a pattern using cursor-based iteration.
    ///
    /// This is the production-safe alternative to `keys()`.
    pub async fn scan_keys(&self, pattern: &str, count: usize) -> RedisResult<Vec<String>> {
        let mut conn = self.get().await?;
        let mut cursor: u64 = 0;
        let mut all_keys = Vec::new();

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(count)
                .query_async(&mut conn)
                .await?;

            all_keys.extend(keys);
            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }
}

/// Shared Redis pool wrapped in Arc for easy cloning
pub type SharedRedisPool = Arc<RedisPool>;

/// Create a shared Redis pool
pub fn create_shared_pool(config: RedisPoolConfig) -> RedisResult<SharedRedisPool> {
    Ok(Arc::new(RedisPool::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisPoolConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.max_connections, 16);
    }

    #[test]
    fn test_config_from_redis_config() {
        let redis_config = chat_common::RedisConfig {
            url: "redis://localhost:6380".to_string(),
            max_connections: 32,
        };
        let pool_config = RedisPoolConfig::from(&redis_config);
        assert_eq!(pool_config.url, "redis://localhost:6380");
        assert_eq!(pool_config.max_connections, 32);
    }
}
