//! Role entity - represents a guild role with permissions

use chrono::{DateTime, Utc};

use crate::value_objects::{Permissions, Snowflake};

/// Role entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: Permissions,
    pub mentionable: bool,
    pub is_everyone: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    /// Create a new Role
    pub fn new(
        id: Snowflake,
        guild_id: Snowflake,
        name: String,
        permissions: Permissions,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            guild_id,
            name,
            color: 0,
            hoist: false,
            position: 0,
            permissions,
            mentionable: false,
            is_everyone: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create the @everyone role for a guild
    pub fn everyone(id: Snowflake, guild_id: Snowflake) -> Self {
        let now = Utc::now();
        Self {
            id,
            guild_id,
            name: "@everyone".to_string(),
            color: 0,
            hoist: false,
            position: 0,
            permissions: Permissions::DEFAULT,
            mentionable: false,
            is_everyone: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this role grants a specific permission
    #[inline]
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.has(permission)
    }

    /// Compare role positions for hierarchy (higher position = more authority)
    #[inline]
    pub fn is_higher_than(&self, other: &Role) -> bool {
        self.position > other.position
    }

    /// Check if this role can manage another role
    pub fn can_manage(&self, other: &Role) -> bool {
        // Can't manage roles at same or higher position
        // @everyone can never be managed
        !other.is_everyone && self.is_higher_than(other)
    }

    /// Get the color as a hex string (without #)
    pub fn color_hex(&self) -> String {
        format!("{:06x}", self.color)
    }

    /// Update role name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Update role color
    pub fn set_color(&mut self, color: i32) {
        self.color = color;
        self.updated_at = Utc::now();
    }

    /// Update role permissions
    pub fn set_permissions(&mut self, permissions: Permissions) {
        self.permissions = permissions;
        self.updated_at = Utc::now();
    }

    /// Update role position
    pub fn set_position(&mut self, position: i32) {
        self.position = position;
        self.updated_at = Utc::now();
    }

    /// Update hoist setting (display role members separately)
    pub fn set_hoist(&mut self, hoist: bool) {
        self.hoist = hoist;
        self.updated_at = Utc::now();
    }

    /// Update mentionable setting
    pub fn set_mentionable(&mut self, mentionable: bool) {
        self.mentionable = mentionable;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let role = Role::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "Moderator".to_string(),
            Permissions::KICK_MEMBERS | Permissions::BAN_MEMBERS,
        );
        assert_eq!(role.name, "Moderator");
        assert!(role.has_permission(Permissions::KICK_MEMBERS));
        assert!(role.has_permission(Permissions::BAN_MEMBERS));
        assert!(!role.is_everyone);
    }

    #[test]
    fn test_everyone_role() {
        let role = Role::everyone(Snowflake::new(1), Snowflake::new(100));
        assert_eq!(role.name, "@everyone");
        assert!(role.is_everyone);
        assert!(role.has_permission(Permissions::VIEW_CHANNEL));
        assert!(role.has_permission(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn test_role_hierarchy() {
        let mut admin = Role::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "Admin".to_string(),
            Permissions::ADMINISTRATOR,
        );
        admin.position = 10;

        let mut mod_role = Role::new(
            Snowflake::new(2),
            Snowflake::new(100),
            "Mod".to_string(),
            Permissions::KICK_MEMBERS,
        );
        mod_role.position = 5;

        assert!(admin.is_higher_than(&mod_role));
        assert!(!mod_role.is_higher_than(&admin));
        assert!(admin.can_manage(&mod_role));
        assert!(!mod_role.can_manage(&admin));
    }

    #[test]
    fn test_cannot_manage_everyone() {
        let mut admin = Role::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "Admin".to_string(),
            Permissions::ADMINISTRATOR,
        );
        admin.position = 100;

        let everyone = Role::everyone(Snowflake::new(2), Snowflake::new(100));
        assert!(!admin.can_manage(&everyone));
    }

    #[test]
    fn test_color_hex() {
        let mut role = Role::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "Red".to_string(),
            Permissions::empty(),
        );
        role.color = 0xFF0000;
        assert_eq!(role.color_hex(), "ff0000");

        role.color = 0;
        assert_eq!(role.color_hex(), "000000");
    }
}
