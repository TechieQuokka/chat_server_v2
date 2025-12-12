//! Handler error types

use crate::protocol::CloseCode;
use chat_core::DomainError;
use thiserror::Error;

/// Handler error type
#[derive(Debug, Error)]
pub enum HandlerError {
    /// Invalid payload received
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Not authenticated
    #[error("Not authenticated")]
    NotAuthenticated,

    /// Already authenticated
    #[error("Already authenticated")]
    AlreadyAuthenticated,

    /// Session error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Service error
    #[error("Service error: {0}")]
    ServiceError(#[from] chat_service::ServiceError),

    /// Domain error (from repositories)
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(#[from] chat_cache::RedisPoolError),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl HandlerError {
    /// Convert to a close code (if applicable)
    pub fn to_close_code(&self) -> Option<CloseCode> {
        match self {
            Self::InvalidPayload(_) => Some(CloseCode::DecodeError),
            Self::AuthenticationFailed(_) => Some(CloseCode::AuthenticationFailed),
            Self::NotAuthenticated => Some(CloseCode::NotAuthenticated),
            Self::AlreadyAuthenticated => Some(CloseCode::AlreadyAuthenticated),
            Self::SessionError(_) => Some(CloseCode::SessionTimeout),
            Self::ServiceError(_) => Some(CloseCode::UnknownError),
            Self::DomainError(_) => Some(CloseCode::UnknownError),
            Self::CacheError(_) => Some(CloseCode::UnknownError),
            Self::Internal(_) => Some(CloseCode::UnknownError),
        }
    }
}

/// Handler result type
pub type HandlerResult<T> = Result<T, HandlerError>;
