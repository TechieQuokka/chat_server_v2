//! Domain events - events emitted when domain state changes
//!
//! These events are used for:
//! - Notifying WebSocket clients of real-time updates
//! - Triggering side effects (e.g., cache invalidation)
//! - Audit logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::Snowflake;

/// All possible domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DomainEvent {
    // =========================================================================
    // User Events
    // =========================================================================
    UserCreated(UserCreatedEvent),
    UserUpdated(UserUpdatedEvent),
    UserDeleted(UserDeletedEvent),

    // =========================================================================
    // Guild Events
    // =========================================================================
    GuildCreated(GuildCreatedEvent),
    GuildUpdated(GuildUpdatedEvent),
    GuildDeleted(GuildDeletedEvent),

    // =========================================================================
    // Channel Events
    // =========================================================================
    ChannelCreated(ChannelCreatedEvent),
    ChannelUpdated(ChannelUpdatedEvent),
    ChannelDeleted(ChannelDeletedEvent),

    // =========================================================================
    // Message Events
    // =========================================================================
    MessageCreated(MessageCreatedEvent),
    MessageUpdated(MessageUpdatedEvent),
    MessageDeleted(MessageDeletedEvent),
    MessageBulkDeleted(MessageBulkDeletedEvent),

    // =========================================================================
    // Member Events
    // =========================================================================
    MemberJoined(MemberJoinedEvent),
    MemberLeft(MemberLeftEvent),
    MemberUpdated(MemberUpdatedEvent),
    MemberKicked(MemberKickedEvent),
    MemberBanned(MemberBannedEvent),
    MemberUnbanned(MemberUnbannedEvent),

    // =========================================================================
    // Role Events
    // =========================================================================
    RoleCreated(RoleCreatedEvent),
    RoleUpdated(RoleUpdatedEvent),
    RoleDeleted(RoleDeletedEvent),

    // =========================================================================
    // Reaction Events
    // =========================================================================
    ReactionAdded(ReactionAddedEvent),
    ReactionRemoved(ReactionRemovedEvent),
    ReactionsBulkRemoved(ReactionsBulkRemovedEvent),

    // =========================================================================
    // Invite Events
    // =========================================================================
    InviteCreated(InviteCreatedEvent),
    InviteDeleted(InviteDeletedEvent),

    // =========================================================================
    // Presence Events
    // =========================================================================
    PresenceUpdated(PresenceUpdatedEvent),
    TypingStarted(TypingStartedEvent),
}

impl DomainEvent {
    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::UserCreated(_) => "USER_CREATED",
            Self::UserUpdated(_) => "USER_UPDATED",
            Self::UserDeleted(_) => "USER_DELETED",
            Self::GuildCreated(_) => "GUILD_CREATED",
            Self::GuildUpdated(_) => "GUILD_UPDATED",
            Self::GuildDeleted(_) => "GUILD_DELETED",
            Self::ChannelCreated(_) => "CHANNEL_CREATED",
            Self::ChannelUpdated(_) => "CHANNEL_UPDATED",
            Self::ChannelDeleted(_) => "CHANNEL_DELETED",
            Self::MessageCreated(_) => "MESSAGE_CREATED",
            Self::MessageUpdated(_) => "MESSAGE_UPDATED",
            Self::MessageDeleted(_) => "MESSAGE_DELETED",
            Self::MessageBulkDeleted(_) => "MESSAGE_BULK_DELETED",
            Self::MemberJoined(_) => "MEMBER_JOINED",
            Self::MemberLeft(_) => "MEMBER_LEFT",
            Self::MemberUpdated(_) => "MEMBER_UPDATED",
            Self::MemberKicked(_) => "MEMBER_KICKED",
            Self::MemberBanned(_) => "MEMBER_BANNED",
            Self::MemberUnbanned(_) => "MEMBER_UNBANNED",
            Self::RoleCreated(_) => "ROLE_CREATED",
            Self::RoleUpdated(_) => "ROLE_UPDATED",
            Self::RoleDeleted(_) => "ROLE_DELETED",
            Self::ReactionAdded(_) => "REACTION_ADDED",
            Self::ReactionRemoved(_) => "REACTION_REMOVED",
            Self::ReactionsBulkRemoved(_) => "REACTIONS_BULK_REMOVED",
            Self::InviteCreated(_) => "INVITE_CREATED",
            Self::InviteDeleted(_) => "INVITE_DELETED",
            Self::PresenceUpdated(_) => "PRESENCE_UPDATED",
            Self::TypingStarted(_) => "TYPING_STARTED",
        }
    }

    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::UserCreated(e) => e.timestamp,
            Self::UserUpdated(e) => e.timestamp,
            Self::UserDeleted(e) => e.timestamp,
            Self::GuildCreated(e) => e.timestamp,
            Self::GuildUpdated(e) => e.timestamp,
            Self::GuildDeleted(e) => e.timestamp,
            Self::ChannelCreated(e) => e.timestamp,
            Self::ChannelUpdated(e) => e.timestamp,
            Self::ChannelDeleted(e) => e.timestamp,
            Self::MessageCreated(e) => e.timestamp,
            Self::MessageUpdated(e) => e.timestamp,
            Self::MessageDeleted(e) => e.timestamp,
            Self::MessageBulkDeleted(e) => e.timestamp,
            Self::MemberJoined(e) => e.timestamp,
            Self::MemberLeft(e) => e.timestamp,
            Self::MemberUpdated(e) => e.timestamp,
            Self::MemberKicked(e) => e.timestamp,
            Self::MemberBanned(e) => e.timestamp,
            Self::MemberUnbanned(e) => e.timestamp,
            Self::RoleCreated(e) => e.timestamp,
            Self::RoleUpdated(e) => e.timestamp,
            Self::RoleDeleted(e) => e.timestamp,
            Self::ReactionAdded(e) => e.timestamp,
            Self::ReactionRemoved(e) => e.timestamp,
            Self::ReactionsBulkRemoved(e) => e.timestamp,
            Self::InviteCreated(e) => e.timestamp,
            Self::InviteDeleted(e) => e.timestamp,
            Self::PresenceUpdated(e) => e.timestamp,
            Self::TypingStarted(e) => e.timestamp,
        }
    }
}

// ============================================================================
// Event Structs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdatedEvent {
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildCreatedEvent {
    pub guild_id: Snowflake,
    pub owner_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildUpdatedEvent {
    pub guild_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDeletedEvent {
    pub guild_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCreatedEvent {
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUpdatedEvent {
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDeletedEvent {
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreatedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub author_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdatedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeletedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBulkDeletedEvent {
    pub message_ids: Vec<Snowflake>,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberJoinedEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberLeftEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberUpdatedEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberKickedEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub kicked_by: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberBannedEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub banned_by: Snowflake,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberUnbannedEvent {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub unbanned_by: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCreatedEvent {
    pub role_id: Snowflake,
    pub guild_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleUpdatedEvent {
    pub role_id: Snowflake,
    pub guild_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDeletedEvent {
    pub role_id: Snowflake,
    pub guild_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionAddedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    pub emoji: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRemovedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    pub emoji: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionsBulkRemovedEvent {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub emoji: Option<String>, // None = all reactions removed
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCreatedEvent {
    pub code: String,
    pub guild_id: Snowflake,
    pub channel_id: Snowflake,
    pub inviter_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteDeletedEvent {
    pub code: String,
    pub guild_id: Snowflake,
    pub channel_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdatedEvent {
    pub user_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub status: String, // online, idle, dnd, offline
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStartedEvent {
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub user_id: Snowflake,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// Event Creation Helpers
// ============================================================================

impl UserCreatedEvent {
    pub fn new(user_id: Snowflake) -> Self {
        Self {
            user_id,
            timestamp: Utc::now(),
        }
    }
}

impl GuildCreatedEvent {
    pub fn new(guild_id: Snowflake, owner_id: Snowflake) -> Self {
        Self {
            guild_id,
            owner_id,
            timestamp: Utc::now(),
        }
    }
}

impl MessageCreatedEvent {
    pub fn new(
        message_id: Snowflake,
        channel_id: Snowflake,
        guild_id: Option<Snowflake>,
        author_id: Snowflake,
    ) -> Self {
        Self {
            message_id,
            channel_id,
            guild_id,
            author_id,
            timestamp: Utc::now(),
        }
    }
}

impl MemberJoinedEvent {
    pub fn new(guild_id: Snowflake, user_id: Snowflake) -> Self {
        Self {
            guild_id,
            user_id,
            timestamp: Utc::now(),
        }
    }
}

impl TypingStartedEvent {
    pub fn new(channel_id: Snowflake, guild_id: Option<Snowflake>, user_id: Snowflake) -> Self {
        Self {
            channel_id,
            guild_id,
            user_id,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = DomainEvent::MessageCreated(MessageCreatedEvent::new(
            Snowflake::new(1),
            Snowflake::new(2),
            Some(Snowflake::new(3)),
            Snowflake::new(4),
        ));

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("MESSAGE_CREATED"));

        let parsed: DomainEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_type(), "MESSAGE_CREATED");
    }

    #[test]
    fn test_event_type() {
        let event = DomainEvent::UserCreated(UserCreatedEvent::new(Snowflake::new(1)));
        assert_eq!(event.event_type(), "USER_CREATED");
    }
}
