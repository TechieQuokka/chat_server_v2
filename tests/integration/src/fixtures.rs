//! Test fixtures and data generators
//!
//! Provides reusable test data for integration tests.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter for unique test data
static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Get a unique suffix for test data
pub fn unique_suffix() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Registration request
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn unique() -> Self {
        let suffix = unique_suffix();
        Self {
            username: format!("testuser{suffix}"),
            email: format!("test{suffix}@example.com"),
            password: "TestPass123!".to_string(),
        }
    }
}

/// Login request
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn from_register(reg: &RegisterRequest) -> Self {
        Self {
            email: reg.email.clone(),
            password: reg.password.clone(),
        }
    }
}

/// Auth response
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

/// User response
#[derive(Debug, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub bot: bool,
    pub created_at: String,
}

/// Create guild request
#[derive(Debug, Serialize)]
pub struct CreateGuildRequest {
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

impl CreateGuildRequest {
    pub fn unique() -> Self {
        let suffix = unique_suffix();
        Self {
            name: format!("Test Guild {suffix}"),
            icon: None,
            description: Some("A test guild".to_string()),
        }
    }
}

/// Guild response
#[derive(Debug, Deserialize)]
pub struct GuildResponse {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: String,
}

/// Create channel request
#[derive(Debug, Serialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub topic: Option<String>,
    pub parent_id: Option<String>,
}

impl CreateChannelRequest {
    pub fn text_channel() -> Self {
        let suffix = unique_suffix();
        Self {
            name: format!("test-channel-{suffix}"),
            channel_type: 0, // GuildText
            topic: Some("Test topic".to_string()),
            parent_id: None,
        }
    }

    pub fn category() -> Self {
        let suffix = unique_suffix();
        Self {
            name: format!("Test Category {suffix}"),
            channel_type: 4, // GuildCategory
            topic: None,
            parent_id: None,
        }
    }
}

/// Channel response
#[derive(Debug, Deserialize)]
pub struct ChannelResponse {
    pub id: String,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<String>,
}

/// Create message request
#[derive(Debug, Serialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub message_reference: Option<MessageReference>,
}

impl CreateMessageRequest {
    pub fn simple(content: &str) -> Self {
        Self {
            content: content.to_string(),
            message_reference: None,
        }
    }

    pub fn reply(content: &str, message_id: &str) -> Self {
        Self {
            content: content.to_string(),
            message_reference: Some(MessageReference {
                message_id: message_id.to_string(),
            }),
        }
    }
}

/// Message reference for replies
#[derive(Debug, Serialize)]
pub struct MessageReference {
    pub message_id: String,
}

/// Message response
#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub author: UserResponse,
    pub content: String,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub reference_id: Option<String>,
}

/// Create role request
#[derive(Debug, Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub color: Option<i32>,
    pub hoist: Option<bool>,
    pub mentionable: Option<bool>,
    pub permissions: Option<String>,
}

impl CreateRoleRequest {
    pub fn unique() -> Self {
        let suffix = unique_suffix();
        Self {
            name: format!("Test Role {suffix}"),
            color: Some(0x0034_98db),
            hoist: Some(true),
            mentionable: Some(true),
            permissions: None,
        }
    }
}

/// Role response
#[derive(Debug, Deserialize)]
pub struct RoleResponse {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: String,
    pub mentionable: bool,
}

/// Token refresh request
#[derive(Debug, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Token pair response
#[derive(Debug, Deserialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// Create invite request
#[derive(Debug, Serialize)]
pub struct CreateInviteRequest {
    pub max_age: Option<i32>,
    pub max_uses: Option<i32>,
    pub temporary: Option<bool>,
}

impl Default for CreateInviteRequest {
    fn default() -> Self {
        Self {
            max_age: Some(86400),
            max_uses: Some(10),
            temporary: Some(false),
        }
    }
}

/// Invite response
#[derive(Debug, Deserialize)]
pub struct InviteResponse {
    pub code: String,
    pub guild_id: String,
    pub channel_id: String,
    pub inviter_id: String,
    pub uses: i32,
    pub max_uses: Option<i32>,
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// Member response
#[derive(Debug, Deserialize)]
pub struct MemberResponse {
    pub user: UserResponse,
    pub guild_id: String,
    pub nickname: Option<String>,
    pub role_ids: Vec<String>,
    pub joined_at: String,
}

/// Error response
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}
