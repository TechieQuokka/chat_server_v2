//! Invite entity - represents an invite link to a guild

use chrono::{DateTime, Duration, Utc};

use crate::value_objects::Snowflake;

/// Invite entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
    pub code: String,
    pub guild_id: Snowflake,
    pub channel_id: Snowflake,
    pub inviter_id: Snowflake,
    pub uses: i32,
    pub max_uses: Option<i32>,
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Invite {
    /// Create a new Invite
    pub fn new(
        code: String,
        guild_id: Snowflake,
        channel_id: Snowflake,
        inviter_id: Snowflake,
    ) -> Self {
        Self {
            code,
            guild_id,
            channel_id,
            inviter_id,
            uses: 0,
            max_uses: None,
            max_age: None,
            temporary: false,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Create an invite with expiration
    pub fn with_expiration(mut self, max_age_seconds: i32) -> Self {
        self.max_age = Some(max_age_seconds);
        if max_age_seconds > 0 {
            self.expires_at = Some(self.created_at + Duration::seconds(i64::from(max_age_seconds)));
        }
        self
    }

    /// Create an invite with max uses
    pub fn with_max_uses(mut self, max_uses: i32) -> Self {
        if max_uses > 0 {
            self.max_uses = Some(max_uses);
        }
        self
    }

    /// Create a temporary invite
    pub fn with_temporary(mut self, temporary: bool) -> Self {
        self.temporary = temporary;
        self
    }

    /// Check if invite is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if invite has reached max uses
    pub fn is_exhausted(&self) -> bool {
        if let Some(max_uses) = self.max_uses {
            self.uses >= max_uses
        } else {
            false
        }
    }

    /// Check if invite is still valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_exhausted()
    }

    /// Increment use count
    pub fn increment_uses(&mut self) {
        self.uses += 1;
    }

    /// Get remaining uses (None if unlimited)
    pub fn remaining_uses(&self) -> Option<i32> {
        self.max_uses.map(|max| max - self.uses)
    }

    /// Get the full invite URL
    pub fn url(&self) -> String {
        format!("https://discord.gg/{}", self.code)
    }
}

/// Generate a cryptographically secure random invite code
pub fn generate_invite_code() -> String {
    use rand::Rng;

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const CODE_LEN: usize = 8;

    let mut rng = rand::thread_rng();
    (0..CODE_LEN)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_creation() {
        let invite = Invite::new(
            "abc123".to_string(),
            Snowflake::new(100),
            Snowflake::new(200),
            Snowflake::new(300),
        );
        assert_eq!(invite.code, "abc123");
        assert!(invite.is_valid());
        assert!(!invite.is_expired());
        assert!(!invite.is_exhausted());
    }

    #[test]
    fn test_invite_with_max_uses() {
        let mut invite = Invite::new(
            "abc123".to_string(),
            Snowflake::new(100),
            Snowflake::new(200),
            Snowflake::new(300),
        )
        .with_max_uses(3);

        assert_eq!(invite.remaining_uses(), Some(3));
        assert!(invite.is_valid());

        invite.increment_uses();
        invite.increment_uses();
        assert_eq!(invite.remaining_uses(), Some(1));
        assert!(invite.is_valid());

        invite.increment_uses();
        assert_eq!(invite.remaining_uses(), Some(0));
        assert!(invite.is_exhausted());
        assert!(!invite.is_valid());
    }

    #[test]
    fn test_invite_unlimited_uses() {
        let invite = Invite::new(
            "abc123".to_string(),
            Snowflake::new(100),
            Snowflake::new(200),
            Snowflake::new(300),
        );
        assert_eq!(invite.remaining_uses(), None);
        assert!(!invite.is_exhausted());
    }

    #[test]
    fn test_invite_url() {
        let invite = Invite::new(
            "testcode".to_string(),
            Snowflake::new(1),
            Snowflake::new(2),
            Snowflake::new(3),
        );
        assert_eq!(invite.url(), "https://discord.gg/testcode");
    }

    #[test]
    fn test_generate_invite_code() {
        let code1 = generate_invite_code();
        let code2 = generate_invite_code();

        assert_eq!(code1.len(), 8);
        assert_eq!(code2.len(), 8);
        // Codes should be alphanumeric
        assert!(code1.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
