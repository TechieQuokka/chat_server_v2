//! Gateway event types
//!
//! Defines all event type names for dispatch messages.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Gateway event types
///
/// These are the event names sent in the `t` field of dispatch messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GatewayEventType {
    // Connection events
    /// Sent after successful Identify
    Ready,
    /// Sent after successful Resume
    Resumed,

    // Guild events
    /// Guild available, joined, or created
    GuildCreate,
    /// Guild settings changed
    GuildUpdate,
    /// Left guild, kicked, or guild deleted
    GuildDelete,

    // Channel events
    /// Channel created
    ChannelCreate,
    /// Channel updated
    ChannelUpdate,
    /// Channel deleted
    ChannelDelete,

    // Message events
    /// New message
    MessageCreate,
    /// Message edited
    MessageUpdate,
    /// Message deleted
    MessageDelete,

    // Reaction events
    /// Reaction added
    MessageReactionAdd,
    /// Reaction removed
    MessageReactionRemove,

    // Member events
    /// User joined guild
    GuildMemberAdd,
    /// Member updated (roles, nickname)
    GuildMemberUpdate,
    /// User left guild
    GuildMemberRemove,

    // Presence events
    /// User status changed
    PresenceUpdate,
    /// User started typing
    TypingStart,

    // User events
    /// Current user updated
    UserUpdate,
}

impl GatewayEventType {
    /// Get the string representation of the event type
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Resumed => "RESUMED",
            Self::GuildCreate => "GUILD_CREATE",
            Self::GuildUpdate => "GUILD_UPDATE",
            Self::GuildDelete => "GUILD_DELETE",
            Self::ChannelCreate => "CHANNEL_CREATE",
            Self::ChannelUpdate => "CHANNEL_UPDATE",
            Self::ChannelDelete => "CHANNEL_DELETE",
            Self::MessageCreate => "MESSAGE_CREATE",
            Self::MessageUpdate => "MESSAGE_UPDATE",
            Self::MessageDelete => "MESSAGE_DELETE",
            Self::MessageReactionAdd => "MESSAGE_REACTION_ADD",
            Self::MessageReactionRemove => "MESSAGE_REACTION_REMOVE",
            Self::GuildMemberAdd => "GUILD_MEMBER_ADD",
            Self::GuildMemberUpdate => "GUILD_MEMBER_UPDATE",
            Self::GuildMemberRemove => "GUILD_MEMBER_REMOVE",
            Self::PresenceUpdate => "PRESENCE_UPDATE",
            Self::TypingStart => "TYPING_START",
            Self::UserUpdate => "USER_UPDATE",
        }
    }

    /// Parse an event type from a string
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "READY" => Some(Self::Ready),
            "RESUMED" => Some(Self::Resumed),
            "GUILD_CREATE" => Some(Self::GuildCreate),
            "GUILD_UPDATE" => Some(Self::GuildUpdate),
            "GUILD_DELETE" => Some(Self::GuildDelete),
            "CHANNEL_CREATE" => Some(Self::ChannelCreate),
            "CHANNEL_UPDATE" => Some(Self::ChannelUpdate),
            "CHANNEL_DELETE" => Some(Self::ChannelDelete),
            "MESSAGE_CREATE" => Some(Self::MessageCreate),
            "MESSAGE_UPDATE" => Some(Self::MessageUpdate),
            "MESSAGE_DELETE" => Some(Self::MessageDelete),
            "MESSAGE_REACTION_ADD" => Some(Self::MessageReactionAdd),
            "MESSAGE_REACTION_REMOVE" => Some(Self::MessageReactionRemove),
            "GUILD_MEMBER_ADD" => Some(Self::GuildMemberAdd),
            "GUILD_MEMBER_UPDATE" => Some(Self::GuildMemberUpdate),
            "GUILD_MEMBER_REMOVE" => Some(Self::GuildMemberRemove),
            "PRESENCE_UPDATE" => Some(Self::PresenceUpdate),
            "TYPING_START" => Some(Self::TypingStart),
            "USER_UPDATE" => Some(Self::UserUpdate),
            _ => None,
        }
    }
}

impl fmt::Display for GatewayEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<GatewayEventType> for String {
    fn from(event: GatewayEventType) -> Self {
        event.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(GatewayEventType::Ready.as_str(), "READY");
        assert_eq!(GatewayEventType::MessageCreate.as_str(), "MESSAGE_CREATE");
        assert_eq!(GatewayEventType::PresenceUpdate.as_str(), "PRESENCE_UPDATE");
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(GatewayEventType::from_str("READY"), Some(GatewayEventType::Ready));
        assert_eq!(
            GatewayEventType::from_str("MESSAGE_CREATE"),
            Some(GatewayEventType::MessageCreate)
        );
        assert_eq!(GatewayEventType::from_str("INVALID"), None);
    }

    #[test]
    fn test_event_type_serialization() {
        let event = GatewayEventType::MessageCreate;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"MESSAGE_CREATE\"");

        let parsed: GatewayEventType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, GatewayEventType::MessageCreate);
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", GatewayEventType::Ready), "READY");
    }
}
