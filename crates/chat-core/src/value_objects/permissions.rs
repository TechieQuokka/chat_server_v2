//! Permissions bitflags for Discord-like access control
//!
//! Defines 11 permissions stored as a 64-bit integer bitfield.

use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

bitflags! {
    /// Discord-like permission flags
    ///
    /// Stored as BIGINT in database, serialized as string in JSON for JavaScript safety.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Permissions: u64 {
        /// View channel and read messages
        const VIEW_CHANNEL     = 1 << 0;
        /// Send messages in text channels
        const SEND_MESSAGES    = 1 << 1;
        /// Delete other users' messages
        const MANAGE_MESSAGES  = 1 << 2;
        /// Create, edit, delete channels
        const MANAGE_CHANNELS  = 1 << 3;
        /// Create, edit, delete, assign roles
        const MANAGE_ROLES     = 1 << 4;
        /// Edit guild settings
        const MANAGE_GUILD     = 1 << 5;
        /// Kick members from guild
        const KICK_MEMBERS     = 1 << 6;
        /// Ban members from guild
        const BAN_MEMBERS      = 1 << 7;
        /// Bypass all permission checks
        const ADMINISTRATOR    = 1 << 8;
        /// Upload files and images
        const ATTACH_FILES     = 1 << 9;
        /// Add emoji reactions
        const ADD_REACTIONS    = 1 << 10;

        /// Default permissions for @everyone role
        const DEFAULT = Self::VIEW_CHANNEL.bits()
            | Self::SEND_MESSAGES.bits()
            | Self::ADD_REACTIONS.bits()
            | Self::ATTACH_FILES.bits();

        /// All permissions (for server owners)
        const ALL = u64::MAX;
    }
}

impl Permissions {
    /// Check if the permission set contains a required permission
    ///
    /// Administrators bypass all permission checks.
    #[inline]
    pub fn has(&self, permission: Permissions) -> bool {
        if self.contains(Permissions::ADMINISTRATOR) {
            return true;
        }
        self.contains(permission)
    }

    /// Check if the permission set has any of the given permissions
    #[inline]
    pub fn has_any(&self, permissions: Permissions) -> bool {
        if self.contains(Permissions::ADMINISTRATOR) {
            return true;
        }
        self.intersects(permissions)
    }

    /// Check if the permission set has all of the given permissions
    #[inline]
    pub fn has_all(&self, permissions: Permissions) -> bool {
        if self.contains(Permissions::ADMINISTRATOR) {
            return true;
        }
        self.contains(permissions)
    }

    /// Combine permissions from multiple roles
    pub fn combine<I>(roles: I) -> Self
    where
        I: IntoIterator<Item = Permissions>,
    {
        roles.into_iter().fold(Permissions::empty(), |acc, p| acc | p)
    }

    /// Get the raw bits as i64 (for database storage)
    #[inline]
    pub fn to_i64(self) -> i64 {
        self.bits() as i64
    }

    /// Create from raw i64 bits (from database)
    #[inline]
    pub fn from_i64(bits: i64) -> Self {
        Permissions::from_bits_truncate(bits as u64)
    }

    /// Parse from string representation (decimal number)
    pub fn parse(s: &str) -> Result<Self, std::num::ParseIntError> {
        s.parse::<u64>().map(Permissions::from_bits_truncate)
    }

    /// Get a list of all individual permissions that are set
    pub fn list(&self) -> Vec<&'static str> {
        let mut result = Vec::new();
        if self.contains(Self::VIEW_CHANNEL) {
            result.push("VIEW_CHANNEL");
        }
        if self.contains(Self::SEND_MESSAGES) {
            result.push("SEND_MESSAGES");
        }
        if self.contains(Self::MANAGE_MESSAGES) {
            result.push("MANAGE_MESSAGES");
        }
        if self.contains(Self::MANAGE_CHANNELS) {
            result.push("MANAGE_CHANNELS");
        }
        if self.contains(Self::MANAGE_ROLES) {
            result.push("MANAGE_ROLES");
        }
        if self.contains(Self::MANAGE_GUILD) {
            result.push("MANAGE_GUILD");
        }
        if self.contains(Self::KICK_MEMBERS) {
            result.push("KICK_MEMBERS");
        }
        if self.contains(Self::BAN_MEMBERS) {
            result.push("BAN_MEMBERS");
        }
        if self.contains(Self::ADMINISTRATOR) {
            result.push("ADMINISTRATOR");
        }
        if self.contains(Self::ATTACH_FILES) {
            result.push("ATTACH_FILES");
        }
        if self.contains(Self::ADD_REACTIONS) {
            result.push("ADD_REACTIONS");
        }
        result
    }

    /// Check if this permission set is a subset of another
    #[inline]
    pub fn is_subset_of(&self, other: Permissions) -> bool {
        (*self & other) == *self
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions::empty()
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bits())
    }
}

// Serialize as string for JSON (JavaScript BigInt safety)
impl Serialize for Permissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.bits().to_string())
    }
}

// Deserialize from string or number
impl<'de> Deserialize<'de> for Permissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct PermissionsVisitor;

        impl Visitor<'_> for PermissionsVisitor {
            type Value = Permissions;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or integer representing permission bits")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Permissions, E>
            where
                E: de::Error,
            {
                Ok(Permissions::from_bits_truncate(value as u64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Permissions, E>
            where
                E: de::Error,
            {
                Ok(Permissions::from_bits_truncate(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Permissions, E>
            where
                E: de::Error,
            {
                value
                    .parse::<u64>()
                    .map(Permissions::from_bits_truncate)
                    .map_err(|_| de::Error::custom("invalid permissions string"))
            }
        }

        deserializer.deserialize_any(PermissionsVisitor)
    }
}

impl From<i64> for Permissions {
    fn from(bits: i64) -> Self {
        Permissions::from_bits_truncate(bits as u64)
    }
}

impl From<u64> for Permissions {
    fn from(bits: u64) -> Self {
        Permissions::from_bits_truncate(bits)
    }
}

impl From<Permissions> for i64 {
    fn from(perms: Permissions) -> Self {
        perms.bits() as i64
    }
}

impl From<Permissions> for u64 {
    fn from(perms: Permissions) -> Self {
        perms.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_permissions() {
        let default = Permissions::DEFAULT;
        assert!(default.contains(Permissions::VIEW_CHANNEL));
        assert!(default.contains(Permissions::SEND_MESSAGES));
        assert!(default.contains(Permissions::ADD_REACTIONS));
        assert!(default.contains(Permissions::ATTACH_FILES));
        assert!(!default.contains(Permissions::ADMINISTRATOR));
        assert!(!default.contains(Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_administrator_bypass() {
        let admin = Permissions::ADMINISTRATOR;
        // Administrator should pass all permission checks
        assert!(admin.has(Permissions::VIEW_CHANNEL));
        assert!(admin.has(Permissions::MANAGE_GUILD));
        assert!(admin.has(Permissions::BAN_MEMBERS));
        assert!(admin.has(Permissions::MANAGE_ROLES));
    }

    #[test]
    fn test_has_permission() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(perms.has(Permissions::VIEW_CHANNEL));
        assert!(perms.has(Permissions::SEND_MESSAGES));
        assert!(!perms.has(Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_has_any() {
        let perms = Permissions::VIEW_CHANNEL;
        let check = Permissions::VIEW_CHANNEL | Permissions::MANAGE_GUILD;
        assert!(perms.has_any(check));

        let perms2 = Permissions::SEND_MESSAGES;
        assert!(!perms2.has_any(check));
    }

    #[test]
    fn test_has_all() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(perms.has_all(Permissions::VIEW_CHANNEL));
        assert!(perms.has_all(Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES));
        assert!(!perms.has_all(Permissions::VIEW_CHANNEL | Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_combine_permissions() {
        let role1 = Permissions::VIEW_CHANNEL;
        let role2 = Permissions::SEND_MESSAGES;
        let role3 = Permissions::MANAGE_GUILD;

        let combined = Permissions::combine([role1, role2, role3]);
        assert!(combined.contains(Permissions::VIEW_CHANNEL));
        assert!(combined.contains(Permissions::SEND_MESSAGES));
        assert!(combined.contains(Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_serialize_json() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        let json = serde_json::to_string(&perms).unwrap();
        assert_eq!(json, "\"3\""); // 1 + 2 = 3
    }

    #[test]
    fn test_deserialize_string() {
        let perms: Permissions = serde_json::from_str("\"3\"").unwrap();
        assert!(perms.contains(Permissions::VIEW_CHANNEL));
        assert!(perms.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn test_deserialize_number() {
        let perms: Permissions = serde_json::from_str("3").unwrap();
        assert!(perms.contains(Permissions::VIEW_CHANNEL));
        assert!(perms.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn test_to_from_i64() {
        let perms = Permissions::DEFAULT;
        let bits = perms.to_i64();
        let restored = Permissions::from_i64(bits);
        assert_eq!(perms, restored);
    }

    #[test]
    fn test_list_permissions() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::ADMINISTRATOR;
        let list = perms.list();
        assert!(list.contains(&"VIEW_CHANNEL"));
        assert!(list.contains(&"ADMINISTRATOR"));
        assert!(!list.contains(&"MANAGE_GUILD"));
    }

    #[test]
    fn test_parse() {
        let perms = Permissions::parse("7").unwrap(); // 1 + 2 + 4
        assert!(perms.contains(Permissions::VIEW_CHANNEL));
        assert!(perms.contains(Permissions::SEND_MESSAGES));
        assert!(perms.contains(Permissions::MANAGE_MESSAGES));
    }

    #[test]
    fn test_display() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert_eq!(perms.to_string(), "3");
    }

    #[test]
    fn test_is_subset_of() {
        let subset = Permissions::VIEW_CHANNEL;
        let superset = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(subset.is_subset_of(superset));
        assert!(!superset.is_subset_of(subset));
    }

    #[test]
    fn test_all_11_permissions_defined() {
        // Verify all 11 permissions are defined
        assert_eq!(Permissions::VIEW_CHANNEL.bits(), 1 << 0);
        assert_eq!(Permissions::SEND_MESSAGES.bits(), 1 << 1);
        assert_eq!(Permissions::MANAGE_MESSAGES.bits(), 1 << 2);
        assert_eq!(Permissions::MANAGE_CHANNELS.bits(), 1 << 3);
        assert_eq!(Permissions::MANAGE_ROLES.bits(), 1 << 4);
        assert_eq!(Permissions::MANAGE_GUILD.bits(), 1 << 5);
        assert_eq!(Permissions::KICK_MEMBERS.bits(), 1 << 6);
        assert_eq!(Permissions::BAN_MEMBERS.bits(), 1 << 7);
        assert_eq!(Permissions::ADMINISTRATOR.bits(), 1 << 8);
        assert_eq!(Permissions::ATTACH_FILES.bits(), 1 << 9);
        assert_eq!(Permissions::ADD_REACTIONS.bits(), 1 << 10);
    }
}
