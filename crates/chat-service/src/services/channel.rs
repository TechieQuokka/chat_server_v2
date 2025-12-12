//! Channel service
//!
//! Handles channel creation, management, and queries.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::{Channel, ChannelType};
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{ChannelResponse, CreateChannelRequest, UpdateChannelRequest};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Channel service
pub struct ChannelService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> ChannelService<'a> {
    /// Create a new ChannelService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create a new channel in a guild
    #[instrument(skip(self, request))]
    pub async fn create_channel(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        request: CreateChannelRequest,
    ) -> ServiceResult<ChannelResponse> {
        // Check permissions
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_CHANNELS)
            .await?;

        // Verify guild exists
        let _guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Parse channel type
        let channel_type = match request.channel_type {
            0 => ChannelType::GuildText,
            4 => ChannelType::GuildCategory,
            _ => return Err(ServiceError::validation("Invalid channel type")),
        };

        // Parse parent_id
        let parent_id = if let Some(ref parent_str) = request.parent_id {
            let parent = parent_str
                .parse::<i64>()
                .map(Snowflake::new)
                .map_err(|_| ServiceError::validation("Invalid parent_id format"))?;

            // Verify parent is a category
            if let Some(parent_channel) = self.ctx.channel_repo().find_by_id(parent).await? {
                if parent_channel.channel_type != ChannelType::GuildCategory {
                    return Err(ServiceError::validation("Parent must be a category channel"));
                }
            } else {
                return Err(ServiceError::not_found("Channel", parent_str.clone()));
            }

            Some(parent)
        } else {
            None
        };

        // Get position (default to end of list)
        let position = request.position.unwrap_or({
            // Would need to query existing channels to get max position
            0
        });

        let channel_id = self.ctx.generate_id();
        let now = Utc::now();

        let channel = Channel {
            id: channel_id,
            guild_id: Some(guild_id),
            name: Some(request.name),
            channel_type,
            topic: request.topic,
            position,
            parent_id,
            created_at: now,
            updated_at: now,
        };

        self.ctx.channel_repo().create(&channel).await?;

        info!(channel_id = %channel_id, guild_id = %guild_id, "Channel created");

        // Publish CHANNEL_CREATE event
        self.publish_channel_event("CHANNEL_CREATE", &channel).await;

        Ok(ChannelResponse::from(&channel))
    }

    /// Get channel by ID
    #[instrument(skip(self))]
    pub async fn get_channel(&self, channel_id: Snowflake) -> ServiceResult<ChannelResponse> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        Ok(ChannelResponse::from(&channel))
    }

    /// Get channel entity by ID
    #[instrument(skip(self))]
    pub async fn get_channel_entity(&self, channel_id: Snowflake) -> ServiceResult<Channel> {
        self.ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))
    }

    /// Get channel with permission check
    #[instrument(skip(self))]
    pub async fn get_channel_with_permission(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Channel> {
        let channel = self.get_channel_entity(channel_id).await?;

        // Check permissions for guild channels
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, user_id, Permissions::VIEW_CHANNEL)
                .await?;
        } else {
            // DM channel - verify user is a recipient
            let recipients = self.ctx.channel_repo().get_dm_recipients(channel_id).await?;
            if !recipients.contains(&user_id) {
                return Err(ServiceError::not_found("Channel", channel_id.to_string()));
            }
        }

        Ok(channel)
    }

    /// Update channel
    #[instrument(skip(self, request))]
    pub async fn update_channel(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        request: UpdateChannelRequest,
    ) -> ServiceResult<ChannelResponse> {
        let mut channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // DM channels cannot be updated
        let guild_id = channel.guild_id.ok_or_else(|| {
            ServiceError::validation("DM channels cannot be updated")
        })?;

        // Check permissions
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_CHANNELS)
            .await?;

        let mut changed = false;

        // Update name
        if let Some(name) = request.name {
            channel.name = Some(name);
            changed = true;
        }

        // Update topic (text channels only)
        if let Some(topic) = request.topic {
            if channel.channel_type != ChannelType::GuildCategory {
                channel.topic = Some(topic);
                changed = true;
            }
        }

        // Update position
        if let Some(position) = request.position {
            channel.position = position;
            changed = true;
        }

        // Update parent
        if let Some(ref parent_str) = request.parent_id {
            let parent = parent_str
                .parse::<i64>()
                .map(Snowflake::new)
                .map_err(|_| ServiceError::validation("Invalid parent_id format"))?;

            // Verify parent is a category in the same guild
            if let Some(parent_channel) = self.ctx.channel_repo().find_by_id(parent).await? {
                if parent_channel.channel_type != ChannelType::GuildCategory {
                    return Err(ServiceError::validation("Parent must be a category channel"));
                }
                if parent_channel.guild_id != Some(guild_id) {
                    return Err(ServiceError::validation("Parent must be in the same guild"));
                }
            } else {
                return Err(ServiceError::not_found("Channel", parent_str.clone()));
            }

            channel.parent_id = Some(parent);
            changed = true;
        }

        if changed {
            channel.updated_at = Utc::now();
            self.ctx.channel_repo().update(&channel).await?;

            // Publish CHANNEL_UPDATE event
            self.publish_channel_event("CHANNEL_UPDATE", &channel).await;
        }

        Ok(ChannelResponse::from(&channel))
    }

    /// Delete channel
    #[instrument(skip(self))]
    pub async fn delete_channel(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // DM channels cannot be deleted
        let guild_id = channel.guild_id.ok_or_else(|| {
            ServiceError::validation("DM channels cannot be deleted")
        })?;

        // Check permissions
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_CHANNELS)
            .await?;

        self.ctx.channel_repo().delete(channel_id).await?;

        info!(channel_id = %channel_id, guild_id = %guild_id, "Channel deleted");

        // Publish CHANNEL_DELETE event
        let event = PubSubEvent::new(
            "CHANNEL_DELETE",
            json!({
                "id": channel_id.to_string(),
                "guild_id": guild_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get all channels in a guild
    #[instrument(skip(self))]
    pub async fn get_guild_channels(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<ChannelResponse>> {
        // Check if user is a member
        let permission_service = PermissionService::new(self.ctx);
        if !permission_service.is_guild_member(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Guild", guild_id.to_string()));
        }

        let channels = self.ctx.channel_repo().find_by_guild(guild_id).await?;

        // Filter channels based on VIEW_CHANNEL permission
        let user_perms = permission_service
            .get_member_permissions(guild_id, user_id)
            .await?;

        let visible_channels: Vec<_> = channels
            .into_iter()
            .filter(|_| user_perms.has(Permissions::VIEW_CHANNEL))
            .map(ChannelResponse::from)
            .collect();

        Ok(visible_channels)
    }

    /// Helper to publish channel events
    async fn publish_channel_event(&self, event_type: &str, channel: &Channel) {
        if let Some(guild_id) = channel.guild_id {
            let data = json!({
                "id": channel.id.to_string(),
                "guild_id": guild_id.to_string(),
                "name": channel.name,
                "type": match channel.channel_type {
                    ChannelType::GuildText => 0,
                    ChannelType::Dm => 1,
                    ChannelType::GuildCategory => 4,
                },
                "topic": channel.topic,
                "position": channel.position,
                "parent_id": channel.parent_id.map(|id| id.to_string())
            });

            let event = PubSubEvent::new(event_type, data);
            self.ctx
                .publisher()
                .publish(&PubSubChannel::guild(guild_id), &event)
                .await
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
