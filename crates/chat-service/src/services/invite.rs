//! Invite service
//!
//! Handles guild invite creation, validation, and usage.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::Invite;
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use rand::Rng;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{CreateInviteRequest, InviteResponse, InviteWithDetails};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Invite service
pub struct InviteService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> InviteService<'a> {
    /// Create a new InviteService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create a new invite for a channel
    #[instrument(skip(self, request))]
    pub async fn create_invite(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        request: CreateInviteRequest,
    ) -> ServiceResult<InviteResponse> {
        // Get channel
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // Invites only for guild channels
        let guild_id = channel
            .guild_id
            .ok_or_else(|| ServiceError::validation("Cannot create invite for DM channels"))?;

        // Check permission - for MVP, members who can send messages can create invites
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::SEND_MESSAGES)
            .await?;

        // Get guild and inviter for the response
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        let inviter = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        // Generate invite code
        let code = self.generate_invite_code();

        // Fields from request are already non-optional with defaults
        let max_age = request.max_age;
        let max_uses = request.max_uses;
        let temporary = request.temporary;

        let invite = Invite::new(
            code.clone(),
            guild_id,
            channel_id,
            user_id,
        )
        .with_max_uses(max_uses)
        .with_expiration(max_age)
        .with_temporary(temporary);

        self.ctx.invite_repo().create(&invite).await?;

        info!(
            code = %code,
            guild_id = %guild_id,
            channel_id = %channel_id,
            inviter_id = %user_id,
            "Invite created"
        );

        // Get member count
        let member_count = self.ctx.guild_repo().member_count(guild_id).await?;

        Ok(InviteResponse::from(InviteWithDetails {
            invite,
            guild,
            channel,
            inviter,
            member_count,
        }))
    }

    /// Get invite by code
    #[instrument(skip(self))]
    pub async fn get_invite(&self, code: &str) -> ServiceResult<InviteResponse> {
        let invite = self
            .ctx
            .invite_repo()
            .find_by_code(code)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", code.to_string()))?;

        // Check if expired
        if let Some(expires_at) = invite.expires_at {
            if expires_at < Utc::now() {
                return Err(ServiceError::not_found("Invite", code.to_string()));
            }
        }

        // Check if max uses reached
        if let Some(max) = invite.max_uses {
            if max > 0 && invite.uses >= max {
                return Err(ServiceError::not_found("Invite", code.to_string()));
            }
        }

        // Get related entities
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(invite.guild_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Invite guild not found"))?;

        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(invite.channel_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Invite channel not found"))?;

        let inviter = self
            .ctx
            .user_repo()
            .find_by_id(invite.inviter_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Invite creator not found"))?;

        let member_count = self.ctx.guild_repo().member_count(guild.id).await?;

        Ok(InviteResponse::from(InviteWithDetails {
            invite,
            guild,
            channel,
            inviter,
            member_count,
        }))
    }

    /// Use invite to join guild
    #[instrument(skip(self))]
    pub async fn use_invite(&self, code: &str, user_id: Snowflake) -> ServiceResult<InviteResponse> {
        let invite = self
            .ctx
            .invite_repo()
            .find_by_code(code)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", code.to_string()))?;

        // Check if expired
        if let Some(expires_at) = invite.expires_at {
            if expires_at < Utc::now() {
                // Delete expired invite
                self.ctx.invite_repo().delete(code).await?;
                return Err(ServiceError::validation("Invite has expired"));
            }
        }

        // Check if max uses reached
        if let Some(max) = invite.max_uses {
            if max > 0 && invite.uses >= max {
                return Err(ServiceError::validation("Invite has reached maximum uses"));
            }
        }

        // Check if user is banned
        if self
            .ctx
            .ban_repo()
            .is_banned(invite.guild_id, user_id)
            .await?
        {
            return Err(ServiceError::conflict("You are banned from this guild"));
        }

        // Check if already a member
        if self
            .ctx
            .member_repo()
            .is_member(invite.guild_id, user_id)
            .await?
        {
            return Err(ServiceError::conflict("Already a member of this guild"));
        }

        // Add user as member
        let now = Utc::now();
        let member = chat_core::entities::GuildMember {
            guild_id: invite.guild_id,
            user_id,
            nickname: None,
            role_ids: vec![],
            joined_at: now,
            updated_at: now,
        };

        self.ctx.member_repo().create(&member).await?;

        // Increment invite uses
        self.ctx.invite_repo().increment_uses(code).await?;

        info!(
            code = %code,
            user_id = %user_id,
            guild_id = %invite.guild_id,
            "Invite used"
        );

        // Publish GUILD_MEMBER_ADD event
        if let Some(user) = self.ctx.user_repo().find_by_id(user_id).await? {
            let event = PubSubEvent::new(
                "GUILD_MEMBER_ADD",
                json!({
                    "guild_id": invite.guild_id.to_string(),
                    "user": {
                        "id": user.id.to_string(),
                        "username": user.username,
                        "discriminator": user.discriminator,
                        "avatar": user.avatar
                    },
                    "nick": null,
                    "roles": [],
                    "joined_at": now.to_rfc3339()
                }),
            );
            self.ctx
                .publisher()
                .publish(&PubSubChannel::guild(invite.guild_id), &event)
                .await
                .ok();
        }

        // Get updated invite and related entities
        let updated_invite = self
            .ctx
            .invite_repo()
            .find_by_code(code)
            .await?
            .unwrap_or(invite);

        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(updated_invite.guild_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Guild not found"))?;

        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(updated_invite.channel_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Channel not found"))?;

        let inviter = self
            .ctx
            .user_repo()
            .find_by_id(updated_invite.inviter_id)
            .await?
            .ok_or_else(|| ServiceError::internal("Inviter not found"))?;

        let member_count = self.ctx.guild_repo().member_count(guild.id).await?;

        Ok(InviteResponse::from(InviteWithDetails {
            invite: updated_invite,
            guild,
            channel,
            inviter,
            member_count,
        }))
    }

    /// Delete invite
    #[instrument(skip(self))]
    pub async fn delete_invite(&self, code: &str, user_id: Snowflake) -> ServiceResult<()> {
        let invite = self
            .ctx
            .invite_repo()
            .find_by_code(code)
            .await?
            .ok_or_else(|| ServiceError::not_found("Invite", code.to_string()))?;

        // Check permission
        // Can delete if:
        // 1. User is the inviter
        // 2. User has MANAGE_CHANNELS permission
        let permission_service = PermissionService::new(self.ctx);
        let can_delete = invite.inviter_id == user_id
            || permission_service
                .check_permission(invite.guild_id, user_id, Permissions::MANAGE_CHANNELS)
                .await?;

        if !can_delete {
            return Err(ServiceError::permission_denied("MANAGE_CHANNELS"));
        }

        self.ctx.invite_repo().delete(code).await?;

        info!(code = %code, user_id = %user_id, "Invite deleted");

        // Publish INVITE_DELETE event
        let event = PubSubEvent::new(
            "INVITE_DELETE",
            json!({
                "channel_id": invite.channel_id.to_string(),
                "guild_id": invite.guild_id.to_string(),
                "code": code
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(invite.guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get all invites for a guild
    #[instrument(skip(self))]
    pub async fn get_guild_invites(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<InviteResponse>> {
        // Check MANAGE_GUILD permission to view all invites
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_GUILD)
            .await?;

        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        let member_count = self.ctx.guild_repo().member_count(guild_id).await?;

        let invites = self.ctx.invite_repo().find_by_guild(guild_id).await?;

        let mut responses = Vec::with_capacity(invites.len());

        for invite in invites {
            // Skip expired invites
            if let Some(expires_at) = invite.expires_at {
                if expires_at < Utc::now() {
                    continue;
                }
            }

            let channel = match self.ctx.channel_repo().find_by_id(invite.channel_id).await? {
                Some(c) => c,
                None => continue, // Skip if channel no longer exists
            };

            let inviter = match self.ctx.user_repo().find_by_id(invite.inviter_id).await? {
                Some(u) => u,
                None => continue, // Skip if inviter no longer exists
            };

            responses.push(InviteResponse::from(InviteWithDetails {
                invite,
                guild: guild.clone(),
                channel,
                inviter,
                member_count,
            }));
        }

        Ok(responses)
    }

    /// Get all invites for a channel
    #[instrument(skip(self))]
    pub async fn get_channel_invites(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<InviteResponse>> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        let guild_id = channel
            .guild_id
            .ok_or_else(|| ServiceError::validation("DM channels don't have invites"))?;

        // Check MANAGE_CHANNELS permission
        let permission_service = PermissionService::new(self.ctx);
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_CHANNELS)
            .await?;

        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        let member_count = self.ctx.guild_repo().member_count(guild_id).await?;

        let invites = self.ctx.invite_repo().find_by_channel(channel_id).await?;

        let mut responses = Vec::with_capacity(invites.len());

        for invite in invites {
            // Skip expired invites
            if let Some(expires_at) = invite.expires_at {
                if expires_at < Utc::now() {
                    continue;
                }
            }

            let inviter = match self.ctx.user_repo().find_by_id(invite.inviter_id).await? {
                Some(u) => u,
                None => continue, // Skip if inviter no longer exists
            };

            responses.push(InviteResponse::from(InviteWithDetails {
                invite,
                guild: guild.clone(),
                channel: channel.clone(),
                inviter,
                member_count,
            }));
        }

        Ok(responses)
    }

    /// Generate a random invite code
    fn generate_invite_code(&self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const CODE_LENGTH: usize = 8;

        let mut rng = rand::thread_rng();
        (0..CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
