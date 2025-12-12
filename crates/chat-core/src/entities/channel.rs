//! Channel entity - represents a text channel, DM, or category

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_objects::Snowflake;

/// Channel type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum ChannelType {
    /// Guild text channel
    #[default]
    GuildText = 0,
    /// Direct message between users
    Dm = 1,
    /// Guild category for organizing channels
    GuildCategory = 4,
}

impl ChannelType {
    /// Get the numeric value
    #[inline]
    #[must_use]
    pub fn as_i16(self) -> i16 {
        self as i16
    }
}

impl From<i16> for ChannelType {
    fn from(value: i16) -> Self {
        match value {
            1 => Self::Dm,
            4 => Self::GuildCategory,
            _ => Self::GuildText, // Default for 0 and unknown values
        }
    }
}

impl From<ChannelType> for i16 {
    fn from(ct: ChannelType) -> Self {
        ct as i16
    }
}

/// Channel entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel {
    pub id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub name: Option<String>,
    pub channel_type: ChannelType,
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<Snowflake>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Channel {
    /// Create a new guild text channel
    #[must_use]
    pub fn new_text(id: Snowflake, guild_id: Snowflake, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            guild_id: Some(guild_id),
            name: Some(name),
            channel_type: ChannelType::GuildText,
            topic: None,
            position: 0,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new DM channel
    #[must_use]
    pub fn new_dm(id: Snowflake) -> Self {
        let now = Utc::now();
        Self {
            id,
            guild_id: None,
            name: None,
            channel_type: ChannelType::Dm,
            topic: None,
            position: 0,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new category channel
    #[must_use]
    pub fn new_category(id: Snowflake, guild_id: Snowflake, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            guild_id: Some(guild_id),
            name: Some(name),
            channel_type: ChannelType::GuildCategory,
            topic: None,
            position: 0,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this is a text channel (guild text or DM)
    #[inline]
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self.channel_type, ChannelType::GuildText | ChannelType::Dm)
    }

    /// Check if this is a category
    #[inline]
    #[must_use]
    pub fn is_category(&self) -> bool {
        matches!(self.channel_type, ChannelType::GuildCategory)
    }

    /// Check if this is a DM channel
    #[inline]
    #[must_use]
    pub fn is_dm(&self) -> bool {
        matches!(self.channel_type, ChannelType::Dm)
    }

    /// Check if this is a guild channel
    #[inline]
    #[must_use]
    pub fn is_guild_channel(&self) -> bool {
        self.guild_id.is_some()
    }

    /// Get display name (channel name or fallback for DMs)
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Direct Message")
    }

    /// Update channel name
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
        self.updated_at = Utc::now();
    }

    /// Update channel topic
    pub fn set_topic(&mut self, topic: Option<String>) {
        self.topic = topic;
        self.updated_at = Utc::now();
    }

    /// Update channel position
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    /// Move channel to a category
    pub fn set_parent(&mut self, parent_id: Option<Snowflake>) {
        self.parent_id = parent_id;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_from_i16() {
        assert_eq!(ChannelType::from(0), ChannelType::GuildText);
        assert_eq!(ChannelType::from(1), ChannelType::Dm);
        assert_eq!(ChannelType::from(4), ChannelType::GuildCategory);
        assert_eq!(ChannelType::from(99), ChannelType::GuildText); // Unknown defaults to text
    }

    #[test]
    fn test_text_channel() {
        let channel = Channel::new_text(
            Snowflake::new(1),
            Snowflake::new(100),
            "general".to_string(),
        );
        assert!(channel.is_text());
        assert!(!channel.is_category());
        assert!(!channel.is_dm());
        assert!(channel.is_guild_channel());
        assert_eq!(channel.display_name(), "general");
    }

    #[test]
    fn test_dm_channel() {
        let channel = Channel::new_dm(Snowflake::new(1));
        assert!(channel.is_text());
        assert!(channel.is_dm());
        assert!(!channel.is_guild_channel());
        assert_eq!(channel.display_name(), "Direct Message");
    }

    #[test]
    fn test_category_channel() {
        let channel = Channel::new_category(
            Snowflake::new(1),
            Snowflake::new(100),
            "Text Channels".to_string(),
        );
        assert!(channel.is_category());
        assert!(!channel.is_text());
        assert!(channel.is_guild_channel());
    }
}
