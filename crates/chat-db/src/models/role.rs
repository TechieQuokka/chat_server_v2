//! Role database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for roles table
#[derive(Debug, Clone, FromRow)]
pub struct RoleModel {
    pub id: i64,
    pub guild_id: i64,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: i64,
    pub mentionable: bool,
    pub is_everyone: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl RoleModel {
    /// Check if role is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
