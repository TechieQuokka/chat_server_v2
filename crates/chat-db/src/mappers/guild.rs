//! Guild entity <-> model mapper

use chat_core::entities::Guild;
use chat_core::value_objects::Snowflake;

use crate::models::GuildModel;

/// Convert GuildModel to Guild entity
impl From<GuildModel> for Guild {
    fn from(model: GuildModel) -> Self {
        Guild {
            id: Snowflake::new(model.id),
            name: model.name,
            icon: model.icon,
            description: model.description,
            owner_id: Snowflake::new(model.owner_id),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert Guild entity reference to values for database insertion
pub struct GuildInsert<'a> {
    pub id: i64,
    pub name: &'a str,
    pub icon: Option<&'a str>,
    pub description: Option<&'a str>,
    pub owner_id: i64,
}

impl<'a> GuildInsert<'a> {
    pub fn new(guild: &'a Guild) -> Self {
        Self {
            id: guild.id.into_inner(),
            name: &guild.name,
            icon: guild.icon.as_deref(),
            description: guild.description.as_deref(),
            owner_id: guild.owner_id.into_inner(),
        }
    }
}

/// Convert Guild entity reference to values for database update
pub struct GuildUpdate<'a> {
    pub id: i64,
    pub name: &'a str,
    pub icon: Option<&'a str>,
    pub description: Option<&'a str>,
    pub owner_id: i64,
}

impl<'a> GuildUpdate<'a> {
    pub fn new(guild: &'a Guild) -> Self {
        Self {
            id: guild.id.into_inner(),
            name: &guild.name,
            icon: guild.icon.as_deref(),
            description: guild.description.as_deref(),
            owner_id: guild.owner_id.into_inner(),
        }
    }
}
