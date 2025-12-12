//! Reaction service
//!
//! Handles message reactions (add, remove, query).

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::Reaction;
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::ReactionResponse;

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Reaction service
pub struct ReactionService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> ReactionService<'a> {
    /// Create a new ReactionService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Add reaction to a message
    #[instrument(skip(self))]
    pub async fn add_reaction(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: String,
    ) -> ServiceResult<()> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists in this channel
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Check if user can add reactions (ADD_REACTIONS permission for guild channels)
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, user_id, Permissions::ADD_REACTIONS)
                .await?;
        }

        // Check if reaction already exists
        if self
            .ctx
            .reaction_repo()
            .find(message_id, user_id, &emoji)
            .await?
            .is_some()
        {
            // Already reacted with this emoji, no-op
            return Ok(());
        }

        let now = Utc::now();
        let reaction = Reaction {
            message_id,
            user_id,
            emoji: emoji.clone(),
            created_at: now,
        };

        self.ctx.reaction_repo().create(&reaction).await?;

        info!(
            message_id = %message_id,
            user_id = %user_id,
            emoji = %emoji,
            "Reaction added"
        );

        // Publish MESSAGE_REACTION_ADD event
        let event = PubSubEvent::new(
            "MESSAGE_REACTION_ADD",
            json!({
                "user_id": user_id.to_string(),
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
                "guild_id": channel.guild_id.map(|id| id.to_string()),
                "emoji": { "name": emoji }
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Remove reaction from a message
    #[instrument(skip(self))]
    pub async fn remove_reaction(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: String,
    ) -> ServiceResult<()> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists in this channel
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Delete the reaction
        self.ctx
            .reaction_repo()
            .delete(message_id, user_id, &emoji)
            .await?;

        info!(
            message_id = %message_id,
            user_id = %user_id,
            emoji = %emoji,
            "Reaction removed"
        );

        // Publish MESSAGE_REACTION_REMOVE event
        let event = PubSubEvent::new(
            "MESSAGE_REACTION_REMOVE",
            json!({
                "user_id": user_id.to_string(),
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
                "guild_id": channel.guild_id.map(|id| id.to_string()),
                "emoji": { "name": emoji }
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Remove another user's reaction (requires MANAGE_MESSAGES)
    #[instrument(skip(self))]
    pub async fn remove_user_reaction(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        actor_id: Snowflake,
        target_user_id: Snowflake,
        emoji: String,
    ) -> ServiceResult<()> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, actor_id).await?;

        // Verify message exists
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Check MANAGE_MESSAGES permission for removing others' reactions
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, actor_id, Permissions::MANAGE_MESSAGES)
                .await?;
        } else {
            // DMs - can only remove own reactions
            if actor_id != target_user_id {
                return Err(ServiceError::permission_denied(
                    "Cannot remove others' reactions in DMs",
                ));
            }
        }

        self.ctx
            .reaction_repo()
            .delete(message_id, target_user_id, &emoji)
            .await?;

        info!(
            message_id = %message_id,
            target_user_id = %target_user_id,
            emoji = %emoji,
            "User reaction removed by moderator"
        );

        // Publish MESSAGE_REACTION_REMOVE event
        let event = PubSubEvent::new(
            "MESSAGE_REACTION_REMOVE",
            json!({
                "user_id": target_user_id.to_string(),
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
                "guild_id": channel.guild_id.map(|id| id.to_string()),
                "emoji": { "name": emoji }
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Remove all reactions from a message
    #[instrument(skip(self))]
    pub async fn remove_all_reactions(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Requires MANAGE_MESSAGES permission
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, user_id, Permissions::MANAGE_MESSAGES)
                .await?;
        } else {
            return Err(ServiceError::permission_denied(
                "Cannot clear reactions in DMs",
            ));
        }

        self.ctx
            .reaction_repo()
            .delete_all(message_id)
            .await?;

        info!(message_id = %message_id, "All reactions removed");

        // Publish MESSAGE_REACTION_REMOVE_ALL event
        let event = PubSubEvent::new(
            "MESSAGE_REACTION_REMOVE_ALL",
            json!({
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
                "guild_id": channel.guild_id.map(|id| id.to_string())
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Remove all reactions for a specific emoji
    #[instrument(skip(self))]
    pub async fn remove_all_reactions_for_emoji(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: String,
    ) -> ServiceResult<()> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Requires MANAGE_MESSAGES permission
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, user_id, Permissions::MANAGE_MESSAGES)
                .await?;
        } else {
            return Err(ServiceError::permission_denied(
                "Cannot clear reactions in DMs",
            ));
        }

        self.ctx
            .reaction_repo()
            .delete_by_emoji(message_id, &emoji)
            .await?;

        info!(
            message_id = %message_id,
            emoji = %emoji,
            "All reactions for emoji removed"
        );

        // Publish MESSAGE_REACTION_REMOVE_EMOJI event
        let event = PubSubEvent::new(
            "MESSAGE_REACTION_REMOVE_EMOJI",
            json!({
                "channel_id": channel_id.to_string(),
                "message_id": message_id.to_string(),
                "guild_id": channel.guild_id.map(|id| id.to_string()),
                "emoji": { "name": emoji }
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get reactions for a message
    #[instrument(skip(self))]
    pub async fn get_reactions(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: Option<String>,
    ) -> ServiceResult<Vec<ReactionResponse>> {
        // Verify channel access
        self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Get reaction counts grouped by emoji
        let reaction_counts = self
            .ctx
            .reaction_repo()
            .count_by_emoji(message_id)
            .await?;

        let mut responses = Vec::with_capacity(reaction_counts.len());

        for (reaction_emoji, count) in reaction_counts {
            // Filter by emoji if specified
            if let Some(ref filter_emoji) = emoji {
                if &reaction_emoji != filter_emoji {
                    continue;
                }
            }

            // Check if current user has reacted
            let me = self
                .ctx
                .reaction_repo()
                .find(message_id, user_id, &reaction_emoji)
                .await?
                .is_some();

            responses.push(ReactionResponse {
                emoji: reaction_emoji,
                count,
                me,
            });
        }

        Ok(responses)
    }

    /// Get users who reacted with a specific emoji
    #[instrument(skip(self))]
    pub async fn get_reaction_users(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: String,
        limit: i64,
        after: Option<Snowflake>,
    ) -> ServiceResult<Vec<crate::dto::UserResponse>> {
        // Verify channel access
        self.verify_channel_access(channel_id, user_id).await?;

        // Verify message exists
        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Get user IDs who reacted with this emoji
        let user_ids = self
            .ctx
            .reaction_repo()
            .find_users_by_emoji(message_id, &emoji, limit.min(100))
            .await?;

        // Filter by 'after' cursor if provided
        let user_ids: Vec<_> = if let Some(after_id) = after {
            user_ids.into_iter().filter(|id| *id > after_id).collect()
        } else {
            user_ids
        };

        let mut users = Vec::with_capacity(user_ids.len());

        for user_id in user_ids {
            if let Some(user) = self.ctx.user_repo().find_by_id(user_id).await? {
                users.push(crate::dto::UserResponse::from(&user));
            }
        }

        Ok(users)
    }

    /// Verify user has access to the channel
    async fn verify_channel_access(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<chat_core::entities::Channel> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

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
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
