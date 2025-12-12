//! Guild entity - represents a Discord-like server

use chrono::{DateTime, Utc};

use crate::value_objects::Snowflake;

/// Guild (server) entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: Snowflake,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guild {
    /// Create a new Guild
    pub fn new(id: Snowflake, name: String, owner_id: Snowflake) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            icon: None,
            description: None,
            owner_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if a user is the guild owner
    #[inline]
    pub fn is_owner(&self, user_id: Snowflake) -> bool {
        self.owner_id == user_id
    }

    /// Get the guild icon URL if set
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|hash| format!("/icons/{}/{}.png", self.id, hash))
    }

    /// Update the guild name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Update the guild icon
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
        self.updated_at = Utc::now();
    }

    /// Update the guild description
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Transfer ownership to another user
    pub fn transfer_ownership(&mut self, new_owner_id: Snowflake) {
        self.owner_id = new_owner_id;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_creation() {
        let guild = Guild::new(
            Snowflake::new(1),
            "Test Guild".to_string(),
            Snowflake::new(100),
        );
        assert_eq!(guild.name, "Test Guild");
        assert!(guild.is_owner(Snowflake::new(100)));
        assert!(!guild.is_owner(Snowflake::new(200)));
    }

    #[test]
    fn test_guild_icon_url() {
        let mut guild = Guild::new(
            Snowflake::new(123),
            "Test".to_string(),
            Snowflake::new(1),
        );
        assert!(guild.icon_url().is_none());

        guild.icon = Some("abc123".to_string());
        assert_eq!(guild.icon_url(), Some("/icons/123/abc123.png".to_string()));
    }

    #[test]
    fn test_transfer_ownership() {
        let mut guild = Guild::new(
            Snowflake::new(1),
            "Test".to_string(),
            Snowflake::new(100),
        );
        assert!(guild.is_owner(Snowflake::new(100)));

        guild.transfer_ownership(Snowflake::new(200));
        assert!(!guild.is_owner(Snowflake::new(100)));
        assert!(guild.is_owner(Snowflake::new(200)));
    }
}
