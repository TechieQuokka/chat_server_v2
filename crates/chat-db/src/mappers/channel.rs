//! Channel entity <-> model mapper

use chat_core::entities::{Channel, ChannelType};
use chat_core::value_objects::Snowflake;

use crate::models::ChannelModel;

/// Convert database channel type string to ChannelType enum
fn parse_channel_type(type_str: &str) -> ChannelType {
    match type_str {
        "text" => ChannelType::GuildText,
        "dm" => ChannelType::Dm,
        "category" => ChannelType::GuildCategory,
        _ => ChannelType::GuildText,
    }
}

/// Convert ChannelType enum to database string
pub fn channel_type_to_str(ct: ChannelType) -> &'static str {
    match ct {
        ChannelType::GuildText => "text",
        ChannelType::Dm => "dm",
        ChannelType::GuildCategory => "category",
    }
}

/// Convert ChannelModel to Channel entity
impl From<ChannelModel> for Channel {
    fn from(model: ChannelModel) -> Self {
        Channel {
            id: Snowflake::new(model.id),
            guild_id: model.guild_id.map(Snowflake::new),
            name: model.name,
            channel_type: parse_channel_type(&model.channel_type),
            topic: model.topic,
            position: model.position,
            parent_id: model.parent_id.map(Snowflake::new),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert Channel entity reference to values for database insertion
pub struct ChannelInsert<'a> {
    pub id: i64,
    pub guild_id: Option<i64>,
    pub name: Option<&'a str>,
    pub channel_type: &'static str,
    pub topic: Option<&'a str>,
    pub position: i32,
    pub parent_id: Option<i64>,
}

impl<'a> ChannelInsert<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        Self {
            id: channel.id.into_inner(),
            guild_id: channel.guild_id.map(chat_core::Snowflake::into_inner),
            name: channel.name.as_deref(),
            channel_type: channel_type_to_str(channel.channel_type),
            topic: channel.topic.as_deref(),
            position: channel.position,
            parent_id: channel.parent_id.map(chat_core::Snowflake::into_inner),
        }
    }
}

/// Convert Channel entity reference to values for database update
pub struct ChannelUpdate<'a> {
    pub id: i64,
    pub name: Option<&'a str>,
    pub topic: Option<&'a str>,
    pub position: i32,
    pub parent_id: Option<i64>,
}

impl<'a> ChannelUpdate<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        Self {
            id: channel.id.into_inner(),
            name: channel.name.as_deref(),
            topic: channel.topic.as_deref(),
            position: channel.position,
            parent_id: channel.parent_id.map(chat_core::Snowflake::into_inner),
        }
    }
}
