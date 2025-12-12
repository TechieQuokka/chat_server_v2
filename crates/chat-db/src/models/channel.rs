//! Channel database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for channels table
#[derive(Debug, Clone, FromRow)]
pub struct ChannelModel {
    pub id: i64,
    pub guild_id: Option<i64>,
    pub name: Option<String>,
    /// Channel type: 'text', 'category', 'dm' (stored as PostgreSQL enum)
    #[sqlx(rename = "type")]
    pub channel_type: String,
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ChannelModel {
    /// Check if channel is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if this is a DM channel
    #[inline]
    pub fn is_dm(&self) -> bool {
        self.channel_type == "dm"
    }

    /// Check if this is a text channel
    #[inline]
    pub fn is_text(&self) -> bool {
        self.channel_type == "text" || self.channel_type == "dm"
    }

    /// Check if this is a category
    #[inline]
    pub fn is_category(&self) -> bool {
        self.channel_type == "category"
    }
}

/// DM channel recipient model
#[derive(Debug, Clone, FromRow)]
pub struct DmRecipientModel {
    pub channel_id: i64,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}
