//! Message service
//!
//! Handles message creation, editing, deletion, and queries.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::{Channel, Message};
use chat_core::traits::MessageQuery;
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument, warn};

use crate::dto::{
    CreateMessageRequest, MessageResponse, MessageWithDetails, UpdateMessageRequest, UserResponse,
};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Message service
pub struct MessageService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> MessageService<'a> {
    /// Create a new MessageService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create a new message
    #[instrument(skip(self, request))]
    pub async fn create_message(
        &self,
        channel_id: Snowflake,
        author_id: Snowflake,
        request: CreateMessageRequest,
    ) -> ServiceResult<MessageResponse> {
        // Get channel and verify access
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // Check permissions based on channel type
        if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .require_permission(guild_id, author_id, Permissions::VIEW_CHANNEL)
                .await?;
            permission_service
                .require_permission(guild_id, author_id, Permissions::SEND_MESSAGES)
                .await?;
        } else {
            // DM channel - verify user is a recipient
            let recipients = self.ctx.channel_repo().get_dm_recipients(channel_id).await?;
            if !recipients.contains(&author_id) {
                return Err(ServiceError::not_found("Channel", channel_id.to_string()));
            }
        }

        // Parse message reference if replying
        let reference_id = if let Some(ref msg_ref) = request.message_reference {
            let ref_id = msg_ref
                .message_id
                .parse::<i64>()
                .map(Snowflake::new)
                .map_err(|_| ServiceError::validation("Invalid message_id in reference"))?;

            // Verify referenced message exists in same channel
            if let Some(ref_msg) = self.ctx.message_repo().find_by_id(ref_id).await? {
                if ref_msg.channel_id != channel_id {
                    return Err(ServiceError::validation(
                        "Referenced message must be in the same channel",
                    ));
                }
            } else {
                warn!(ref_id = %ref_id, "Referenced message not found");
            }

            Some(ref_id)
        } else {
            None
        };

        let message_id = self.ctx.generate_id();
        let now = Utc::now();

        let message = Message {
            id: message_id,
            channel_id,
            author_id,
            content: request.content,
            created_at: now,
            edited_at: None,
            reference_id,
        };

        self.ctx.message_repo().create(&message).await?;

        // Get author for response
        let author = self
            .ctx
            .user_repo()
            .find_by_id(author_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", author_id.to_string()))?;

        info!(message_id = %message_id, channel_id = %channel_id, "Message created");

        // Publish MESSAGE_CREATE event
        self.publish_message_create(&channel, &message, &author).await;

        Ok(MessageResponse::from(MessageWithDetails {
            message,
            author,
            guild_id: channel.guild_id,
            attachments: vec![],
            reactions: vec![],
            reference: None,
        }))
    }

    /// Get message by ID
    #[instrument(skip(self))]
    pub async fn get_message(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<MessageResponse> {
        // Verify channel access
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        // Verify message is in this channel
        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Get author
        let author = self
            .ctx
            .user_repo()
            .find_by_id(message.author_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Message author not found"))?;

        // Get attachments
        let attachments = self
            .ctx
            .attachment_repo()
            .find_by_message(message_id)
            .await?;

        // Get reactions with counts
        let reaction_counts = self
            .ctx
            .reaction_repo()
            .count_by_emoji(message_id)
            .await?;

        // Check if user reacted
        let reactions: Vec<_> = {
            let mut result = Vec::new();
            for (emoji, count) in reaction_counts {
                let me = self
                    .ctx
                    .reaction_repo()
                    .find(message_id, user_id, &emoji)
                    .await?
                    .is_some();
                result.push((emoji, count, me));
            }
            result
        };

        Ok(MessageResponse::from(MessageWithDetails {
            message,
            author,
            guild_id: channel.guild_id,
            attachments,
            reactions,
            reference: None,
        }))
    }

    /// Update message
    #[instrument(skip(self, request))]
    pub async fn update_message(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
        request: UpdateMessageRequest,
    ) -> ServiceResult<MessageResponse> {
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        let mut message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        // Only author can edit their message
        if message.author_id != user_id {
            return Err(ServiceError::permission_denied("Can only edit own messages"));
        }

        // Verify message is in this channel
        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        message.content = request.content;
        message.edited_at = Some(Utc::now());

        self.ctx.message_repo().update(&message).await?;

        // Get author for response
        let author = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        info!(message_id = %message_id, "Message updated");

        // Publish MESSAGE_UPDATE event
        self.publish_message_update(&channel, &message).await;

        Ok(MessageResponse::from(MessageWithDetails {
            message,
            author,
            guild_id: channel.guild_id,
            attachments: vec![],
            reactions: vec![],
            reference: None,
        }))
    }

    /// Delete message
    #[instrument(skip(self))]
    pub async fn delete_message(
        &self,
        channel_id: Snowflake,
        message_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        let message = self
            .ctx
            .message_repo()
            .find_by_id(message_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Message", message_id.to_string()))?;

        // Verify message is in this channel
        if message.channel_id != channel_id {
            return Err(ServiceError::not_found("Message", message_id.to_string()));
        }

        // Can delete if:
        // 1. Own message
        // 2. Has MANAGE_MESSAGES permission in guild
        let can_delete = if message.author_id == user_id {
            true
        } else if let Some(guild_id) = channel.guild_id {
            let permission_service = PermissionService::new(self.ctx);
            permission_service
                .check_permission(guild_id, user_id, Permissions::MANAGE_MESSAGES)
                .await?
        } else {
            // DM - can only delete own messages
            false
        };

        if !can_delete {
            return Err(ServiceError::permission_denied("MANAGE_MESSAGES"));
        }

        self.ctx.message_repo().delete(message_id).await?;

        info!(message_id = %message_id, "Message deleted");

        // Publish MESSAGE_DELETE event
        self.publish_message_delete(&channel, message_id).await;

        Ok(())
    }

    /// Bulk delete messages
    #[instrument(skip(self, message_ids))]
    pub async fn bulk_delete_messages(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        message_ids: Vec<String>,
    ) -> ServiceResult<u64> {
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        // Bulk delete only works in guild channels
        let guild_id = channel
            .guild_id
            .ok_or_else(|| ServiceError::validation("Bulk delete only works in guild channels"))?;

        // Check MANAGE_MESSAGES permission
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_MESSAGES)
            .await?;

        // Parse message IDs
        let snowflake_ids: Result<Vec<_>, _> = message_ids
            .iter()
            .map(|s| {
                s.parse::<i64>()
                    .map(Snowflake::new)
                    .map_err(|_| ServiceError::validation("Invalid message ID format"))
            })
            .collect();
        let snowflake_ids = snowflake_ids?;

        let deleted_count = self
            .ctx
            .message_repo()
            .bulk_delete(channel_id, &snowflake_ids)
            .await?;

        info!(
            channel_id = %channel_id,
            count = deleted_count,
            "Messages bulk deleted"
        );

        // Publish MESSAGE_DELETE_BULK event
        let event = PubSubEvent::new(
            "MESSAGE_DELETE_BULK",
            json!({
                "ids": message_ids,
                "channel_id": channel_id.to_string(),
                "guild_id": guild_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel_id), &event)
            .await
            .ok();

        Ok(deleted_count)
    }

    /// Get messages in a channel with pagination
    #[instrument(skip(self))]
    pub async fn get_channel_messages(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        before: Option<Snowflake>,
        after: Option<Snowflake>,
        limit: i64,
    ) -> ServiceResult<Vec<MessageResponse>> {
        let channel = self.verify_channel_access(channel_id, user_id).await?;

        let query = MessageQuery {
            before,
            after,
            limit: limit.min(100), // Cap at 100
        };

        let messages = self
            .ctx
            .message_repo()
            .find_by_channel(channel_id, query)
            .await?;

        // Build responses with author info
        let mut responses = Vec::with_capacity(messages.len());

        for message in messages {
            let author = self
                .ctx
                .user_repo()
                .find_by_id(message.author_id)
                .await?
                .map(|u| crate::dto::UserResponse::from(&u))
                .unwrap_or_else(|| UserResponse {
                    id: message.author_id.to_string(),
                    username: "[Deleted User]".to_string(),
                    discriminator: "0000".to_string(),
                    avatar: None,
                    bot: false,
                    system: false,
                    created_at: message.created_at,
                });

            // Get attachments
            let attachments = self
                .ctx
                .attachment_repo()
                .find_by_message(message.id)
                .await?;

            // Get reactions
            let reaction_counts = self.ctx.reaction_repo().count_by_emoji(message.id).await?;
            let reactions: Vec<_> = {
                let mut result = Vec::new();
                for (emoji, count) in reaction_counts {
                    let me = self
                        .ctx
                        .reaction_repo()
                        .find(message.id, user_id, &emoji)
                        .await?
                        .is_some();
                    result.push((emoji, count, me));
                }
                result
            };

            responses.push(MessageResponse::from(MessageWithDetails {
                message,
                author: chat_core::entities::User {
                    id: author.id.parse::<i64>().map(Snowflake::new).unwrap_or_default(),
                    username: author.username,
                    discriminator: author.discriminator,
                    email: String::new(),
                    avatar: author.avatar,
                    bot: author.bot,
                    system: author.system,
                    created_at: author.created_at,
                    updated_at: author.created_at,
                },
                guild_id: channel.guild_id,
                attachments,
                reactions,
                reference: None,
            }));
        }

        Ok(responses)
    }

    /// Verify user has access to channel
    async fn verify_channel_access(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Channel> {
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

    /// Helper to publish MESSAGE_CREATE event
    async fn publish_message_create(
        &self,
        channel: &Channel,
        message: &Message,
        author: &chat_core::entities::User,
    ) {
        let pubsub_channel = PubSubChannel::channel(channel.id);

        let data = json!({
            "id": message.id.to_string(),
            "channel_id": message.channel_id.to_string(),
            "guild_id": channel.guild_id.map(|id| id.to_string()),
            "author": {
                "id": author.id.to_string(),
                "username": author.username,
                "discriminator": author.discriminator,
                "avatar": author.avatar
            },
            "content": message.content,
            "timestamp": message.created_at.to_rfc3339(),
            "edited_timestamp": message.edited_at.map(|t| t.to_rfc3339()),
            "message_reference": message.reference_id.map(|id| {
                json!({"message_id": id.to_string()})
            })
        });

        let event = PubSubEvent::new("MESSAGE_CREATE", data);
        self.ctx
            .publisher()
            .publish(&pubsub_channel, &event)
            .await
            .ok();
    }

    /// Helper to publish MESSAGE_UPDATE event
    async fn publish_message_update(&self, channel: &Channel, message: &Message) {
        let data = json!({
            "id": message.id.to_string(),
            "channel_id": message.channel_id.to_string(),
            "guild_id": channel.guild_id.map(|id| id.to_string()),
            "content": message.content,
            "edited_timestamp": message.edited_at.map(|t| t.to_rfc3339())
        });

        let event = PubSubEvent::new("MESSAGE_UPDATE", data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel.id), &event)
            .await
            .ok();
    }

    /// Helper to publish MESSAGE_DELETE event
    async fn publish_message_delete(&self, channel: &Channel, message_id: Snowflake) {
        let data = json!({
            "id": message_id.to_string(),
            "channel_id": channel.id.to_string(),
            "guild_id": channel.guild_id.map(|id| id.to_string())
        });

        let event = PubSubEvent::new("MESSAGE_DELETE", data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::channel(channel.id), &event)
            .await
            .ok();
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
