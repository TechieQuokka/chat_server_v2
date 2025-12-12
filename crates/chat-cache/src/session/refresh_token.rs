//! Refresh token storage in Redis.
//!
//! Stores refresh tokens with automatic expiration for secure session management.

use crate::pool::{RedisPool, RedisResult};
use chat_core::Snowflake;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Key prefix for refresh tokens
const REFRESH_TOKEN_PREFIX: &str = "refresh_token:";

/// Default TTL for refresh tokens (7 days)
const DEFAULT_REFRESH_TOKEN_TTL: u64 = 7 * 24 * 60 * 60;

/// Stored refresh token data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenData {
    /// User ID this token belongs to
    pub user_id: Snowflake,
    /// Session ID (for tracking multiple sessions per user)
    pub session_id: String,
    /// Token creation timestamp (Unix epoch seconds)
    pub created_at: i64,
    /// Client device info (optional)
    pub device_info: Option<String>,
    /// IP address at token creation (optional)
    pub ip_address: Option<String>,
}

impl RefreshTokenData {
    /// Create new refresh token data
    #[must_use]
    pub fn new(user_id: Snowflake, session_id: String) -> Self {
        Self {
            user_id,
            session_id,
            created_at: chrono::Utc::now().timestamp(),
            device_info: None,
            ip_address: None,
        }
    }

    /// Add device info
    #[must_use]
    pub fn with_device_info(mut self, device: impl Into<String>) -> Self {
        self.device_info = Some(device.into());
        self
    }

    /// Add IP address
    #[must_use]
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }
}

/// Refresh token store for managing authentication sessions
#[derive(Clone)]
pub struct RefreshTokenStore {
    pool: RedisPool,
    ttl_seconds: u64,
}

impl RefreshTokenStore {
    /// Create a new refresh token store
    #[must_use]
    pub fn new(pool: RedisPool) -> Self {
        Self {
            pool,
            ttl_seconds: DEFAULT_REFRESH_TOKEN_TTL,
        }
    }

    /// Create with custom TTL
    #[must_use]
    pub fn with_ttl(pool: RedisPool, ttl_seconds: u64) -> Self {
        Self { pool, ttl_seconds }
    }

    /// Generate Redis key for a refresh token
    fn key(token_id: &str) -> String {
        format!("{REFRESH_TOKEN_PREFIX}{token_id}")
    }


    /// Store a refresh token
    pub async fn store(
        &self,
        token_id: &str,
        data: &RefreshTokenData,
    ) -> RedisResult<()> {
        let key = Self::key(token_id);
        self.pool.set(&key, data, Some(self.ttl_seconds)).await?;

        // Also add to user's token set for tracking
        let user_set_key = format!("user_tokens:{}", data.user_id);
        let mut conn = self.pool.get().await?;
        conn.sadd::<_, _, ()>(&user_set_key, token_id).await?;
        conn.expire::<_, ()>(&user_set_key, self.ttl_seconds as i64).await?;

        tracing::debug!(
            token_id = %token_id,
            user_id = %data.user_id,
            session_id = %data.session_id,
            "Stored refresh token"
        );

        Ok(())
    }

    /// Get refresh token data
    pub async fn get(&self, token_id: &str) -> RedisResult<Option<RefreshTokenData>> {
        let key = Self::key(token_id);
        self.pool.get_value(&key).await
    }

    /// Validate and return token data (returns None if expired or invalid)
    pub async fn validate(&self, token_id: &str) -> RedisResult<Option<RefreshTokenData>> {
        self.get(token_id).await
    }

    /// Revoke (delete) a refresh token
    pub async fn revoke(&self, token_id: &str) -> RedisResult<bool> {
        // First get the token to find user_id
        if let Some(data) = self.get(token_id).await? {
            // Remove from user's token set
            let user_set_key = format!("user_tokens:{}", data.user_id);
            let mut conn = self.pool.get().await?;
            conn.srem::<_, _, ()>(&user_set_key, token_id).await?;
        }

        let key = Self::key(token_id);
        let deleted = self.pool.delete(&key).await?;

        if deleted {
            tracing::debug!(token_id = %token_id, "Revoked refresh token");
        }

        Ok(deleted)
    }

    /// Revoke all tokens for a user (logout from all devices)
    pub async fn revoke_all_for_user(&self, user_id: Snowflake) -> RedisResult<u32> {
        let user_set_key = format!("user_tokens:{user_id}");
        let mut conn = self.pool.get().await?;

        // Get all token IDs for this user
        let token_ids: Vec<String> = conn.smembers(&user_set_key).await?;
        let count = token_ids.len() as u32;

        if !token_ids.is_empty() {
            // Delete all tokens
            let keys: Vec<String> = token_ids.iter().map(|id| Self::key(id)).collect();
            let keys_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
            self.pool.delete_many(&keys_refs).await?;
        }

        // Delete the user's token set
        conn.del::<_, ()>(&user_set_key).await?;

        tracing::info!(
            user_id = %user_id,
            count = count,
            "Revoked all refresh tokens for user"
        );

        Ok(count)
    }

    /// Get all active session IDs for a user
    pub async fn get_user_sessions(&self, user_id: Snowflake) -> RedisResult<Vec<String>> {
        let user_set_key = format!("user_tokens:{user_id}");
        let mut conn = self.pool.get().await?;

        let token_ids: Vec<String> = conn.smembers(&user_set_key).await?;
        let mut sessions = Vec::new();

        for token_id in token_ids {
            if let Some(data) = self.get(&token_id).await? {
                sessions.push(data.session_id);
            }
        }

        Ok(sessions)
    }

    /// Refresh a token (extend TTL)
    pub async fn refresh(&self, token_id: &str) -> RedisResult<bool> {
        let key = Self::key(token_id);
        self.pool.expire(&key, self.ttl_seconds).await
    }

    /// Get remaining TTL for a token
    pub async fn get_ttl(&self, token_id: &str) -> RedisResult<Option<i64>> {
        let key = Self::key(token_id);
        self.pool.ttl(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_token_data_creation() {
        let user_id = Snowflake::from(12345i64);
        let data = RefreshTokenData::new(user_id, "session123".to_string())
            .with_device_info("Chrome on Windows")
            .with_ip_address("192.168.1.1");

        assert_eq!(data.user_id, user_id);
        assert_eq!(data.session_id, "session123");
        assert_eq!(data.device_info, Some("Chrome on Windows".to_string()));
        assert_eq!(data.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_key_generation() {
        let key = RefreshTokenStore::key("abc123");
        assert_eq!(key, "refresh_token:abc123");
    }
}
