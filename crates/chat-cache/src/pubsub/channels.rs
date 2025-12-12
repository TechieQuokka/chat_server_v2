//! Pub/Sub channel definitions.
//!
//! Defines the channel naming conventions for Redis Pub/Sub.

use chat_core::Snowflake;

/// Channel prefix for guild events
pub const GUILD_CHANNEL_PREFIX: &str = "guild:";
/// Channel prefix for channel-specific events
pub const CHANNEL_PREFIX: &str = "channel:";
/// Channel prefix for user-specific events
pub const USER_CHANNEL_PREFIX: &str = "user:";
/// Channel for broadcast events (all connected clients)
pub const BROADCAST_CHANNEL: &str = "broadcast";

/// Pub/Sub channel types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PubSubChannel {
    /// Events for a specific guild (all members)
    Guild(Snowflake),
    /// Events for a specific channel
    Channel(Snowflake),
    /// Events for a specific user (all their sessions)
    User(Snowflake),
    /// Broadcast to all connected clients
    Broadcast,
    /// Custom channel name
    Custom(String),
}

impl PubSubChannel {
    /// Create a guild channel
    #[must_use]
    pub fn guild(guild_id: Snowflake) -> Self {
        Self::Guild(guild_id)
    }

    /// Create a channel channel
    #[must_use]
    pub fn channel(channel_id: Snowflake) -> Self {
        Self::Channel(channel_id)
    }

    /// Create a user channel
    #[must_use]
    pub fn user(user_id: Snowflake) -> Self {
        Self::User(user_id)
    }

    /// Create a broadcast channel
    #[must_use]
    pub fn broadcast() -> Self {
        Self::Broadcast
    }

    /// Create a custom channel
    #[must_use]
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Get the Redis channel name
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Guild(id) => format!("{GUILD_CHANNEL_PREFIX}{id}"),
            Self::Channel(id) => format!("{CHANNEL_PREFIX}{id}"),
            Self::User(id) => format!("{USER_CHANNEL_PREFIX}{id}"),
            Self::Broadcast => BROADCAST_CHANNEL.to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Parse a channel name back to a `PubSubChannel`
    #[must_use]
    pub fn parse(name: &str) -> Self {
        if name == BROADCAST_CHANNEL {
            return Self::Broadcast;
        }

        if let Some(id_str) = name.strip_prefix(GUILD_CHANNEL_PREFIX) {
            if let Ok(id) = id_str.parse::<i64>() {
                return Self::Guild(Snowflake::from(id));
            }
        }

        if let Some(id_str) = name.strip_prefix(CHANNEL_PREFIX) {
            if let Ok(id) = id_str.parse::<i64>() {
                return Self::Channel(Snowflake::from(id));
            }
        }

        if let Some(id_str) = name.strip_prefix(USER_CHANNEL_PREFIX) {
            if let Ok(id) = id_str.parse::<i64>() {
                return Self::User(Snowflake::from(id));
            }
        }

        Self::Custom(name.to_string())
    }
}

impl std::fmt::Display for PubSubChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_names() {
        let guild_id = Snowflake::from(12345i64);
        let channel_id = Snowflake::from(67890i64);
        let user_id = Snowflake::from(11111i64);

        assert_eq!(PubSubChannel::guild(guild_id).name(), "guild:12345");
        assert_eq!(PubSubChannel::channel(channel_id).name(), "channel:67890");
        assert_eq!(PubSubChannel::user(user_id).name(), "user:11111");
        assert_eq!(PubSubChannel::broadcast().name(), "broadcast");
        assert_eq!(PubSubChannel::custom("test").name(), "test");
    }

    #[test]
    fn test_channel_parse() {
        let guild_channel = PubSubChannel::parse("guild:12345");
        assert_eq!(guild_channel, PubSubChannel::Guild(Snowflake::from(12345i64)));

        let channel_channel = PubSubChannel::parse("channel:67890");
        assert_eq!(channel_channel, PubSubChannel::Channel(Snowflake::from(67890i64)));

        let user_channel = PubSubChannel::parse("user:11111");
        assert_eq!(user_channel, PubSubChannel::User(Snowflake::from(11111i64)));

        let broadcast = PubSubChannel::parse("broadcast");
        assert_eq!(broadcast, PubSubChannel::Broadcast);

        let custom = PubSubChannel::parse("unknown:123");
        assert_eq!(custom, PubSubChannel::Custom("unknown:123".to_string()));
    }
}
