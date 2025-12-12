//! Response DTOs for API endpoints
//!
//! All response DTOs implement `Serialize` for JSON output.
//! Snowflake IDs are serialized as strings for JavaScript compatibility.

use chrono::{DateTime, Utc};
use serde::Serialize;

// ============================================================================
// Common Response Types
// ============================================================================

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// Paginated response with cursor-based pagination
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, before: Option<String>, after: Option<String>, has_more: bool, limit: i32) -> Self {
        Self {
            data,
            pagination: PaginationMeta {
                before,
                after,
                has_more,
                limit,
            },
        }
    }
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Cursor for fetching previous page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Cursor for fetching next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Whether more results exist
    pub has_more: bool,
    /// Page size limit used
    pub limit: i32,
}

// ============================================================================
// Auth Responses
// ============================================================================

/// Authentication response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: CurrentUserResponse,
}

impl AuthResponse {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        user: CurrentUserResponse,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            user,
        }
    }
}

// ============================================================================
// User Responses
// ============================================================================

/// Public user response (limited fields)
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
}

/// Current authenticated user response (includes email)
#[derive(Debug, Clone, Serialize)]
pub struct CurrentUserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
}

/// Public user response (for viewing other users)
#[derive(Debug, Clone, Serialize)]
pub struct PublicUserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub bot: bool,
}

// ============================================================================
// Guild Responses
// ============================================================================

/// Basic guild response
#[derive(Debug, Clone, Serialize)]
pub struct GuildResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
}

/// Guild response with member and channel counts
#[derive(Debug, Clone, Serialize)]
pub struct GuildWithCountsResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub owner_id: String,
    pub member_count: i64,
    pub channel_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Guild preview (for invite previews)
#[derive(Debug, Clone, Serialize)]
pub struct GuildPreviewResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub member_count: i64,
}

// ============================================================================
// Channel Responses
// ============================================================================

/// Channel response
#[derive(Debug, Clone, Serialize)]
pub struct ChannelResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// DM channel response with recipients
#[derive(Debug, Clone, Serialize)]
pub struct DmChannelResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub recipients: Vec<UserResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<String>,
}

// ============================================================================
// Message Responses
// ============================================================================

/// Message response
#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    pub author: UserResponse,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentResponse>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<ReactionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReferenceResponse>,
}

/// Attachment response
#[derive(Debug, Clone, Serialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
}

/// Reaction response
#[derive(Debug, Clone, Serialize)]
pub struct ReactionResponse {
    pub emoji: String,
    pub count: i64,
    pub me: bool,
}

/// Message reference response (for replies)
#[derive(Debug, Clone, Serialize)]
pub struct MessageReferenceResponse {
    pub message_id: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
}

// ============================================================================
// Role Responses
// ============================================================================

/// Role response
#[derive(Debug, Clone, Serialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: String,
    pub mentionable: bool,
}

// ============================================================================
// Member Responses
// ============================================================================

/// Guild member response
#[derive(Debug, Clone, Serialize)]
pub struct MemberResponse {
    pub user: UserResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: DateTime<Utc>,
}

/// Ban response
#[derive(Debug, Clone, Serialize)]
pub struct BanResponse {
    pub user: UserResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// ============================================================================
// Invite Responses
// ============================================================================

/// Invite response
#[derive(Debug, Clone, Serialize)]
pub struct InviteResponse {
    pub code: String,
    pub guild: GuildPreviewResponse,
    pub channel: InviteChannelResponse,
    pub inviter: UserResponse,
    pub uses: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Simplified channel info for invite
#[derive(Debug, Clone, Serialize)]
pub struct InviteChannelResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
}

/// Minimal invite response (for invite usage)
#[derive(Debug, Clone, Serialize)]
pub struct InviteMinimalResponse {
    pub code: String,
    pub guild_id: String,
    pub channel_id: String,
}

// ============================================================================
// Presence Responses
// ============================================================================

/// Presence response
#[derive(Debug, Clone, Serialize)]
pub struct PresenceResponse {
    pub user_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<DateTime<Utc>>,
}

/// Typing indicator response
#[derive(Debug, Clone, Serialize)]
pub struct TypingResponse {
    pub channel_id: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Health Responses
// ============================================================================

/// Basic health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

impl HealthResponse {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
        }
    }
}

/// Readiness check response
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub checks: HealthChecks,
}

/// Health check status for each service
#[derive(Debug, Clone, Serialize)]
pub struct HealthChecks {
    pub database: String,
    pub redis: String,
}

impl ReadinessResponse {
    pub fn ready(database_healthy: bool, redis_healthy: bool) -> Self {
        let all_healthy = database_healthy && redis_healthy;
        Self {
            status: if all_healthy { "ready" } else { "not_ready" }.to_string(),
            timestamp: Utc::now(),
            checks: HealthChecks {
                database: if database_healthy { "healthy" } else { "unhealthy" }.to_string(),
                redis: if redis_healthy { "healthy" } else { "unhealthy" }.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response_serialization() {
        let user = CurrentUserResponse {
            id: "123456789".to_string(),
            username: "testuser".to_string(),
            discriminator: "0001".to_string(),
            email: "test@example.com".to_string(),
            avatar: None,
            bot: false,
            system: false,
            created_at: Utc::now(),
        };

        let auth = AuthResponse::new(
            "access_token_here".to_string(),
            "refresh_token_here".to_string(),
            900,
            user,
        );

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("\"token_type\":\"Bearer\""));
        assert!(json.contains("\"expires_in\":900"));
    }

    #[test]
    fn test_paginated_response() {
        let users = vec![
            UserResponse {
                id: "1".to_string(),
                username: "user1".to_string(),
                discriminator: "0001".to_string(),
                avatar: None,
                bot: false,
                system: false,
                created_at: Utc::now(),
            },
        ];

        let response = PaginatedResponse::new(
            users,
            None,
            Some("123".to_string()),
            true,
            50,
        );

        assert!(response.pagination.has_more);
        assert_eq!(response.pagination.limit, 50);
        assert!(response.pagination.before.is_none());
        assert!(response.pagination.after.is_some());
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse::healthy();
        assert_eq!(health.status, "healthy");
    }

    #[test]
    fn test_readiness_response() {
        let ready = ReadinessResponse::ready(true, true);
        assert_eq!(ready.status, "ready");
        assert_eq!(ready.checks.database, "healthy");
        assert_eq!(ready.checks.redis, "healthy");

        let not_ready = ReadinessResponse::ready(true, false);
        assert_eq!(not_ready.status, "not_ready");
        assert_eq!(not_ready.checks.redis, "unhealthy");
    }
}
