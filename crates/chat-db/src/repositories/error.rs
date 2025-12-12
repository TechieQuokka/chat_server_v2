//! Error handling utilities for repositories

use chat_core::error::DomainError;
use chat_core::value_objects::Snowflake;
use sqlx::Error as SqlxError;

/// Convert SQLx error to DomainError
pub fn map_db_error(e: SqlxError) -> DomainError {
    DomainError::DatabaseError(e.to_string())
}

/// Check for unique violation and return appropriate error or fallback
pub fn map_unique_violation<F>(e: SqlxError, on_unique: F) -> DomainError
where
    F: FnOnce() -> DomainError,
{
    if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
            return on_unique();
        }
    }
    DomainError::DatabaseError(e.to_string())
}

/// Create a "user not found" error
pub fn user_not_found(id: Snowflake) -> DomainError {
    DomainError::UserNotFound(id)
}

/// Create a "guild not found" error
pub fn guild_not_found(id: Snowflake) -> DomainError {
    DomainError::GuildNotFound(id)
}

/// Create a "channel not found" error
pub fn channel_not_found(id: Snowflake) -> DomainError {
    DomainError::ChannelNotFound(id)
}

/// Create a "message not found" error
pub fn message_not_found(id: Snowflake) -> DomainError {
    DomainError::MessageNotFound(id)
}

/// Create a "role not found" error
pub fn role_not_found(id: Snowflake) -> DomainError {
    DomainError::RoleNotFound(id)
}

/// Create a "member not found" error
pub fn member_not_found() -> DomainError {
    DomainError::MemberNotFound
}

/// Create an "invite not found" error
pub fn invite_not_found(code: &str) -> DomainError {
    DomainError::InviteNotFound(code.to_string())
}

/// Create a "ban not found" error
pub fn ban_not_found() -> DomainError {
    DomainError::DatabaseError("Ban not found".to_string())
}
