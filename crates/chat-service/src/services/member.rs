//! Member service
//!
//! Handles guild member management including adding, removing, and updating members.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::GuildMember;
use chat_core::traits::Ban;
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{BanResponse, CreateBanRequest, MemberResponse, MemberWithUser, UpdateMemberRequest, UserResponse};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Member service
pub struct MemberService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> MemberService<'a> {
    /// Create a new MemberService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Add member to guild (via invite)
    #[instrument(skip(self))]
    pub async fn add_member(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<MemberResponse> {
        // Check if user is banned
        if self.ctx.ban_repo().is_banned(guild_id, user_id).await? {
            return Err(ServiceError::conflict("User is banned from this guild"));
        }

        // Check if already a member
        if self.ctx.member_repo().is_member(guild_id, user_id).await? {
            return Err(ServiceError::conflict("User is already a member"));
        }

        // Verify guild exists
        let _guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Get user
        let user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        let now = Utc::now();
        let member = GuildMember {
            guild_id,
            user_id,
            nickname: None,
            role_ids: vec![],
            joined_at: now,
            updated_at: now,
        };

        self.ctx.member_repo().create(&member).await?;

        info!(guild_id = %guild_id, user_id = %user_id, "Member added to guild");

        // Publish GUILD_MEMBER_ADD event
        self.publish_member_event("GUILD_MEMBER_ADD", guild_id, &member, &user)
            .await;

        Ok(MemberResponse::from(MemberWithUser { member, user }))
    }

    /// Get member by guild and user ID
    #[instrument(skip(self))]
    pub async fn get_member(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<MemberResponse> {
        let member = self
            .ctx
            .member_repo()
            .find(guild_id, user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Member", format!("{guild_id}/{user_id}")))?;

        let user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        Ok(MemberResponse::from(MemberWithUser { member, user }))
    }

    /// Update member (nickname, roles)
    #[instrument(skip(self, request))]
    pub async fn update_member(
        &self,
        guild_id: Snowflake,
        target_id: Snowflake,
        actor_id: Snowflake,
        request: UpdateMemberRequest,
    ) -> ServiceResult<MemberResponse> {
        // Check if actor can manage this member
        let permission_service = PermissionService::new(self.ctx);

        // Self-edit only allows nickname change
        let is_self = actor_id == target_id;

        if !is_self {
            // Need MANAGE_ROLES to change roles
            if request.roles.is_some() {
                permission_service
                    .require_permission(guild_id, actor_id, Permissions::MANAGE_ROLES)
                    .await?;
            }

            // Need to have higher role to manage
            if !permission_service
                .can_manage_member(guild_id, actor_id, target_id)
                .await?
            {
                return Err(ServiceError::permission_denied("Cannot manage this member"));
            }
        }

        let mut member = self
            .ctx
            .member_repo()
            .find(guild_id, target_id)
            .await?
            .ok_or_else(|| {
                ServiceError::not_found("Member", format!("{guild_id}/{target_id}"))
            })?;

        let user = self
            .ctx
            .user_repo()
            .find_by_id(target_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", target_id.to_string()))?;

        let mut changed = false;

        // Update nickname
        if let Some(nickname) = request.nickname {
            member.nickname = if nickname.is_empty() {
                None
            } else {
                Some(nickname)
            };
            changed = true;
        }

        // Update roles (only for non-self edits with permission)
        if let Some(role_ids) = request.roles {
            if !is_self {
                // Parse role IDs
                let new_roles: Result<Vec<_>, _> = role_ids
                    .iter()
                    .map(|s| {
                        s.parse::<i64>()
                            .map(Snowflake::new)
                            .map_err(|_| ServiceError::validation("Invalid role ID format"))
                    })
                    .collect();
                let new_roles = new_roles?;

                // Verify actor can assign each role
                for &role_id in &new_roles {
                    if !permission_service
                        .can_assign_role(guild_id, actor_id, role_id)
                        .await?
                    {
                        return Err(ServiceError::permission_denied(format!(
                            "Cannot assign role {role_id}"
                        )));
                    }
                }

                member.role_ids = new_roles;
                changed = true;
            }
        }

        if changed {
            member.updated_at = Utc::now();
            self.ctx.member_repo().update(&member).await?;

            info!(guild_id = %guild_id, user_id = %target_id, "Member updated");

            // Publish GUILD_MEMBER_UPDATE event
            self.publish_member_event("GUILD_MEMBER_UPDATE", guild_id, &member, &user)
                .await;
        }

        Ok(MemberResponse::from(MemberWithUser { member, user }))
    }

    /// Remove member from guild (kick)
    #[instrument(skip(self))]
    pub async fn remove_member(
        &self,
        guild_id: Snowflake,
        target_id: Snowflake,
        actor_id: Snowflake,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check KICK_MEMBERS permission
        permission_service
            .require_permission(guild_id, actor_id, Permissions::KICK_MEMBERS)
            .await?;

        // Check hierarchy
        if !permission_service
            .can_manage_member(guild_id, actor_id, target_id)
            .await?
        {
            return Err(ServiceError::permission_denied("Cannot kick this member"));
        }

        // Cannot kick the owner
        if permission_service.is_guild_owner(guild_id, target_id).await? {
            return Err(ServiceError::conflict("Cannot kick the guild owner"));
        }

        self.ctx.member_repo().delete(guild_id, target_id).await?;

        info!(guild_id = %guild_id, user_id = %target_id, actor_id = %actor_id, "Member kicked");

        // Publish GUILD_MEMBER_REMOVE event
        let event = PubSubEvent::new(
            "GUILD_MEMBER_REMOVE",
            json!({
                "guild_id": guild_id.to_string(),
                "user_id": target_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get all members in a guild (paginated)
    #[instrument(skip(self))]
    pub async fn get_guild_members(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        limit: i64,
        after: Option<Snowflake>,
    ) -> ServiceResult<Vec<MemberResponse>> {
        // Verify user is a member
        let permission_service = PermissionService::new(self.ctx);
        if !permission_service.is_guild_member(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Guild", guild_id.to_string()));
        }

        let members = self
            .ctx
            .member_repo()
            .find_by_guild(guild_id, limit.min(1000), after)
            .await?;

        let mut responses = Vec::with_capacity(members.len());

        for member in members {
            if let Some(user) = self.ctx.user_repo().find_by_id(member.user_id).await? {
                responses.push(MemberResponse::from(MemberWithUser { member, user }));
            }
        }

        Ok(responses)
    }

    /// Add role to member
    #[instrument(skip(self))]
    pub async fn add_role(
        &self,
        guild_id: Snowflake,
        target_id: Snowflake,
        role_id: Snowflake,
        actor_id: Snowflake,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, actor_id, Permissions::MANAGE_ROLES)
            .await?;

        // Check can assign role
        if !permission_service
            .can_assign_role(guild_id, actor_id, role_id)
            .await?
        {
            return Err(ServiceError::permission_denied("Cannot assign this role"));
        }

        self.ctx
            .member_repo()
            .add_role(guild_id, target_id, role_id)
            .await?;

        info!(guild_id = %guild_id, user_id = %target_id, role_id = %role_id, "Role added to member");

        Ok(())
    }

    /// Remove role from member
    #[instrument(skip(self))]
    pub async fn remove_role(
        &self,
        guild_id: Snowflake,
        target_id: Snowflake,
        role_id: Snowflake,
        actor_id: Snowflake,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, actor_id, Permissions::MANAGE_ROLES)
            .await?;

        // Check can assign role (same rules for removal)
        if !permission_service
            .can_assign_role(guild_id, actor_id, role_id)
            .await?
        {
            return Err(ServiceError::permission_denied("Cannot remove this role"));
        }

        self.ctx
            .member_repo()
            .remove_role(guild_id, target_id, role_id)
            .await?;

        info!(guild_id = %guild_id, user_id = %target_id, role_id = %role_id, "Role removed from member");

        Ok(())
    }

    // ========================================================================
    // Ban operations
    // ========================================================================

    /// Ban a user from guild
    #[instrument(skip(self, request))]
    pub async fn ban_member(
        &self,
        guild_id: Snowflake,
        target_id: Snowflake,
        actor_id: Snowflake,
        request: CreateBanRequest,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check BAN_MEMBERS permission
        permission_service
            .require_permission(guild_id, actor_id, Permissions::BAN_MEMBERS)
            .await?;

        // Cannot ban the owner
        if permission_service.is_guild_owner(guild_id, target_id).await? {
            return Err(ServiceError::conflict("Cannot ban the guild owner"));
        }

        // Check hierarchy
        if self.ctx.member_repo().is_member(guild_id, target_id).await?
            && !permission_service
                .can_manage_member(guild_id, actor_id, target_id)
                .await?
            {
                return Err(ServiceError::permission_denied("Cannot ban this member"));
            }

        // Create ban record
        let ban = Ban {
            guild_id,
            user_id: target_id,
            reason: request.reason,
        };

        self.ctx.ban_repo().create(&ban).await?;

        // Remove member if they were one
        if self.ctx.member_repo().is_member(guild_id, target_id).await? {
            self.ctx.member_repo().delete(guild_id, target_id).await?;
        }

        info!(guild_id = %guild_id, user_id = %target_id, actor_id = %actor_id, "User banned");

        // Publish GUILD_BAN_ADD event
        let event = PubSubEvent::new(
            "GUILD_BAN_ADD",
            json!({
                "guild_id": guild_id.to_string(),
                "user_id": target_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Unban a user from guild
    #[instrument(skip(self))]
    pub async fn unban_member(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        actor_id: Snowflake,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check BAN_MEMBERS permission
        permission_service
            .require_permission(guild_id, actor_id, Permissions::BAN_MEMBERS)
            .await?;

        // Check if ban exists
        if !self.ctx.ban_repo().is_banned(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Ban", format!("{guild_id}/{user_id}")));
        }

        self.ctx.ban_repo().delete(guild_id, user_id).await?;

        info!(guild_id = %guild_id, user_id = %user_id, actor_id = %actor_id, "User unbanned");

        // Publish GUILD_BAN_REMOVE event
        let event = PubSubEvent::new(
            "GUILD_BAN_REMOVE",
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

    /// Get all bans in a guild
    #[instrument(skip(self))]
    pub async fn get_guild_bans(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<BanResponse>> {
        let permission_service = PermissionService::new(self.ctx);

        // Check BAN_MEMBERS permission
        permission_service
            .require_permission(guild_id, user_id, Permissions::BAN_MEMBERS)
            .await?;

        let bans = self.ctx.ban_repo().find_by_guild(guild_id).await?;

        let mut responses = Vec::with_capacity(bans.len());

        for ban in bans {
            if let Some(user) = self.ctx.user_repo().find_by_id(ban.user_id).await? {
                responses.push(BanResponse {
                    user: UserResponse::from(&user),
                    reason: ban.reason,
                });
            }
        }

        Ok(responses)
    }

    /// Helper to publish member events
    async fn publish_member_event(
        &self,
        event_type: &str,
        guild_id: Snowflake,
        member: &GuildMember,
        user: &chat_core::entities::User,
    ) {
        let data = json!({
            "guild_id": guild_id.to_string(),
            "user": {
                "id": user.id.to_string(),
                "username": user.username,
                "discriminator": user.discriminator,
                "avatar": user.avatar
            },
            "nick": member.nickname,
            "roles": member.role_ids.iter().map(std::string::ToString::to_string).collect::<Vec<_>>(),
            "joined_at": member.joined_at.to_rfc3339()
        });

        let event = PubSubEvent::new(event_type, data);
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
