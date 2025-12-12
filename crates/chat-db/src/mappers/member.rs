//! GuildMember entity <-> model mapper

use chat_core::entities::GuildMember;
use chat_core::value_objects::Snowflake;

use crate::models::GuildMemberModel;

/// Convert GuildMemberModel to GuildMember entity
/// Note: role_ids need to be loaded separately or via MemberWithRolesModel
impl From<GuildMemberModel> for GuildMember {
    fn from(model: GuildMemberModel) -> Self {
        GuildMember {
            guild_id: Snowflake::new(model.guild_id),
            user_id: Snowflake::new(model.user_id),
            nickname: model.nickname,
            role_ids: Vec::new(), // Loaded separately
            joined_at: model.joined_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert GuildMemberModel with role IDs to GuildMember entity
pub fn member_with_roles(model: GuildMemberModel, role_ids: Vec<i64>) -> GuildMember {
    GuildMember {
        guild_id: Snowflake::new(model.guild_id),
        user_id: Snowflake::new(model.user_id),
        nickname: model.nickname,
        role_ids: role_ids.into_iter().map(Snowflake::new).collect(),
        joined_at: model.joined_at,
        updated_at: model.updated_at,
    }
}

/// Convert GuildMember entity reference to values for database insertion
pub struct MemberInsert {
    pub guild_id: i64,
    pub user_id: i64,
    pub nickname: Option<String>,
}

impl MemberInsert {
    pub fn new(member: &GuildMember) -> Self {
        Self {
            guild_id: member.guild_id.into_inner(),
            user_id: member.user_id.into_inner(),
            nickname: member.nickname.clone(),
        }
    }
}

/// Convert GuildMember entity reference to values for database update
pub struct MemberUpdate {
    pub guild_id: i64,
    pub user_id: i64,
    pub nickname: Option<String>,
}

impl MemberUpdate {
    pub fn new(member: &GuildMember) -> Self {
        Self {
            guild_id: member.guild_id.into_inner(),
            user_id: member.user_id.into_inner(),
            nickname: member.nickname.clone(),
        }
    }
}
