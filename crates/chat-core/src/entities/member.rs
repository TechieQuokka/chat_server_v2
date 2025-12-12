//! Member entity - represents a user's membership in a guild

use chrono::{DateTime, Utc};

use crate::value_objects::Snowflake;

/// Guild member entity (junction between User and Guild)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildMember {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub nickname: Option<String>,
    pub role_ids: Vec<Snowflake>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GuildMember {
    /// Create a new GuildMember
    pub fn new(guild_id: Snowflake, user_id: Snowflake) -> Self {
        let now = Utc::now();
        Self {
            guild_id,
            user_id,
            nickname: None,
            role_ids: Vec::new(),
            joined_at: now,
            updated_at: now,
        }
    }

    /// Get display name (nickname if set, otherwise fallback)
    pub fn display_name<'a>(&'a self, username: &'a str) -> &'a str {
        self.nickname.as_deref().unwrap_or(username)
    }

    /// Check if member has a specific role
    #[inline]
    pub fn has_role(&self, role_id: Snowflake) -> bool {
        self.role_ids.contains(&role_id)
    }

    /// Add a role to the member
    pub fn add_role(&mut self, role_id: Snowflake) {
        if !self.has_role(role_id) {
            self.role_ids.push(role_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a role from the member
    pub fn remove_role(&mut self, role_id: Snowflake) {
        if let Some(pos) = self.role_ids.iter().position(|&id| id == role_id) {
            self.role_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Set the member's roles (replaces all existing roles)
    pub fn set_roles(&mut self, role_ids: Vec<Snowflake>) {
        self.role_ids = role_ids;
        self.updated_at = Utc::now();
    }

    /// Update the member's nickname
    pub fn set_nickname(&mut self, nickname: Option<String>) {
        self.nickname = nickname;
        self.updated_at = Utc::now();
    }

    /// Get number of roles
    #[inline]
    pub fn role_count(&self) -> usize {
        self.role_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_creation() {
        let member = GuildMember::new(Snowflake::new(100), Snowflake::new(200));
        assert_eq!(member.guild_id, Snowflake::new(100));
        assert_eq!(member.user_id, Snowflake::new(200));
        assert!(member.nickname.is_none());
        assert!(member.role_ids.is_empty());
    }

    #[test]
    fn test_display_name() {
        let mut member = GuildMember::new(Snowflake::new(1), Snowflake::new(2));
        assert_eq!(member.display_name("TestUser"), "TestUser");

        member.nickname = Some("Nickname".to_string());
        assert_eq!(member.display_name("TestUser"), "Nickname");
    }

    #[test]
    fn test_role_management() {
        let mut member = GuildMember::new(Snowflake::new(1), Snowflake::new(2));
        let role1 = Snowflake::new(100);
        let role2 = Snowflake::new(101);

        assert!(!member.has_role(role1));

        member.add_role(role1);
        assert!(member.has_role(role1));
        assert_eq!(member.role_count(), 1);

        // Adding same role again should not duplicate
        member.add_role(role1);
        assert_eq!(member.role_count(), 1);

        member.add_role(role2);
        assert_eq!(member.role_count(), 2);

        member.remove_role(role1);
        assert!(!member.has_role(role1));
        assert!(member.has_role(role2));
        assert_eq!(member.role_count(), 1);
    }

    #[test]
    fn test_set_roles() {
        let mut member = GuildMember::new(Snowflake::new(1), Snowflake::new(2));
        member.add_role(Snowflake::new(100));
        member.add_role(Snowflake::new(101));

        let new_roles = vec![Snowflake::new(200), Snowflake::new(201), Snowflake::new(202)];
        member.set_roles(new_roles);

        assert!(!member.has_role(Snowflake::new(100)));
        assert!(member.has_role(Snowflake::new(200)));
        assert_eq!(member.role_count(), 3);
    }
}
