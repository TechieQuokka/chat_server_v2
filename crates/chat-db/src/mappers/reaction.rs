//! Reaction entity <-> model mapper

use chat_core::entities::Reaction;
use chat_core::value_objects::Snowflake;

use crate::models::ReactionModel;

/// Convert ReactionModel to Reaction entity
impl From<ReactionModel> for Reaction {
    fn from(model: ReactionModel) -> Self {
        Reaction {
            message_id: Snowflake::new(model.message_id),
            user_id: Snowflake::new(model.user_id),
            emoji: model.emoji,
            created_at: model.created_at,
        }
    }
}

/// Convert Reaction entity reference to values for database insertion
pub struct ReactionInsert<'a> {
    pub message_id: i64,
    pub user_id: i64,
    pub emoji: &'a str,
}

impl<'a> ReactionInsert<'a> {
    pub fn new(reaction: &'a Reaction) -> Self {
        Self {
            message_id: reaction.message_id.into_inner(),
            user_id: reaction.user_id.into_inner(),
            emoji: &reaction.emoji,
        }
    }
}
