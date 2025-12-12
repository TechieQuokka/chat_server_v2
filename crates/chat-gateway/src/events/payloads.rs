//! Event payload definitions
//!
//! Defines the data structures for each gateway event type.

use chat_core::Snowflake;
use serde::{Deserialize, Serialize};

// === Connection Events ===

/// READY event payload
///
/// Sent after successful Identify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyEvent {
    /// Gateway protocol version
    pub v: i32,

    /// Current user
    pub user: UserPayload,

    /// Guilds the user is in (initially unavailable)
    pub guilds: Vec<UnavailableGuild>,

    /// Session ID for resuming
    pub session_id: String,

    /// Gateway URL for resuming (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_gateway_url: Option<String>,
}

/// RESUMED event payload
///
/// Sent after successful Resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumedEvent {}

/// Unavailable guild in READY event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    pub unavailable: bool,
}

impl UnavailableGuild {
    #[must_use]
    pub fn new(id: Snowflake) -> Self {
        Self {
            id,
            unavailable: true,
        }
    }
}

// === User Payload ===

/// User data included in events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPayload {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
}

// === Guild Events ===

/// GUILD_CREATE event payload
///
/// Sent for each guild on connect, or when joining a new guild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildCreateEvent {
    pub id: Snowflake,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub owner_id: Snowflake,
    #[serde(default)]
    pub channels: Vec<ChannelPayload>,
    #[serde(default)]
    pub roles: Vec<RolePayload>,
    #[serde(default)]
    pub members: Vec<MemberPayload>,
    pub member_count: i32,
    pub created_at: String,
}

/// GUILD_UPDATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildEvent {
    pub id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Snowflake>,
}

/// GUILD_DELETE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDeleteEvent {
    pub id: Snowflake,
    /// If true, this is a temporary outage; if false, the user left/was kicked/guild was deleted
    #[serde(default)]
    pub unavailable: bool,
}

// === Channel Events ===

/// Channel data included in events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPayload {
    pub id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Snowflake>,
}

/// CHANNEL_CREATE/CHANNEL_UPDATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEvent {
    pub id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Snowflake>,
}

/// CHANNEL_DELETE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeleteEvent {
    pub id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(rename = "type")]
    pub channel_type: i32,
}

// === Role Events ===

/// Role data included in events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePayload {
    pub id: Snowflake,
    pub name: String,
    pub permissions: String,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<i32>,
}

// === Message Events ===

/// MESSAGE_CREATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreateEvent {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    pub author: UserPayload,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub attachments: Vec<AttachmentPayload>,
    #[serde(default)]
    pub reactions: Vec<ReactionPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReferencePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_message: Option<Box<MessageCreateEvent>>,
}

/// MESSAGE_UPDATE event payload (partial update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_timestamp: Option<String>,
}

/// MESSAGE_DELETE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeleteEvent {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
}

/// Attachment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPayload {
    pub id: Snowflake,
    pub filename: String,
    pub size: i64,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Reaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionPayload {
    pub emoji: String,
    pub count: i32,
}

/// Message reference for replies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReferencePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
}

// === Reaction Events ===

/// MESSAGE_REACTION_ADD/REMOVE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReactionEvent {
    pub user_id: Snowflake,
    pub channel_id: Snowflake,
    pub message_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    pub emoji: String,
}

// === Member Events ===

/// Member data included in events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPayload {
    pub user: UserPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default)]
    pub roles: Vec<Snowflake>,
    pub joined_at: String,
}

/// GUILD_MEMBER_ADD event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberAddEvent {
    pub guild_id: Snowflake,
    pub user: UserPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default)]
    pub roles: Vec<Snowflake>,
    pub joined_at: String,
}

/// GUILD_MEMBER_UPDATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: Snowflake,
    pub user: UserIdPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<Snowflake>>,
}

/// GUILD_MEMBER_REMOVE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: Snowflake,
    pub user: UserPayload,
}

/// Partial user with just ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdPayload {
    pub id: Snowflake,
}

/// Convenience type for member events
pub type MemberEvent = GuildMemberUpdateEvent;

// === Presence Events ===

/// PRESENCE_UPDATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub user: UserIdPayload,
    pub guild_id: Snowflake,
    pub status: String,
}

/// TYPING_START event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStartEvent {
    pub channel_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    /// Unix timestamp in seconds
    pub timestamp: i64,
}

// === User Events ===

/// USER_UPDATE event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unavailable_guild() {
        let guild = UnavailableGuild::new(Snowflake::from(12345i64));
        assert!(guild.unavailable);
        assert_eq!(guild.id, Snowflake::from(12345i64));
    }

    #[test]
    fn test_ready_event_serialization() {
        let ready = ReadyEvent {
            v: 1,
            user: UserPayload {
                id: Snowflake::from(12345i64),
                username: "testuser".to_string(),
                discriminator: "0001".to_string(),
                avatar: None,
                bot: false,
            },
            guilds: vec![UnavailableGuild::new(Snowflake::from(67890i64))],
            session_id: "session123".to_string(),
            resume_gateway_url: Some("wss://gateway.example.com".to_string()),
        };

        let json = serde_json::to_string(&ready).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("session123"));
    }

    #[test]
    fn test_message_create_event() {
        let msg = MessageCreateEvent {
            id: Snowflake::from(1i64),
            channel_id: Snowflake::from(2i64),
            guild_id: Some(Snowflake::from(3i64)),
            author: UserPayload {
                id: Snowflake::from(4i64),
                username: "user".to_string(),
                discriminator: "0001".to_string(),
                avatar: None,
                bot: false,
            },
            content: "Hello!".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            edited_timestamp: None,
            attachments: vec![],
            reactions: vec![],
            message_reference: None,
            referenced_message: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello!"));
    }

    #[test]
    fn test_presence_event() {
        let presence = PresenceEvent {
            user: UserIdPayload {
                id: Snowflake::from(12345i64),
            },
            guild_id: Snowflake::from(67890i64),
            status: "online".to_string(),
        };

        let json = serde_json::to_string(&presence).unwrap();
        assert!(json.contains("online"));
    }
}
