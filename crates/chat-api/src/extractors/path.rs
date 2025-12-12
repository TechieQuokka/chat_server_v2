//! Path parameter extractors
//!
//! Type-safe extraction of Snowflake IDs from path parameters.

use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use chat_core::Snowflake;
use serde::de::DeserializeOwned;

use crate::response::ApiError;

/// Extract a single Snowflake ID from a path parameter
#[derive(Debug, Clone)]
pub struct SnowflakePath<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for SnowflakePath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(inner) = Path::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| ApiError::invalid_path(e.to_string()))?;

        Ok(SnowflakePath(inner))
    }
}

/// Path parameters with guild_id
#[derive(Debug, serde::Deserialize)]
pub struct GuildIdPath {
    pub guild_id: String,
}

impl GuildIdPath {
    /// Parse guild_id as Snowflake
    pub fn guild_id(&self) -> Result<Snowflake, ApiError> {
        self.guild_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))
    }
}

/// Path parameters with channel_id
#[derive(Debug, serde::Deserialize)]
pub struct ChannelIdPath {
    pub channel_id: String,
}

impl ChannelIdPath {
    /// Parse channel_id as Snowflake
    pub fn channel_id(&self) -> Result<Snowflake, ApiError> {
        self.channel_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))
    }
}

/// Path parameters with user_id
#[derive(Debug, serde::Deserialize)]
pub struct UserIdPath {
    pub user_id: String,
}

impl UserIdPath {
    /// Parse user_id as Snowflake
    pub fn user_id(&self) -> Result<Snowflake, ApiError> {
        self.user_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid user_id format"))
    }
}

/// Path parameters with message_id
#[derive(Debug, serde::Deserialize)]
pub struct MessageIdPath {
    pub channel_id: String,
    pub message_id: String,
}

impl MessageIdPath {
    /// Parse channel_id as Snowflake
    pub fn channel_id(&self) -> Result<Snowflake, ApiError> {
        self.channel_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))
    }

    /// Parse message_id as Snowflake
    pub fn message_id(&self) -> Result<Snowflake, ApiError> {
        self.message_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid message_id format"))
    }
}

/// Path parameters with guild_id and user_id
#[derive(Debug, serde::Deserialize)]
pub struct GuildUserPath {
    pub guild_id: String,
    pub user_id: String,
}

impl GuildUserPath {
    /// Parse guild_id as Snowflake
    pub fn guild_id(&self) -> Result<Snowflake, ApiError> {
        self.guild_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))
    }

    /// Parse user_id as Snowflake
    pub fn user_id(&self) -> Result<Snowflake, ApiError> {
        self.user_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid user_id format"))
    }
}

/// Path parameters with guild_id and role_id
#[derive(Debug, serde::Deserialize)]
pub struct GuildRolePath {
    pub guild_id: String,
    pub role_id: String,
}

impl GuildRolePath {
    /// Parse guild_id as Snowflake
    pub fn guild_id(&self) -> Result<Snowflake, ApiError> {
        self.guild_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))
    }

    /// Parse role_id as Snowflake
    pub fn role_id(&self) -> Result<Snowflake, ApiError> {
        self.role_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid role_id format"))
    }
}

/// Path parameters for invite code
#[derive(Debug, serde::Deserialize)]
pub struct InviteCodePath {
    pub invite_code: String,
}

impl InviteCodePath {
    /// Get the invite code
    pub fn code(&self) -> &str {
        &self.invite_code
    }
}

/// Path parameters for reactions
#[derive(Debug, serde::Deserialize)]
pub struct ReactionPath {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: String,
}

impl ReactionPath {
    /// Parse channel_id as Snowflake
    pub fn channel_id(&self) -> Result<Snowflake, ApiError> {
        self.channel_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))
    }

    /// Parse message_id as Snowflake
    pub fn message_id(&self) -> Result<Snowflake, ApiError> {
        self.message_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid message_id format"))
    }

    /// Get the emoji (URL-decoded)
    pub fn emoji(&self) -> &str {
        &self.emoji
    }
}

/// Path parameters for removing user reaction
#[derive(Debug, serde::Deserialize)]
pub struct UserReactionPath {
    pub channel_id: String,
    pub message_id: String,
    pub emoji: String,
    pub user_id: String,
}

impl UserReactionPath {
    /// Parse channel_id as Snowflake
    pub fn channel_id(&self) -> Result<Snowflake, ApiError> {
        self.channel_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))
    }

    /// Parse message_id as Snowflake
    pub fn message_id(&self) -> Result<Snowflake, ApiError> {
        self.message_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid message_id format"))
    }

    /// Parse user_id as Snowflake
    pub fn user_id(&self) -> Result<Snowflake, ApiError> {
        self.user_id
            .parse()
            .map_err(|_| ApiError::invalid_path("Invalid user_id format"))
    }

    /// Get the emoji (URL-decoded)
    pub fn emoji(&self) -> &str {
        &self.emoji
    }
}
