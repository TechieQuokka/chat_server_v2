//! Guild service
//!
//! Handles guild (server) creation, management, and queries.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::{Channel, Guild, GuildMember, Role};
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{
    CreateGuildRequest, GuildResponse, GuildWithCounts, GuildWithCountsResponse, UpdateGuildRequest,
};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Guild service
pub struct GuildService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> GuildService<'a> {
    /// Create a new GuildService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create a new guild
    #[instrument(skip(self, request))]
    pub async fn create_guild(
        &self,
        owner_id: Snowflake,
        request: CreateGuildRequest,
    ) -> ServiceResult<GuildResponse> {
        let guild_id = self.ctx.generate_id();
        let now = Utc::now();

        let guild = Guild {
            id: guild_id,
            name: request.name,
            icon: request.icon,
            description: request.description,
            owner_id,
            created_at: now,
            updated_at: now,
        };

        // Create guild
        self.ctx.guild_repo().create(&guild).await?;

        // Create @everyone role (id first, then guild_id)
        let everyone_role = Role::everyone(self.ctx.generate_id(), guild_id);
        self.ctx.role_repo().create(&everyone_role).await?;

        // Create default "general" text channel
        let channel_id = self.ctx.generate_id();
        let default_channel = Channel::new_text(channel_id, guild_id, "general".to_string());
        self.ctx.channel_repo().create(&default_channel).await?;

        // Add owner as member
        let member = GuildMember {
            guild_id,
            user_id: owner_id,
            nickname: None,
            role_ids: vec![],
            joined_at: now,
            updated_at: now,
        };
        self.ctx.member_repo().create(&member).await?;

        info!(guild_id = %guild_id, owner_id = %owner_id, "Guild created successfully");

        // Publish GUILD_CREATE event
        self.publish_guild_event("GUILD_CREATE", &guild).await;

        Ok(GuildResponse::from(&guild))
    }

    /// Get guild by ID
    #[instrument(skip(self))]
    pub async fn get_guild(&self, guild_id: Snowflake) -> ServiceResult<GuildResponse> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        Ok(GuildResponse::from(&guild))
    }

    /// Get guild with member and channel counts
    #[instrument(skip(self))]
    pub async fn get_guild_with_counts(
        &self,
        guild_id: Snowflake,
    ) -> ServiceResult<GuildWithCountsResponse> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        let member_count = self.ctx.guild_repo().member_count(guild_id).await?;
        let channels = self.ctx.channel_repo().find_by_guild(guild_id).await?;

        Ok(GuildWithCountsResponse::from(GuildWithCounts {
            guild,
            member_count,
            channel_count: channels.len() as i64,
        }))
    }

    /// Get guild entity by ID
    #[instrument(skip(self))]
    pub async fn get_guild_entity(&self, guild_id: Snowflake) -> ServiceResult<Guild> {
        self.ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))
    }

    /// Update guild settings
    #[instrument(skip(self, request))]
    pub async fn update_guild(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        request: UpdateGuildRequest,
    ) -> ServiceResult<GuildResponse> {
        // Check permissions
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_GUILD)
            .await?;

        let mut guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        let mut changed = false;

        // Update name
        if let Some(name) = request.name {
            guild.name = name;
            changed = true;
        }

        // Update icon
        if let Some(icon) = request.icon {
            guild.icon = Some(icon);
            changed = true;
        }

        // Update description
        if let Some(description) = request.description {
            guild.description = Some(description);
            changed = true;
        }

        // Transfer ownership
        if let Some(new_owner_id) = request.owner_id {
            // Only owner can transfer ownership
            if guild.owner_id != user_id {
                return Err(ServiceError::permission_denied("Only owner can transfer ownership"));
            }

            let new_owner = new_owner_id
                .parse::<i64>()
                .map(Snowflake::new)
                .map_err(|_| ServiceError::validation("Invalid owner_id format"))?;

            // Verify new owner is a member
            if !self.ctx.member_repo().is_member(guild_id, new_owner).await? {
                return Err(ServiceError::validation("New owner must be a guild member"));
            }

            guild.owner_id = new_owner;
            changed = true;
            info!(guild_id = %guild_id, old_owner = %user_id, new_owner = %new_owner, "Guild ownership transferred");
        }

        if changed {
            guild.updated_at = Utc::now();
            self.ctx.guild_repo().update(&guild).await?;

            // Publish GUILD_UPDATE event
            self.publish_guild_event("GUILD_UPDATE", &guild).await;
        }

        Ok(GuildResponse::from(&guild))
    }

    /// Delete guild
    #[instrument(skip(self))]
    pub async fn delete_guild(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Only owner can delete guild
        if guild.owner_id != user_id {
            return Err(ServiceError::permission_denied("Only owner can delete guild"));
        }

        // Soft delete guild (cascades to channels, roles, members, etc.)
        self.ctx.guild_repo().delete(guild_id).await?;

        info!(guild_id = %guild_id, "Guild deleted");

        // Publish GUILD_DELETE event
        let event = PubSubEvent::new("GUILD_DELETE", json!({ "id": guild_id.to_string() }));
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get all guilds for a user
    #[instrument(skip(self))]
    pub async fn get_user_guilds(&self, user_id: Snowflake) -> ServiceResult<Vec<GuildResponse>> {
        let guilds = self.ctx.guild_repo().find_by_user(user_id).await?;
        Ok(guilds.into_iter().map(GuildResponse::from).collect())
    }

    /// Leave guild
    #[instrument(skip(self))]
    pub async fn leave_guild(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Owner cannot leave, must transfer ownership first
        if guild.owner_id == user_id {
            return Err(ServiceError::conflict(
                "Guild owner cannot leave. Transfer ownership first.",
            ));
        }

        // Check if user is a member
        if !self.ctx.member_repo().is_member(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Member", format!("{guild_id}/{user_id}")));
        }

        // Remove member
        self.ctx.member_repo().delete(guild_id, user_id).await?;

        info!(guild_id = %guild_id, user_id = %user_id, "User left guild");

        // Publish GUILD_MEMBER_REMOVE event
        let event = PubSubEvent::new(
            "GUILD_MEMBER_REMOVE",
            json!({
                "guild_id": guild_id.to_string(),
                "user_id": user_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Helper to publish guild events
    async fn publish_guild_event(&self, event_type: &str, guild: &Guild) {
        let data = json!({
            "id": guild.id.to_string(),
            "name": guild.name,
            "icon": guild.icon,
            "description": guild.description,
            "owner_id": guild.owner_id.to_string(),
            "created_at": guild.created_at.to_rfc3339()
        });

        let event = PubSubEvent::new(event_type, data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild.id), &event)
            .await
            .ok();
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
