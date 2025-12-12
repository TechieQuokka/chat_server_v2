//! Guild database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for guilds table
#[derive(Debug, Clone, FromRow)]
pub struct GuildModel {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl GuildModel {
    /// Check if guild is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
