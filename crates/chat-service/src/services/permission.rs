//! Permission service
//!
//! Handles permission checking and computation for guild members.

use chat_core::entities::Channel;
use chat_core::Permissions;
use chat_core::Snowflake;
use tracing::{debug, instrument};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};

/// Permission service for access control
pub struct PermissionService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> PermissionService<'a> {
    /// Create a new PermissionService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Check if a user has a specific permission in a guild
    #[instrument(skip(self))]
    pub async fn check_permission(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        permission: Permissions,
    ) -> ServiceResult<bool> {
        let permissions = self.get_member_permissions(guild_id, user_id).await?;
        Ok(permissions.has(permission))
    }

    /// Check if a user has a specific permission in a channel
    #[instrument(skip(self))]
    pub async fn check_channel_permission(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        permission: Permissions,
    ) -> ServiceResult<bool> {
        let channel = self
            .ctx
            .channel_repo()
            .find_by_id(channel_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Channel", channel_id.to_string()))?;

        // DM channels don't have guild-based permissions
        let guild_id = match channel.guild_id {
            Some(id) => id,
            None => return Ok(true),
        };
        let permissions = self.get_member_permissions(guild_id, user_id).await?;
        Ok(permissions.has(permission))
    }

    /// Check permission and return error if denied
    #[instrument(skip(self))]
    pub async fn require_permission(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        permission: Permissions,
    ) -> ServiceResult<()> {
        if !self.check_permission(guild_id, user_id, permission).await? {
            let perm_names = permission.list().join(", ");
            return Err(ServiceError::permission_denied(perm_names));
        }
        Ok(())
    }

    /// Check channel permission and return error if denied
    #[instrument(skip(self))]
    pub async fn require_channel_permission(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
        permission: Permissions,
    ) -> ServiceResult<()> {
        if !self
            .check_channel_permission(channel_id, user_id, permission)
            .await?
        {
            let perm_names = permission.list().join(", ");
            return Err(ServiceError::permission_denied(perm_names));
        }
        Ok(())
    }

    /// Get all permissions for a member in a guild
    #[instrument(skip(self))]
    pub async fn get_member_permissions(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Permissions> {
        // Get the guild to check if user is owner
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Guild owners have all permissions
        if guild.owner_id == user_id {
            debug!(user_id = %user_id, guild_id = %guild_id, "User is guild owner, granting all permissions");
            return Ok(Permissions::ALL);
        }

        // Check if user is a member
        let member = self
            .ctx
            .member_repo()
            .find(guild_id, user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Member", format!("{}/{}", guild_id, user_id)))?;

        // Get all roles for the member
        let mut permissions = Permissions::empty();

        // Always include @everyone role
        if let Some(everyone_role) = self.ctx.role_repo().find_everyone(guild_id).await? {
            permissions |= everyone_role.permissions;
        }

        // Add permissions from member's roles
        for role_id in &member.role_ids {
            if let Some(role) = self.ctx.role_repo().find_by_id(*role_id).await? {
                permissions |= role.permissions;
            }
        }

        debug!(
            user_id = %user_id,
            guild_id = %guild_id,
            permissions = %permissions,
            "Computed member permissions"
        );

        Ok(permissions)
    }

    /// Compute permissions for a specific channel (with overwrites)
    ///
    /// For MVP, we don't implement channel permission overwrites.
    /// This returns the member's guild permissions.
    #[instrument(skip(self))]
    pub async fn compute_channel_permissions(
        &self,
        channel: &Channel,
        user_id: Snowflake,
    ) -> ServiceResult<Permissions> {
        match channel.guild_id {
            Some(guild_id) => self.get_member_permissions(guild_id, user_id).await,
            None => {
                // DM channels - full permissions for participants
                Ok(Permissions::DEFAULT)
            }
        }
    }

    /// Check if user is guild owner
    #[instrument(skip(self))]
    pub async fn is_guild_owner(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<bool> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        Ok(guild.owner_id == user_id)
    }

    /// Check if user is guild member
    #[instrument(skip(self))]
    pub async fn is_guild_member(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<bool> {
        self.ctx.member_repo().is_member(guild_id, user_id).await.map_err(ServiceError::from)
    }

    /// Check if actor can manage target member (for kicks, bans, role changes)
    ///
    /// Rules:
    /// 1. Guild owner can manage anyone
    /// 2. Cannot manage the guild owner
    /// 3. Can only manage members with lower highest role
    #[instrument(skip(self))]
    pub async fn can_manage_member(
        &self,
        guild_id: Snowflake,
        actor_id: Snowflake,
        target_id: Snowflake,
    ) -> ServiceResult<bool> {
        // Same user cannot manage themselves (for some operations)
        if actor_id == target_id {
            return Ok(false);
        }

        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Owner can manage anyone
        if guild.owner_id == actor_id {
            return Ok(true);
        }

        // Cannot manage the owner
        if guild.owner_id == target_id {
            return Ok(false);
        }

        // Get highest role positions
        let actor_highest = self.get_highest_role_position(guild_id, actor_id).await?;
        let target_highest = self.get_highest_role_position(guild_id, target_id).await?;

        Ok(actor_highest > target_highest)
    }

    /// Check if actor can assign a specific role
    ///
    /// Can only assign roles lower than own highest role
    #[instrument(skip(self))]
    pub async fn can_assign_role(
        &self,
        guild_id: Snowflake,
        actor_id: Snowflake,
        role_id: Snowflake,
    ) -> ServiceResult<bool> {
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Owner can assign any role
        if guild.owner_id == actor_id {
            return Ok(true);
        }

        // Get role to be assigned
        let role = self
            .ctx
            .role_repo()
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Role", role_id.to_string()))?;

        // Cannot assign @everyone role
        if role.is_everyone {
            return Ok(false);
        }

        // Get actor's highest role position
        let actor_highest = self.get_highest_role_position(guild_id, actor_id).await?;

        Ok(actor_highest > role.position)
    }

    /// Get the highest role position for a member
    async fn get_highest_role_position(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<i32> {
        let member = self
            .ctx
            .member_repo()
            .find(guild_id, user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Member", format!("{}/{}", guild_id, user_id)))?;

        let mut highest_position = 0;

        for role_id in &member.role_ids {
            if let Some(role) = self.ctx.role_repo().find_by_id(*role_id).await? {
                if role.position > highest_position {
                    highest_position = role.position;
                }
            }
        }

        Ok(highest_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_flags() {
        let perms = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(perms.has(Permissions::VIEW_CHANNEL));
        assert!(perms.has(Permissions::SEND_MESSAGES));
        assert!(!perms.has(Permissions::MANAGE_GUILD));
    }

    #[test]
    fn test_administrator_bypass() {
        let perms = Permissions::ADMINISTRATOR;
        assert!(perms.has(Permissions::VIEW_CHANNEL));
        assert!(perms.has(Permissions::MANAGE_GUILD));
        assert!(perms.has(Permissions::BAN_MEMBERS));
    }
}
