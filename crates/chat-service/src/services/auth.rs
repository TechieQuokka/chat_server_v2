//! Authentication service
//!
//! Handles user registration, login, token refresh, and logout.

use chat_cache::RefreshTokenData;
use chat_common::auth::{hash_password, validate_password_strength, verify_password};
use uuid::Uuid;
use chat_core::entities::User;
use chat_core::Snowflake;
use chrono::Utc;
use tracing::{info, instrument, warn};

use crate::dto::{
    AuthResponse, CurrentUserResponse, LoginRequest, RefreshTokenRequest, RegisterRequest,
};

use super::context::ServiceContext;
use super::error::{ServiceError, ServiceResult};

/// Authentication service
pub struct AuthService<'a> {
    ctx: &'a ServiceContext,
}

impl<'a> AuthService<'a> {
    /// Create a new AuthService
    pub fn new(ctx: &'a ServiceContext) -> Self {
        Self { ctx }
    }

    /// Register a new user
    #[instrument(skip(self, request), fields(username = %request.username, email = %request.email))]
    pub async fn register(&self, request: RegisterRequest) -> ServiceResult<AuthResponse> {
        // Validate password strength before proceeding
        validate_password_strength(&request.password).map_err(ServiceError::from)?;

        // Check if email already exists
        if self.ctx.user_repo().email_exists(&request.email).await? {
            return Err(ServiceError::conflict("Email already registered"));
        }

        // Generate discriminator
        let discriminator = self
            .ctx
            .user_repo()
            .next_discriminator(&request.username)
            .await?;

        // Hash password
        let password_hash =
            hash_password(&request.password).map_err(|e| ServiceError::internal(e.to_string()))?;

        // Create user
        let user_id = self.ctx.generate_id();
        let now = Utc::now();

        let user = User {
            id: user_id,
            username: request.username,
            discriminator,
            email: request.email,
            avatar: None,
            bot: false,
            system: false,
            created_at: now,
            updated_at: now,
        };

        // Save to database
        self.ctx.user_repo().create(&user, &password_hash).await?;

        info!(user_id = %user_id, "User registered successfully");

        // Generate tokens
        let token_pair = self
            .ctx
            .jwt_service()
            .generate_token_pair(user_id)
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        // Store refresh token in Redis
        let session_id = Uuid::new_v4().to_string();
        let refresh_data = RefreshTokenData::new(user_id, session_id);
        self.ctx
            .refresh_token_store()
            .store(&token_pair.refresh_token, &refresh_data)
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        Ok(AuthResponse::new(
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.expires_in,
            CurrentUserResponse::from(&user),
        ))
    }

    /// Login with email and password
    #[instrument(skip(self, request), fields(email = %request.email))]
    pub async fn login(&self, request: LoginRequest) -> ServiceResult<AuthResponse> {
        // Find user by email
        let user = self
            .ctx
            .user_repo()
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| {
                warn!(email = %request.email, "Login failed: user not found");
                ServiceError::App(chat_common::AppError::InvalidCredentials)
            })?;

        // Get password hash
        let password_hash = self
            .ctx
            .user_repo()
            .get_password_hash(user.id)
            .await?
            .ok_or_else(|| {
                warn!(user_id = %user.id, "Login failed: no password hash");
                ServiceError::App(chat_common::AppError::InvalidCredentials)
            })?;

        // Verify password
        let is_valid = verify_password(&request.password, &password_hash)
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        if !is_valid {
            warn!(user_id = %user.id, "Login failed: invalid password");
            return Err(ServiceError::App(chat_common::AppError::InvalidCredentials));
        }

        info!(user_id = %user.id, "User logged in successfully");

        // Generate tokens
        let token_pair = self
            .ctx
            .jwt_service()
            .generate_token_pair(user.id)
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        // Store refresh token in Redis
        let session_id = Uuid::new_v4().to_string();
        let refresh_data = RefreshTokenData::new(user.id, session_id);
        self.ctx
            .refresh_token_store()
            .store(&token_pair.refresh_token, &refresh_data)
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        Ok(AuthResponse::new(
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.expires_in,
            CurrentUserResponse::from(&user),
        ))
    }

    /// Refresh access token using refresh token
    #[instrument(skip(self, request))]
    pub async fn refresh_tokens(
        &self,
        request: RefreshTokenRequest,
    ) -> ServiceResult<AuthResponse> {
        // Validate refresh token exists in Redis
        let refresh_data = self
            .ctx
            .refresh_token_store()
            .validate(&request.refresh_token)
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?
            .ok_or(ServiceError::App(chat_common::AppError::InvalidToken))?;

        // Get user
        let user = self
            .ctx
            .user_repo()
            .find_by_id(refresh_data.user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", refresh_data.user_id.to_string()))?;

        // Revoke old refresh token
        self.ctx
            .refresh_token_store()
            .revoke(&request.refresh_token)
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        // Generate new tokens
        let token_pair = self
            .ctx
            .jwt_service()
            .generate_token_pair(user.id)
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        // Store new refresh token
        let session_id = Uuid::new_v4().to_string();
        let new_refresh_data = RefreshTokenData::new(user.id, session_id);
        self.ctx
            .refresh_token_store()
            .store(&token_pair.refresh_token, &new_refresh_data)
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?;

        info!(user_id = %user.id, "Tokens refreshed successfully");

        Ok(AuthResponse::new(
            token_pair.access_token,
            token_pair.refresh_token,
            token_pair.expires_in,
            CurrentUserResponse::from(&user),
        ))
    }

    /// Logout user by revoking refresh token
    #[instrument(skip(self, refresh_token))]
    pub async fn logout(&self, user_id: Snowflake, refresh_token: Option<String>) -> ServiceResult<()> {
        if let Some(token) = refresh_token {
            // Revoke specific refresh token
            self.ctx
                .refresh_token_store()
                .revoke(&token)
                .await
                .map_err(|e| ServiceError::internal(e.to_string()))?;
        } else {
            // Revoke all refresh tokens for user
            self.ctx
                .refresh_token_store()
                .revoke_all_for_user(user_id)
                .await
                .map_err(|e| ServiceError::internal(e.to_string()))?;
        }

        info!(user_id = %user_id, "User logged out successfully");
        Ok(())
    }

    /// Validate an access token and return the user ID
    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> ServiceResult<Snowflake> {
        let claims = self
            .ctx
            .jwt_service()
            .validate_access_token(token)
            .map_err(ServiceError::from)?;

        claims.user_id().map_err(ServiceError::from)
    }

    /// Get user by access token
    #[instrument(skip(self, token))]
    pub async fn get_user_from_token(&self, token: &str) -> ServiceResult<User> {
        let user_id = self.validate_token(token).await?;

        self.ctx
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ServiceError::not_found("User", user_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would go here with mocked dependencies
}
