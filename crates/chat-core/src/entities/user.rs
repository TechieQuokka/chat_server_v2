//! User entity - represents a chat user

use chrono::{DateTime, Utc};

use crate::value_objects::Snowflake;

/// User entity representing a Discord-like user account
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new User with required fields
    pub fn new(
        id: Snowflake,
        username: String,
        discriminator: String,
        email: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            discriminator,
            email,
            avatar: None,
            bot: false,
            system: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the full tag: username#discriminator
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username, self.discriminator)
    }

    /// Get avatar URL or default avatar URL
    pub fn avatar_url(&self) -> String {
        match &self.avatar {
            Some(hash) => format!("/avatars/{}/{}.png", self.id, hash),
            None => format!("/embed/avatars/{}.png", self.default_avatar_index()),
        }
    }

    /// Get default avatar index (0-4) based on discriminator
    fn default_avatar_index(&self) -> u8 {
        self.discriminator.parse::<u16>().unwrap_or(0) as u8 % 5
    }

    /// Check if user is a bot account
    #[inline]
    pub fn is_bot(&self) -> bool {
        self.bot
    }

    /// Check if user is a system account
    #[inline]
    pub fn is_system(&self) -> bool {
        self.system
    }

    /// Update the username
    pub fn set_username(&mut self, username: String) {
        self.username = username;
        self.updated_at = Utc::now();
    }

    /// Update the avatar
    pub fn set_avatar(&mut self, avatar: Option<String>) {
        self.avatar = avatar;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_tag() {
        let user = User::new(
            Snowflake::new(1),
            "testuser".to_string(),
            "1234".to_string(),
            "test@example.com".to_string(),
        );
        assert_eq!(user.tag(), "testuser#1234");
    }

    #[test]
    fn test_avatar_url_with_avatar() {
        let mut user = User::new(
            Snowflake::new(123),
            "testuser".to_string(),
            "1234".to_string(),
            "test@example.com".to_string(),
        );
        user.avatar = Some("abc123".to_string());
        assert_eq!(user.avatar_url(), "/avatars/123/abc123.png");
    }

    #[test]
    fn test_avatar_url_default() {
        let user = User::new(
            Snowflake::new(123),
            "testuser".to_string(),
            "0000".to_string(),
            "test@example.com".to_string(),
        );
        // 0 % 5 = 0
        assert_eq!(user.avatar_url(), "/embed/avatars/0.png");
    }

    #[test]
    fn test_default_avatar_indices() {
        for i in 0..5 {
            let user = User::new(
                Snowflake::new(1),
                "test".to_string(),
                i.to_string(),
                "test@example.com".to_string(),
            );
            assert_eq!(user.default_avatar_index(), i as u8);
        }
    }
}
