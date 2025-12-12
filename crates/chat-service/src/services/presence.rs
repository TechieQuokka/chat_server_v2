//! Presence service
//!
//! Handles user presence (online status) management and updates.

use chat_cache::{PubSubChannel, PubSubEvent, UserStatus};
use chat_core::Snowflake;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::PresenceResponse;

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Presence service
pub struct PresenceService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> PresenceService<'a> {
    /// Create a new PresenceService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Update user's presence status
    #[instrument(skip(self))]
    pub async fn update_presence(
        &self,
        user_id: Snowflake,
        status: UserStatus,
    ) -> ServiceResult<()> {
        // Verify user exists
        let user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        // Update presence in cache
        self.ctx
            .presence_store()
            .update_status(user_id, status)
            .await
            .map_err(|e| ServiceError::internal(format!("Failed to update presence: {e}")))?;

        info!(user_id = %user_id, status = %status, "Presence updated");

        // Publish PRESENCE_UPDATE event to all guilds the user is in
        let guilds = self.ctx.guild_repo().find_by_user(user_id).await?;

        for guild in guilds {
            let event = PubSubEvent::new(
                "PRESENCE_UPDATE",
                json!({
                    "user": {
                        "id": user.id.to_string(),
                        "username": user.username,
                        "discriminator": user.discriminator,
                        "avatar": user.avatar
                    },
                    "guild_id": guild.id.to_string(),
                    "status": status.to_string(),
                    "activities": []
                }),
            );
            self.ctx
                .publisher()
                .publish(&PubSubChannel::guild(guild.id), &event)
                .await
                .ok();
        }

        Ok(())
    }

    /// Get user's current presence
    #[instrument(skip(self))]
    pub async fn get_presence(&self, user_id: Snowflake) -> ServiceResult<PresenceResponse> {
        let _user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        let presence_data = self
            .ctx
            .presence_store()
            .get_presence(user_id)
            .await
            .map_err(|e| ServiceError::internal(format!("Failed to get presence: {e}")))?;

        let (status, custom_status, last_seen) = if let Some(data) = presence_data {
            (
                data.status.to_string(),
                data.custom_status,
                Some(chrono::DateTime::from_timestamp(data.updated_at, 0).unwrap_or_default()),
            )
        } else {
            (UserStatus::Offline.to_string(), None, None)
        };

        Ok(PresenceResponse {
            user_id: user_id.to_string(),
            status,
            custom_status,
            last_seen,
        })
    }

    /// Get presence for multiple users
    #[instrument(skip(self, user_ids))]
    pub async fn get_presences(
        &self,
        user_ids: &[Snowflake],
    ) -> ServiceResult<Vec<PresenceResponse>> {
        let mut responses = Vec::with_capacity(user_ids.len());

        for &user_id in user_ids {
            let presence_data = self
                .ctx
                .presence_store()
                .get_presence(user_id)
                .await
                .map_err(|e| ServiceError::internal(format!("Failed to get presence: {e}")))?;

            let (status, custom_status, last_seen) = if let Some(data) = presence_data {
                (
                    data.status.to_string(),
                    data.custom_status,
                    Some(chrono::DateTime::from_timestamp(data.updated_at, 0).unwrap_or_default()),
                )
            } else {
                (UserStatus::Offline.to_string(), None, None)
            };

            responses.push(PresenceResponse {
                user_id: user_id.to_string(),
                status,
                custom_status,
                last_seen,
            });
        }

        Ok(responses)
    }

    /// Get presence of all members in a guild
    #[instrument(skip(self))]
    pub async fn get_guild_presences(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<PresenceResponse>> {
        // Verify user is a member
        let permission_service = PermissionService::new(self.ctx);
        if !permission_service.is_guild_member(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Guild", guild_id.to_string()));
        }

        // Get all members
        let members = self
            .ctx
            .member_repo()
            .find_by_guild(guild_id, 1000, None)
            .await?;

        let mut responses = Vec::with_capacity(members.len());

        for member in members {
            let presence_data = self
                .ctx
                .presence_store()
                .get_presence(member.user_id)
                .await
                .map_err(|e| ServiceError::internal(format!("Failed to get presence: {e}")))?;

            let (status, custom_status, last_seen) = if let Some(data) = presence_data {
                (
                    data.status.to_string(),
                    data.custom_status,
                    Some(chrono::DateTime::from_timestamp(data.updated_at, 0).unwrap_or_default()),
                )
            } else {
                (UserStatus::Offline.to_string(), None, None)
            };

            responses.push(PresenceResponse {
                user_id: member.user_id.to_string(),
                status,
                custom_status,
                last_seen,
            });
        }

        Ok(responses)
    }

    /// Set user online (called when WebSocket connects)
    #[instrument(skip(self))]
    pub async fn set_online(&self, user_id: Snowflake) -> ServiceResult<()> {
        self.update_presence(user_id, UserStatus::Online).await
    }

    /// Set user offline (called when WebSocket disconnects)
    #[instrument(skip(self))]
    pub async fn set_offline(&self, user_id: Snowflake) -> ServiceResult<()> {
        // Remove presence from cache
        self.ctx
            .presence_store()
            .remove_presence(user_id)
            .await
            .map_err(|e| ServiceError::internal(format!("Failed to remove presence: {e}")))?;

        info!(user_id = %user_id, "User went offline");

        // Verify user exists for event data
        if let Some(user) = self.ctx.user_repo().find_by_id(user_id).await? {
            // Publish PRESENCE_UPDATE event to all guilds
            let guilds = self.ctx.guild_repo().find_by_user(user_id).await?;

            for guild in guilds {
                let event = PubSubEvent::new(
                    "PRESENCE_UPDATE",
                    json!({
                        "user": {
                            "id": user.id.to_string(),
                            "username": user.username,
                            "discriminator": user.discriminator,
                            "avatar": user.avatar
                        },
                        "guild_id": guild.id.to_string(),
                        "status": "offline",
                        "activities": []
                    }),
                );
                self.ctx
                    .publisher()
                    .publish(&PubSubChannel::guild(guild.id), &event)
                    .await
                    .ok();
            }
        }

        Ok(())
    }

    /// Check if a user is online
    #[instrument(skip(self))]
    pub async fn is_online(&self, user_id: Snowflake) -> ServiceResult<bool> {
        let presence_data = self
            .ctx
            .presence_store()
            .get_presence(user_id)
            .await
            .map_err(|e| ServiceError::internal(format!("Failed to get presence: {e}")))?;

        Ok(presence_data.is_some_and(|d| d.status != UserStatus::Offline))
    }

    /// Get online member count for a guild
    #[instrument(skip(self))]
    pub async fn get_online_count(&self, guild_id: Snowflake) -> ServiceResult<i64> {
        let count = self
            .ctx
            .presence_store()
            .get_guild_online_count(guild_id)
            .await
            .map_err(|e| ServiceError::internal(format!("Failed to get online count: {e}")))?;

        Ok(count as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status_display() {
        assert_eq!(UserStatus::Online.to_string(), "online");
        assert_eq!(UserStatus::Idle.to_string(), "idle");
        assert_eq!(UserStatus::Dnd.to_string(), "dnd");
        assert_eq!(UserStatus::Offline.to_string(), "offline");
    }
}
