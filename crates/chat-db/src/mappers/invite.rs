//! Invite entity <-> model mapper

use chat_core::entities::Invite;
use chat_core::value_objects::Snowflake;

use crate::models::InviteModel;

/// Convert InviteModel to Invite entity
impl From<InviteModel> for Invite {
    fn from(model: InviteModel) -> Self {
        Invite {
            code: model.code,
            guild_id: Snowflake::new(model.guild_id),
            channel_id: Snowflake::new(model.channel_id),
            inviter_id: Snowflake::new(model.inviter_id),
            uses: model.uses,
            max_uses: model.max_uses,
            max_age: model.max_age,
            temporary: model.temporary,
            created_at: model.created_at,
            expires_at: model.expires_at,
        }
    }
}

/// Convert Invite entity reference to values for database insertion
pub struct InviteInsert<'a> {
    pub code: &'a str,
    pub guild_id: i64,
    pub channel_id: i64,
    pub inviter_id: i64,
    pub max_uses: Option<i32>,
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> InviteInsert<'a> {
    pub fn new(invite: &'a Invite) -> Self {
        Self {
            code: &invite.code,
            guild_id: invite.guild_id.into_inner(),
            channel_id: invite.channel_id.into_inner(),
            inviter_id: invite.inviter_id.into_inner(),
            max_uses: invite.max_uses,
            max_age: invite.max_age,
            temporary: invite.temporary,
            expires_at: invite.expires_at,
        }
    }
}
