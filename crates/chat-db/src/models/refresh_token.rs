//! Refresh token database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for refresh_tokens table
#[derive(Debug, Clone, FromRow)]
pub struct RefreshTokenModel {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshTokenModel {
    /// Check if token is revoked
    #[inline]
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Check if token is expired
    #[inline]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid (not revoked and not expired)
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.is_revoked() && !self.is_expired()
    }
}
