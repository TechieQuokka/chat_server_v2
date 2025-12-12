//! Invite database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for invites table
#[derive(Debug, Clone, FromRow)]
pub struct InviteModel {
    pub code: String,
    pub guild_id: i64,
    pub channel_id: i64,
    pub inviter_id: i64,
    pub uses: i32,
    pub max_uses: Option<i32>,
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl InviteModel {
    /// Check if invite is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if invite is expired
    #[inline]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if invite has reached max uses
    #[inline]
    pub fn is_exhausted(&self) -> bool {
        if let Some(max_uses) = self.max_uses {
            self.uses >= max_uses
        } else {
            false
        }
    }
}
