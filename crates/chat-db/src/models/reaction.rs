//! Reaction database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for reactions table
#[derive(Debug, Clone, FromRow)]
pub struct ReactionModel {
    pub message_id: i64,
    pub user_id: i64,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

/// Aggregated reaction count (from query)
#[derive(Debug, Clone, FromRow)]
pub struct ReactionCountModel {
    pub emoji: String,
    pub count: i64,
}
