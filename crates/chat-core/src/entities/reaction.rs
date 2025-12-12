//! Reaction entity - represents an emoji reaction on a message

use chrono::{DateTime, Utc};

use crate::value_objects::Snowflake;

/// Reaction entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    pub message_id: Snowflake,
    pub user_id: Snowflake,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl Reaction {
    /// Create a new Reaction
    pub fn new(message_id: Snowflake, user_id: Snowflake, emoji: String) -> Self {
        Self {
            message_id,
            user_id,
            emoji,
            created_at: Utc::now(),
        }
    }

    /// Check if reaction uses a specific emoji
    #[inline]
    pub fn is_emoji(&self, emoji: &str) -> bool {
        self.emoji == emoji
    }
}

/// Aggregated reaction count for display
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i32,
    pub me: bool,
}

impl ReactionCount {
    /// Create a new ReactionCount
    pub fn new(emoji: String, count: i32, me: bool) -> Self {
        Self { emoji, count, me }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_creation() {
        let reaction = Reaction::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "👍".to_string(),
        );
        assert_eq!(reaction.message_id, Snowflake::new(1));
        assert_eq!(reaction.user_id, Snowflake::new(100));
        assert_eq!(reaction.emoji, "👍");
    }

    #[test]
    fn test_is_emoji() {
        let reaction = Reaction::new(
            Snowflake::new(1),
            Snowflake::new(100),
            "👍".to_string(),
        );
        assert!(reaction.is_emoji("👍"));
        assert!(!reaction.is_emoji("👎"));
    }

    #[test]
    fn test_reaction_count() {
        let count = ReactionCount::new("👍".to_string(), 5, true);
        assert_eq!(count.emoji, "👍");
        assert_eq!(count.count, 5);
        assert!(count.me);
    }
}
