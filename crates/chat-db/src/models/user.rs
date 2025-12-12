//! User database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for users table
#[derive(Debug, Clone, FromRow)]
pub struct UserModel {
    pub id: i64,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    pub password_hash: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserModel {
    /// Check if user is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
