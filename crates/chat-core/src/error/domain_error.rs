//! Domain errors - error types for the domain layer

use thiserror::Error;

use crate::value_objects::Snowflake;

/// Domain layer errors
#[derive(Debug, Error)]
pub enum DomainError {
    // =========================================================================
    // Not Found Errors
    // =========================================================================
    #[error("User not found: {0}")]
    UserNotFound(Snowflake),

    #[error("Guild not found: {0}")]
    GuildNotFound(Snowflake),

    #[error("Channel not found: {0}")]
    ChannelNotFound(Snowflake),

    #[error("Message not found: {0}")]
    MessageNotFound(Snowflake),

    #[error("Role not found: {0}")]
    RoleNotFound(Snowflake),

    #[error("Member not found in guild")]
    MemberNotFound,

    #[error("Invite not found: {0}")]
    InviteNotFound(String),

    // =========================================================================
    // Validation Errors
    // =========================================================================
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Password too weak: {0}")]
    WeakPassword(String),

    #[error("Content too long: max {max} characters")]
    ContentTooLong { max: usize },

    // =========================================================================
    // Authorization Errors
    // =========================================================================
    #[error("Missing permission: {0}")]
    MissingPermission(String),

    #[error("Not guild owner")]
    NotGuildOwner,

    #[error("Not message author")]
    NotMessageAuthor,

    #[error("Cannot modify higher role")]
    CannotModifyHigherRole,

    #[error("Cannot modify @everyone role")]
    CannotModifyEveryoneRole,

    // =========================================================================
    // Conflict Errors
    // =========================================================================
    #[error("Email already in use")]
    EmailAlreadyExists,

    #[error("Already a member of this guild")]
    AlreadyMember,

    #[error("Already has this role")]
    AlreadyHasRole,

    #[error("Reaction already exists")]
    ReactionAlreadyExists,

    #[error("Invite code already exists")]
    InviteCodeExists,

    // =========================================================================
    // Business Rule Violations
    // =========================================================================
    #[error("Cannot leave owned guild (transfer ownership first)")]
    CannotLeaveOwnedGuild,

    #[error("Cannot kick guild owner")]
    CannotKickOwner,

    #[error("Cannot ban guild owner")]
    CannotBanOwner,

    #[error("User is banned from this guild")]
    UserBanned,

    #[error("Invite has expired")]
    InviteExpired,

    #[error("Invite has reached maximum uses")]
    InviteExhausted,

    #[error("Cannot send messages in this channel")]
    CannotSendMessages,

    #[error("Cannot delete @everyone role")]
    CannotDeleteEveryoneRole,

    // =========================================================================
    // Infrastructure Errors (wrapped)
    // =========================================================================
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl DomainError {
    /// Get an error code string for API responses
    pub fn code(&self) -> &'static str {
        match self {
            // Not Found
            Self::UserNotFound(_) => "UNKNOWN_USER",
            Self::GuildNotFound(_) => "UNKNOWN_GUILD",
            Self::ChannelNotFound(_) => "UNKNOWN_CHANNEL",
            Self::MessageNotFound(_) => "UNKNOWN_MESSAGE",
            Self::RoleNotFound(_) => "UNKNOWN_ROLE",
            Self::MemberNotFound => "UNKNOWN_MEMBER",
            Self::InviteNotFound(_) => "UNKNOWN_INVITE",

            // Validation
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::InvalidEmail => "INVALID_EMAIL",
            Self::InvalidUsername(_) => "INVALID_USERNAME",
            Self::WeakPassword(_) => "WEAK_PASSWORD",
            Self::ContentTooLong { .. } => "CONTENT_TOO_LONG",

            // Authorization
            Self::MissingPermission(_) => "MISSING_PERMISSIONS",
            Self::NotGuildOwner => "NOT_GUILD_OWNER",
            Self::NotMessageAuthor => "NOT_MESSAGE_AUTHOR",
            Self::CannotModifyHigherRole => "CANNOT_MODIFY_HIGHER_ROLE",
            Self::CannotModifyEveryoneRole => "CANNOT_MODIFY_EVERYONE_ROLE",

            // Conflict
            Self::EmailAlreadyExists => "EMAIL_ALREADY_EXISTS",
            Self::AlreadyMember => "ALREADY_MEMBER",
            Self::AlreadyHasRole => "ALREADY_HAS_ROLE",
            Self::ReactionAlreadyExists => "REACTION_ALREADY_EXISTS",
            Self::InviteCodeExists => "INVITE_CODE_EXISTS",

            // Business Rules
            Self::CannotLeaveOwnedGuild => "CANNOT_LEAVE_OWNED_GUILD",
            Self::CannotKickOwner => "CANNOT_KICK_OWNER",
            Self::CannotBanOwner => "CANNOT_BAN_OWNER",
            Self::UserBanned => "USER_BANNED",
            Self::InviteExpired => "INVITE_EXPIRED",
            Self::InviteExhausted => "INVITE_EXHAUSTED",
            Self::CannotSendMessages => "CANNOT_SEND_MESSAGES",
            Self::CannotDeleteEveryoneRole => "CANNOT_DELETE_EVERYONE_ROLE",

            // Infrastructure
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    /// Check if this is a "not found" error
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::UserNotFound(_)
                | Self::GuildNotFound(_)
                | Self::ChannelNotFound(_)
                | Self::MessageNotFound(_)
                | Self::RoleNotFound(_)
                | Self::MemberNotFound
                | Self::InviteNotFound(_)
        )
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(
            self,
            Self::ValidationError(_)
                | Self::InvalidEmail
                | Self::InvalidUsername(_)
                | Self::WeakPassword(_)
                | Self::ContentTooLong { .. }
        )
    }

    /// Check if this is an authorization error
    pub fn is_authorization(&self) -> bool {
        matches!(
            self,
            Self::MissingPermission(_)
                | Self::NotGuildOwner
                | Self::NotMessageAuthor
                | Self::CannotModifyHigherRole
                | Self::CannotModifyEveryoneRole
        )
    }

    /// Check if this is a conflict error
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            Self::EmailAlreadyExists
                | Self::AlreadyMember
                | Self::AlreadyHasRole
                | Self::ReactionAlreadyExists
                | Self::InviteCodeExists
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = DomainError::UserNotFound(Snowflake::new(1));
        assert_eq!(err.code(), "UNKNOWN_USER");

        let err = DomainError::MissingPermission("MANAGE_GUILD".to_string());
        assert_eq!(err.code(), "MISSING_PERMISSIONS");
    }

    #[test]
    fn test_is_not_found() {
        assert!(DomainError::UserNotFound(Snowflake::new(1)).is_not_found());
        assert!(DomainError::GuildNotFound(Snowflake::new(1)).is_not_found());
        assert!(!DomainError::EmailAlreadyExists.is_not_found());
    }

    #[test]
    fn test_is_authorization() {
        assert!(DomainError::NotGuildOwner.is_authorization());
        assert!(DomainError::MissingPermission("test".to_string()).is_authorization());
        assert!(!DomainError::UserNotFound(Snowflake::new(1)).is_authorization());
    }

    #[test]
    fn test_error_display() {
        let err = DomainError::UserNotFound(Snowflake::new(123));
        assert_eq!(err.to_string(), "User not found: 123");

        let err = DomainError::ContentTooLong { max: 2000 };
        assert_eq!(err.to_string(), "Content too long: max 2000 characters");
    }
}
