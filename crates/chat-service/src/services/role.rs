//! Role service
//!
//! Handles role creation, management, and permission assignment.

use chat_cache::{PubSubChannel, PubSubEvent};
use chat_core::entities::Role;
use chat_core::{Permissions, Snowflake};
use chrono::Utc;
use serde_json::json;
use tracing::{info, instrument};

use crate::dto::{CreateRoleRequest, RoleResponse, RolePosition, UpdateRoleRequest};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};
use super::permission::PermissionService;

/// Role service
pub struct RoleService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> RoleService<'a> {
    /// Create a new RoleService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Create a new role
    #[instrument(skip(self, request))]
    pub async fn create_role(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        request: CreateRoleRequest,
    ) -> ServiceResult<RoleResponse> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_ROLES)
            .await?;

        // Verify guild exists
        let _guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        // Parse permissions if provided
        let permissions = if let Some(ref perms_str) = request.permissions {
            Permissions::parse(perms_str)
                .map_err(|_| ServiceError::validation("Invalid permissions format"))?
        } else {
            Permissions::empty()
        };

        // Get position (default to 1, above @everyone)
        let existing_roles = self.ctx.role_repo().find_by_guild(guild_id).await?;
        let max_position = existing_roles.iter().map(|r| r.position).max().unwrap_or(0);
        let position = max_position + 1;

        // Get actor's highest role to ensure they can only create lower roles
        let actor_highest = self.get_actor_highest_position(guild_id, user_id).await?;
        if position >= actor_highest && !permission_service.is_guild_owner(guild_id, user_id).await? {
            // Place role just below actor's highest
            // This is simplified - Discord has more complex position handling
        }

        let role_id = self.ctx.generate_id();
        let now = Utc::now();

        let role = Role {
            id: role_id,
            guild_id,
            name: request.name,
            color: request.color,
            hoist: request.hoist,
            position,
            permissions,
            mentionable: request.mentionable,
            is_everyone: false,
            created_at: now,
            updated_at: now,
        };

        self.ctx.role_repo().create(&role).await?;

        info!(role_id = %role_id, guild_id = %guild_id, "Role created");

        // Publish GUILD_ROLE_CREATE event
        self.publish_role_event("GUILD_ROLE_CREATE", guild_id, &role).await;

        Ok(RoleResponse::from(&role))
    }

    /// Get role by ID
    #[instrument(skip(self))]
    pub async fn get_role(&self, role_id: Snowflake) -> ServiceResult<RoleResponse> {
        let role = self
            .ctx
            .role_repo()
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Role", role_id.to_string()))?;

        Ok(RoleResponse::from(&role))
    }

    /// Update role
    #[instrument(skip(self, request))]
    pub async fn update_role(
        &self,
        guild_id: Snowflake,
        role_id: Snowflake,
        user_id: Snowflake,
        request: UpdateRoleRequest,
    ) -> ServiceResult<RoleResponse> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_ROLES)
            .await?;

        let mut role = self
            .ctx
            .role_repo()
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Role", role_id.to_string()))?;

        // Verify role belongs to guild
        if role.guild_id != guild_id {
            return Err(ServiceError::not_found("Role", role_id.to_string()));
        }

        // Cannot edit @everyone role's position or delete it
        if role.is_everyone
            && request.position.is_some() {
                return Err(ServiceError::validation("Cannot change @everyone role position"));
            }

        // Check hierarchy - can only edit roles below own highest role
        let actor_highest = self.get_actor_highest_position(guild_id, user_id).await?;
        if role.position >= actor_highest
            && !permission_service.is_guild_owner(guild_id, user_id).await?
        {
            return Err(ServiceError::permission_denied("Cannot edit this role"));
        }

        let mut changed = false;

        // Update name
        if let Some(name) = request.name {
            role.name = name;
            changed = true;
        }

        // Update color
        if let Some(color) = request.color {
            role.color = color;
            changed = true;
        }

        // Update hoist
        if let Some(hoist) = request.hoist {
            role.hoist = hoist;
            changed = true;
        }

        // Update permissions
        if let Some(ref perms_str) = request.permissions {
            let permissions = Permissions::parse(perms_str)
                .map_err(|_| ServiceError::validation("Invalid permissions format"))?;
            role.permissions = permissions;
            changed = true;
        }

        // Update mentionable
        if let Some(mentionable) = request.mentionable {
            role.mentionable = mentionable;
            changed = true;
        }

        // Update position
        if let Some(position) = request.position {
            if !role.is_everyone {
                // Ensure position is valid (0 is reserved for @everyone)
                if position < 1 {
                    return Err(ServiceError::validation("Position must be >= 1"));
                }
                role.position = position;
                changed = true;
            }
        }

        if changed {
            role.updated_at = Utc::now();
            self.ctx.role_repo().update(&role).await?;

            info!(role_id = %role_id, "Role updated");

            // Publish GUILD_ROLE_UPDATE event
            self.publish_role_event("GUILD_ROLE_UPDATE", guild_id, &role).await;
        }

        Ok(RoleResponse::from(&role))
    }

    /// Delete role
    #[instrument(skip(self))]
    pub async fn delete_role(
        &self,
        guild_id: Snowflake,
        role_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<()> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_ROLES)
            .await?;

        let role = self
            .ctx
            .role_repo()
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Role", role_id.to_string()))?;

        // Verify role belongs to guild
        if role.guild_id != guild_id {
            return Err(ServiceError::not_found("Role", role_id.to_string()));
        }

        // Cannot delete @everyone role
        if role.is_everyone {
            return Err(ServiceError::validation("Cannot delete @everyone role"));
        }

        // Check hierarchy
        let actor_highest = self.get_actor_highest_position(guild_id, user_id).await?;
        if role.position >= actor_highest
            && !permission_service.is_guild_owner(guild_id, user_id).await?
        {
            return Err(ServiceError::permission_denied("Cannot delete this role"));
        }

        self.ctx.role_repo().delete(role_id).await?;

        info!(role_id = %role_id, guild_id = %guild_id, "Role deleted");

        // Publish GUILD_ROLE_DELETE event
        let event = PubSubEvent::new(
            "GUILD_ROLE_DELETE",
            json!({
                "guild_id": guild_id.to_string(),
                "role_id": role_id.to_string()
            }),
        );
        self.ctx
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();

        Ok(())
    }

    /// Get all roles in a guild
    #[instrument(skip(self))]
    pub async fn get_guild_roles(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<Vec<RoleResponse>> {
        // Verify user is a member
        let permission_service = PermissionService::new(self.ctx);
        if !permission_service.is_guild_member(guild_id, user_id).await? {
            return Err(ServiceError::not_found("Guild", guild_id.to_string()));
        }

        let roles = self.ctx.role_repo().find_by_guild(guild_id).await?;
        Ok(roles.into_iter().map(RoleResponse::from).collect())
    }

    /// Update role positions in bulk
    #[instrument(skip(self, positions))]
    pub async fn update_role_positions(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        positions: Vec<RolePosition>,
    ) -> ServiceResult<Vec<RoleResponse>> {
        let permission_service = PermissionService::new(self.ctx);

        // Check MANAGE_ROLES permission
        permission_service
            .require_permission(guild_id, user_id, Permissions::MANAGE_ROLES)
            .await?;

        // Parse and validate positions
        let mut updates: Vec<(Snowflake, i32)> = Vec::with_capacity(positions.len());

        for pos in positions {
            let role_id = pos.id.parse::<i64>()
                .map(Snowflake::new)
                .map_err(|_| ServiceError::validation("Invalid role ID format"))?;

            // Verify role exists and belongs to guild
            let role = self
                .ctx
                .role_repo()
                .find_by_id(role_id)
                .await?
                .ok_or_else(|| ServiceError::not_found("Role", pos.id.clone()))?;

            if role.guild_id != guild_id {
                return Err(ServiceError::not_found("Role", pos.id.clone()));
            }

            // Cannot move @everyone role
            if role.is_everyone {
                continue;
            }

            updates.push((role_id, pos.position));
        }

        // Update positions
        self.ctx
            .role_repo()
            .update_positions(guild_id, &updates)
            .await?;

        info!(guild_id = %guild_id, count = updates.len(), "Role positions updated");

        // Return updated roles
        self.get_guild_roles(guild_id, user_id).await
    }

    /// Get actor's highest role position
    async fn get_actor_highest_position(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> ServiceResult<i32> {
        // Guild owner has implicit highest position
        let guild = self
            .ctx
            .guild_repo()
            .find_by_id(guild_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("Guild", guild_id.to_string()))?;

        if guild.owner_id == user_id {
            return Ok(i32::MAX);
        }

        // Get member's roles
        let member = self.ctx.member_repo().find(guild_id, user_id).await?;

        let mut highest = 0;
        if let Some(member) = member {
            for role_id in member.role_ids {
                if let Some(role) = self.ctx.role_repo().find_by_id(role_id).await? {
                    if role.position > highest {
                        highest = role.position;
                    }
                }
            }
        }

        Ok(highest)
    }

    /// Helper to publish role events
    async fn publish_role_event(&self, event_type: &str, guild_id: Snowflake, role: &Role) {
        let data = json!({
            "guild_id": guild_id.to_string(),
            "role": {
                "id": role.id.to_string(),
                "name": role.name,
                "color": role.color,
                "hoist": role.hoist,
                "position": role.position,
                "permissions": role.permissions.to_string(),
                "mentionable": role.mentionable
            }
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
