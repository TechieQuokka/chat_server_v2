//! Request DTOs for API endpoints
//!
//! All request DTOs implement `Deserialize` and `Validate` for input validation.

use serde::Deserialize;
use validator::Validate;

// ============================================================================
// Auth Requests
// ============================================================================

/// User registration request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 32, message = "Username must be 2-32 characters"))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, max = 72, message = "Password must be 8-72 characters"))]
    pub password: String,
}

/// User login request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub password: String,
}

/// Token refresh request
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Logout request (optional refresh token to revoke)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

// ============================================================================
// User Requests
// ============================================================================

/// Update current user request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2, max = 32, message = "Username must be 2-32 characters"))]
    pub username: Option<String>,

    /// Avatar hash or null to remove
    pub avatar: Option<String>,
}

// ============================================================================
// Guild Requests
// ============================================================================

/// Create guild request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateGuildRequest {
    #[validate(length(min = 1, max = 100, message = "Guild name must be 1-100 characters"))]
    pub name: String,

    /// Icon hash
    pub icon: Option<String>,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
}

/// Update guild request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateGuildRequest {
    #[validate(length(min = 1, max = 100, message = "Guild name must be 1-100 characters"))]
    pub name: Option<String>,

    /// Icon hash or null to remove
    pub icon: Option<String>,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    /// Transfer ownership to another user (Snowflake ID as string)
    pub owner_id: Option<String>,
}

// ============================================================================
// Channel Requests
// ============================================================================

/// Create channel request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 100, message = "Channel name must be 1-100 characters"))]
    pub name: String,

    /// Channel type: 0 = text, 4 = category (defaults to 0)
    #[serde(rename = "type", default)]
    pub channel_type: i32,

    #[validate(length(max = 1024, message = "Topic must be at most 1024 characters"))]
    pub topic: Option<String>,

    /// Parent category ID (Snowflake as string)
    pub parent_id: Option<String>,

    /// Position in channel list
    pub position: Option<i32>,
}

/// Update channel request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100, message = "Channel name must be 1-100 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 1024, message = "Topic must be at most 1024 characters"))]
    pub topic: Option<String>,

    /// Position in channel list
    pub position: Option<i32>,

    /// Parent category ID (Snowflake as string, null to remove)
    pub parent_id: Option<String>,
}

// ============================================================================
// Message Requests
// ============================================================================

/// Message reference for replies
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReference {
    /// ID of the message being replied to
    pub message_id: String,
}

/// Create message request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "Message must be 1-2000 characters"))]
    pub content: String,

    /// Optional reference to a message being replied to
    pub message_reference: Option<MessageReference>,
}

/// Update message request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 2000, message = "Message must be 1-2000 characters"))]
    pub content: String,
}

/// Bulk delete messages request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct BulkDeleteMessagesRequest {
    /// Message IDs to delete (2-100 messages)
    #[validate(length(min = 2, max = 100, message = "Must delete 2-100 messages"))]
    pub messages: Vec<String>,
}

// ============================================================================
// Role Requests
// ============================================================================

/// Create role request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 100, message = "Role name must be 1-100 characters"))]
    pub name: String,

    /// Role color as integer (RGB)
    #[serde(default)]
    pub color: i32,

    /// Whether to display role members separately
    #[serde(default)]
    pub hoist: bool,

    /// Permissions bitfield as string
    pub permissions: Option<String>,

    /// Whether the role can be mentioned
    #[serde(default)]
    pub mentionable: bool,
}

/// Update role request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100, message = "Role name must be 1-100 characters"))]
    pub name: Option<String>,

    /// Role color as integer (RGB)
    pub color: Option<i32>,

    /// Whether to display role members separately
    pub hoist: Option<bool>,

    /// Permissions bitfield as string
    pub permissions: Option<String>,

    /// Whether the role can be mentioned
    pub mentionable: Option<bool>,

    /// Position in role hierarchy
    pub position: Option<i32>,
}

/// Update role positions request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRolePositionsRequest {
    /// Array of role IDs and their new positions
    pub positions: Vec<RolePosition>,
}

/// Role position entry
#[derive(Debug, Clone, Deserialize)]
pub struct RolePosition {
    /// Role ID (Snowflake as string)
    pub id: String,
    /// New position
    pub position: i32,
}

// ============================================================================
// Member Requests
// ============================================================================

/// Update member request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateMemberRequest {
    /// Nickname (null to remove)
    #[validate(length(max = 32, message = "Nickname must be at most 32 characters"))]
    pub nickname: Option<String>,

    /// Role IDs to set (replaces existing roles)
    pub roles: Option<Vec<String>>,
}

/// Add role to member request
#[derive(Debug, Clone, Deserialize)]
pub struct AddMemberRoleRequest {
    /// Role ID to add
    pub role_id: String,
}

// ============================================================================
// Invite Requests
// ============================================================================

/// Create invite request
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CreateInviteRequest {
    /// Max age in seconds (0 = never expires, default = 86400)
    #[serde(default = "default_invite_max_age")]
    pub max_age: i32,

    /// Max number of uses (0 = unlimited, default = 0)
    #[serde(default)]
    pub max_uses: i32,

    /// Whether membership is temporary
    #[serde(default)]
    pub temporary: bool,
}

impl CreateInviteRequest {
    /// Create a new CreateInviteRequest with default values
    pub fn new() -> Self {
        Self {
            max_age: default_invite_max_age(),
            max_uses: 0,
            temporary: false,
        }
    }
}

fn default_invite_max_age() -> i32 {
    86400 // 24 hours
}

// ============================================================================
// Reaction Requests
// ============================================================================

/// Add reaction request (emoji can be Unicode or custom emoji ID)
#[derive(Debug, Clone, Deserialize)]
pub struct AddReactionRequest {
    /// Emoji (Unicode emoji or custom emoji ID)
    pub emoji: String,
}

// ============================================================================
// DM Requests
// ============================================================================

/// Create DM channel request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDmRequest {
    /// Recipient user ID (Snowflake as string)
    pub recipient_id: String,
}

// ============================================================================
// Presence Requests
// ============================================================================

/// Update presence request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePresenceRequest {
    /// Status: online, idle, dnd, offline
    pub status: String,

    /// Custom status message
    pub custom_status: Option<String>,
}

/// Typing indicator request (typically just POST to endpoint, no body)
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypingRequest {}

// ============================================================================
// Ban Requests
// ============================================================================

/// Create ban request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBanRequest {
    /// Reason for the ban
    #[validate(length(max = 512, message = "Reason must be at most 512 characters"))]
    pub reason: Option<String>,

    /// Number of days of messages to delete (0-7)
    #[serde(default)]
    pub delete_message_days: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_register_request_validation() {
        // Valid request
        let valid = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Invalid - username too short
        let short_username = RegisterRequest {
            username: "a".to_string(),
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(short_username.validate().is_err());

        // Invalid - bad email
        let bad_email = RegisterRequest {
            username: "testuser".to_string(),
            email: "not-an-email".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(bad_email.validate().is_err());

        // Invalid - password too short
        let short_password = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(short_password.validate().is_err());
    }

    #[test]
    fn test_create_message_validation() {
        // Valid message
        let valid = CreateMessageRequest {
            content: "Hello, world!".to_string(),
            message_reference: None,
        };
        assert!(valid.validate().is_ok());

        // Invalid - empty message
        let empty = CreateMessageRequest {
            content: "".to_string(),
            message_reference: None,
        };
        assert!(empty.validate().is_err());

        // Invalid - message too long
        let too_long = CreateMessageRequest {
            content: "a".repeat(2001),
            message_reference: None,
        };
        assert!(too_long.validate().is_err());
    }

    #[test]
    fn test_create_guild_validation() {
        let valid = CreateGuildRequest {
            name: "My Guild".to_string(),
            icon: None,
            description: Some("A cool guild".to_string()),
        };
        assert!(valid.validate().is_ok());

        let empty_name = CreateGuildRequest {
            name: "".to_string(),
            icon: None,
            description: None,
        };
        assert!(empty_name.validate().is_err());
    }

    #[test]
    fn test_bulk_delete_validation() {
        // Valid - 2 messages
        let valid = BulkDeleteMessagesRequest {
            messages: vec!["123".to_string(), "456".to_string()],
        };
        assert!(valid.validate().is_ok());

        // Invalid - only 1 message
        let too_few = BulkDeleteMessagesRequest {
            messages: vec!["123".to_string()],
        };
        assert!(too_few.validate().is_err());
    }
}
