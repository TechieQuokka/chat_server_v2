//! User service
//!
//! Handles user profile operations.

use chat_core::entities::User;
use chat_core::Snowflake;
use chrono::Utc;
use tracing::{info, instrument};

use crate::dto::{CurrentUserResponse, PublicUserResponse, UpdateUserRequest, UserResponse};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};

/// User service
pub struct UserService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> UserService<'a> {
    /// Create a new UserService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Get user by ID (public profile)
    #[instrument(skip(self))]
    pub async fn get_user(&self, user_id: Snowflake) -> ServiceResult<PublicUserResponse> {
        let user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        Ok(PublicUserResponse::from(&user))
    }

    /// Get current authenticated user (full profile)
    #[instrument(skip(self))]
    pub async fn get_current_user(&self, user_id: Snowflake) -> ServiceResult<CurrentUserResponse> {
        let user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        Ok(CurrentUserResponse::from(&user))
    }

    /// Get user entity by ID
    #[instrument(skip(self))]
    pub async fn get_user_entity(&self, user_id: Snowflake) -> ServiceResult<User> {
        self.ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))
    }

    /// Update current user
    #[instrument(skip(self, request))]
    pub async fn update_user(
        &self,
        user_id: Snowflake,
        request: UpdateUserRequest,
    ) -> ServiceResult<CurrentUserResponse> {
        let mut user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        let mut changed = false;

        // Update username if provided
        if let Some(username) = request.username {
            if username != user.username {
                // Generate new discriminator for the new username
                let discriminator = self
                    .ctx
                    .user_repo()
                    .next_discriminator(&username)
                    .await?;
                user.username = username;
                user.discriminator = discriminator;
                changed = true;
            }
        }

        // Update avatar if provided
        if let Some(avatar) = request.avatar {
            user.avatar = Some(avatar);
            changed = true;
        }

        if changed {
            user.updated_at = Utc::now();
            self.ctx.user_repo().update(&user).await?;
            info!(user_id = %user_id, "User profile updated");
        }

        Ok(CurrentUserResponse::from(&user))
    }

    /// Delete user account (soft delete)
    #[instrument(skip(self))]
    pub async fn delete_user(&self, user_id: Snowflake) -> ServiceResult<()> {
        // Verify user exists
        let _user = self
            .ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))?;

        self.ctx.user_repo().delete(user_id).await?;
        info!(user_id = %user_id, "User account deleted");

        Ok(())
    }

    /// Find user by username and discriminator
    #[instrument(skip(self))]
    pub async fn find_by_tag(
        &self,
        username: &str,
        discriminator: &str,
    ) -> ServiceResult<Option<PublicUserResponse>> {
        let user = self
            .ctx
            .user_repo()
            .find_by_tag(username, discriminator)
            .await?;

        Ok(user.map(|u| PublicUserResponse::from(&u)))
    }

    /// Get multiple users by ID
    #[instrument(skip(self))]
    pub async fn get_users(&self, user_ids: &[Snowflake]) -> ServiceResult<Vec<UserResponse>> {
        let mut users = Vec::with_capacity(user_ids.len());

        for &user_id in user_ids {
            if let Some(user) = self.ctx.user_repo().find_by_id(user_id).await? {
                users.push(UserResponse::from(&user));
            }
        }

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
