//! Ban database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for bans table
#[derive(Debug, Clone, FromRow)]
pub struct BanModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub reason: Option<String>,
    pub banned_by: i64,
    pub created_at: DateTime<Utc>,
}
