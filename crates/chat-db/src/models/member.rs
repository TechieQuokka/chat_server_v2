//! Member database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for guild_members table
#[derive(Debug, Clone, FromRow)]
pub struct GuildMemberModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub nickname: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for member_roles table
#[derive(Debug, Clone, FromRow)]
pub struct MemberRoleModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub role_id: i64,
    pub assigned_at: DateTime<Utc>,
}

/// Extended member model with aggregated role IDs (from view or query)
#[derive(Debug, Clone, FromRow)]
pub struct MemberWithRolesModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub nickname: Option<String>,
    pub joined_at: DateTime<Utc>,
    #[sqlx(default)]
    pub role_ids: Vec<i64>,
}
