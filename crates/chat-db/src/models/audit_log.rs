//! Audit log database model

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

/// Database model for audit_logs table
#[derive(Debug, Clone, FromRow)]
pub struct AuditLogModel {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    /// Audit action type: PostgreSQL enum stored as string
    pub action: String,
    pub target_id: Option<i64>,
    pub target_type: Option<String>,
    /// JSON object containing before/after changes
    pub changes: Option<JsonValue>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
