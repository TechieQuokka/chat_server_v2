//! Role entity <-> model mapper

use chat_core::entities::Role;
use chat_core::value_objects::{Permissions, Snowflake};

use crate::models::RoleModel;

/// Convert RoleModel to Role entity
impl From<RoleModel> for Role {
    fn from(model: RoleModel) -> Self {
        Role {
            id: Snowflake::new(model.id),
            guild_id: Snowflake::new(model.guild_id),
            name: model.name,
            color: model.color,
            hoist: model.hoist,
            position: model.position,
            permissions: Permissions::from_i64(model.permissions),
            mentionable: model.mentionable,
            is_everyone: model.is_everyone,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert Role entity reference to values for database insertion
pub struct RoleInsert<'a> {
    pub id: i64,
    pub guild_id: i64,
    pub name: &'a str,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: i64,
    pub mentionable: bool,
    pub is_everyone: bool,
}

impl<'a> RoleInsert<'a> {
    pub fn new(role: &'a Role) -> Self {
        Self {
            id: role.id.into_inner(),
            guild_id: role.guild_id.into_inner(),
            name: &role.name,
            color: role.color,
            hoist: role.hoist,
            position: role.position,
            permissions: role.permissions.to_i64(),
            mentionable: role.mentionable,
            is_everyone: role.is_everyone,
        }
    }
}

/// Convert Role entity reference to values for database update
pub struct RoleUpdate<'a> {
    pub id: i64,
    pub name: &'a str,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: i64,
    pub mentionable: bool,
}

impl<'a> RoleUpdate<'a> {
    pub fn new(role: &'a Role) -> Self {
        Self {
            id: role.id.into_inner(),
            name: &role.name,
            color: role.color,
            hoist: role.hoist,
            position: role.position,
            permissions: role.permissions.to_i64(),
            mentionable: role.mentionable,
        }
    }
}
