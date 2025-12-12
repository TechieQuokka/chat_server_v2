//! DM (Direct Message) service
//!
//! Handles direct message channel creation and management.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::{Channel, ChannelType};
use chat_core::Snowflake;
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{DmChannelResponse, DmChannelWithRecipients};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};

/// DM service
pub struct DmService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> DmService<'a> {
    /// Create a new DmService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create or get existing DM channel with a user
    #[instrument(skip(self))]
    pub async fn create_dm(
        &self,
        user_id: Snowflake,
        recipient_id: Snowflake,
    ) -> ServiceResult<DmChannelResponse> {
        // Cannot DM yourself
        if user_id == recipient_id {
            return Err(ServiceError::validation("Cannot create DM with yourself"));
        }

        // Verify recipient exists
        let recipient = self
            .ctx
            .user_repo()
            .find_by_id(recipient_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", recipient_id.to_string()))?;

        // Check if DM channel already exists between these users
        if let Some(existing_channel) = self
            .ctx
            .channel_repo()
            .find_dm(user_id, recipient_id)
            .await?
        {
            // Return existing channel
            let recipients = self
                .ctx
                .channel_repo()
                .get_dm_recipients(existing_channel.id)
                .await?;

            let mut recipient_users = Vec::new();
            for rid in recipients {
                if rid != user_id {
                    if let Some(user) = self.ctx.user_repo().find_by_id(rid).await? {
                        recipient_users.push(user);
                    }
                }
            }

            return Ok(DmChannelResponse::from(DmChannelWithRecipients {
                channel: existing_channel,
                recipients: recipient_users,
                last_message_id: None,
            }));
        }

        // Create new DM channel
        let channel_id = self.ctx.generate_id();
        let now = Utc::now();

        let channel = Channel {
            id: channel_id,
            guild_id: None, // DM channels have no guild
            name: None,     // DM channels have no name
            channel_type: ChannelType::Dm,
            topic: None,
            position: 0,
            parent_id: None,
            created_at: now,
            updated_at: now,
        };

        self.ctx.channel_repo().create(&channel).await?;

        // Add both users as recipients
        self.ctx
            .channel_repo()
            .add_dm_recipient(channel_id, user_id)
            .await?;
        self.ctx
            .channel_repo()
            .add_dm_recipient(channel_id, recipient_id)
            .await?;

        info!(
            channel_id = %channel_id,
            user_id = %user_id,
            recipient_id = %recipient_id,
            "DM channel created"
        );

        // Get current user for recipients list
        let current_user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Current user not found"))?;

        // Publish CHANNEL_CREATE event to both users
        let channel_data = json!({
            "id": channel_id.to_string(),
            "type": 1, // DM type
            "recipients": [
                {
                    "id": recipient.id.to_string(),
                    "username": recipient.username,
                    "discriminator": recipient.discriminator,
                    "avatar": recipient.avatar
                }
            ]
        });

        // Publish to initiator
        let event = PubSubEvent::new("CHANNEL_CREATE", channel_data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::user(user_id), &event)
            .await
            .ok();

        // Publish to recipient with swapped recipient list
        let recipient_channel_data = json!({
            "id": channel_id.to_string(),
            "type": 1,
            "recipients": [
                {
                    "id": current_user.id.to_string(),
                    "username": current_user.username,
                    "discriminator": current_user.discriminator,
                    "avatar": current_user.avatar
                }
            ]
        });

        let recipient_event = PubSubEvent::new("CHANNEL_CREATE", recipient_channel_data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::user(recipient_id), &recipient_event)
            .await
            .ok();

        Ok(DmChannelResponse::from(DmChannelWithRecipients {
            channel,
            recipients: vec![recipient],
            last_message_id: None,
        }))
    }

    /// Get user's DM channels
    #[instrument(skip(self))]
    pub async fn get_user_dms(&self, user_id: Snowflake) -> ServiceResult<Vec<DmChannelResponse>> {
        let channels = self.ctx.channel_repo().find_dms_by_user(user_id).await?;

        let mut responses = Vec::with_capacity(channels.len());

        for channel in channels {
            let recipients = self
                .ctx
                .channel_repo()
                .get_dm_recipients(channel.id)
                .await?;

            let mut recipient_users = Vec::new();
            for rid in recipients {
                // Don't include the requesting user in recipients list
                if rid != user_id {
                    if let Some(user) = self.ctx.user_repo().find_by_id(rid).await? {
                        recipient_users.push(user);
                    }
                }
            }

            responses.push(DmChannelResponse::from(DmChannelWithRecipients {
                channel,
                recipients: recipient_users,
                last_message_id: None,
            }));
        }

        Ok(responses)
    }

    /// Get a specific DM channel
    #[instrument(skip(self))]
    pub async fn get_dm_channel(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<DmChannelResponse> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // Verify it's a DM channel
        if channel.guild_id.is_some() {
            return Err(ServiceError::not_found("DM Channel", channel_id.to_string()));
        }

        // Verify user is a recipient
        let recipients = self
            .ctx
            .channel_repo()
            .get_dm_recipients(channel_id)
            .await?;

        if !recipients.contains(&user_id) {
            return Err(ServiceError::not_found("DM Channel", channel_id.to_string()));
        }

        let mut recipient_users = Vec::new();
        for rid in recipients {
            // Don't include the requesting user in recipients list
            if rid != user_id {
                if let Some(user) = self.ctx.user_repo().find_by_id(rid).await? {
                    recipient_users.push(user);
                }
            }
        }

        Ok(DmChannelResponse::from(DmChannelWithRecipients {
            channel,
            recipients: recipient_users,
            last_message_id: None,
        }))
    }

    /// Close DM channel (removes from user's DM list, doesn't delete messages)
    #[instrument(skip(self))]
    pub async fn close_dm(&self, channel_id: Snowflake, user_id: Snowflake) -> ServiceResult<()> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // Verify it's a DM channel
        if channel.guild_id.is_some() {
            return Err(ServiceError::not_found("DM Channel", channel_id.to_string()));
        }

        // Verify user is a recipient
        let recipients = self
            .ctx
            .channel_repo()
            .get_dm_recipients(channel_id)
            .await?;

        if !recipients.contains(&user_id) {
            return Err(ServiceError::not_found("DM Channel", channel_id.to_string()));
        }

        // Note: We don't have a remove_dm_recipient method in the trait.
        // For now, we'll just publish the event. In production, this should
        // mark the DM as "closed" for this user in the database.

        info!(
            channel_id = %channel_id,
            user_id = %user_id,
            "DM channel closed"
        );

        // Publish CHANNEL_DELETE event to the user
        let event = PubSubEvent::new(
            "CHANNEL_DELETE",
            json!({
                "id": channel_id.to_string(),
                "type": 1
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::user(user_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get DM channel between two specific users (internal use)
    pub async fn find_dm_between(
        &self,
        user1: Snowflake,
        user2: Snowflake,
    ) -> ServiceResult<Option<Channel>> {
        self.ctx
            .channel_repo()
            .find_dm(user1, user2)
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
