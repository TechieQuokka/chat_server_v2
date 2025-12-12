//! Application error types
//!
//! Unified error handling for the entire application.

use chat_core::DomainError;
use serde::Serialize;
use std::fmt;

/// Application-wide error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Authentication errors
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Missing authentication")]
    MissingAuth,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // Resource errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    // Rate limiting
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    // Database errors
    #[error("Database error: {0}")]
    Database(String),

    // Redis errors
    #[error("Cache error: {0}")]
    Cache(String),

    // External service errors
    #[error("External service error: {0}")]
    ExternalService(String),

    // Internal errors
    #[error("Internal server error")]
    Internal(#[source] anyhow::Error),

    // Domain errors
    #[error(transparent)]
    Domain(#[from] DomainError),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
}

impl AppError {
    /// Get HTTP status code for this error
    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            // 400 Bad Request
            Self::Validation(_) | Self::InvalidInput(_) => 400,

            // 401 Unauthorized
            Self::InvalidCredentials | Self::InvalidToken | Self::TokenExpired | Self::MissingAuth => 401,

            // 403 Forbidden
            Self::InsufficientPermissions => 403,

            // 404 Not Found
            Self::NotFound(_) => 404,

            // 409 Conflict
            Self::AlreadyExists(_) | Self::Conflict(_) => 409,

            // 429 Too Many Requests
            Self::RateLimitExceeded => 429,

            // 500 Internal Server Error
            Self::Database(_)
            | Self::Cache(_)
            | Self::ExternalService(_)
            | Self::Internal(_)
            | Self::Config(_) => 500,

            // Map domain errors to appropriate status codes
            Self::Domain(e) => {
                if e.is_not_found() {
                    404
                } else if e.is_authorization() {
                    403
                } else if e.is_validation() {
                    400
                } else if e.is_conflict() {
                    409
                } else {
                    500
                }
            }
        }
    }

    /// Get error code for API responses
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::MissingAuth => "MISSING_AUTH",
            Self::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::AlreadyExists(_) => "ALREADY_EXISTS",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Cache(_) => "CACHE_ERROR",
            Self::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Config(_) => "CONFIG_ERROR",
            Self::Domain(e) => e.code(),
        }
    }

    /// Check if this is a client error (4xx)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        let status = self.status_code();
        (400..500).contains(&status)
    }

    /// Check if this is a server error (5xx)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        let status = self.status_code();
        (500..600).contains(&status)
    }

    /// Create a not found error for a resource type
    #[must_use]
    pub fn not_found(resource: impl fmt::Display) -> Self {
        Self::NotFound(resource.to_string())
    }

    /// Create a validation error
    #[must_use]
    pub fn validation(msg: impl fmt::Display) -> Self {
        Self::Validation(msg.to_string())
    }

    /// Create an internal error from any error
    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        Self::Internal(err.into())
    }
}

/// Error response structure for API responses
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<&AppError> for ErrorResponse {
    fn from(err: &AppError) -> Self {
        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        Self::from(&err)
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_codes() {
        assert_eq!(AppError::InvalidCredentials.status_code(), 401);
        assert_eq!(AppError::InsufficientPermissions.status_code(), 403);
        assert_eq!(AppError::NotFound("user".to_string()).status_code(), 404);
        assert_eq!(AppError::Validation("test".to_string()).status_code(), 400);
        assert_eq!(AppError::RateLimitExceeded.status_code(), 429);
        assert_eq!(AppError::Database("test".to_string()).status_code(), 500);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AppError::NotFound("user".to_string()).error_code(), "NOT_FOUND");
        assert_eq!(AppError::RateLimitExceeded.error_code(), "RATE_LIMIT_EXCEEDED");
    }

    #[test]
    fn test_is_client_error() {
        assert!(AppError::InvalidCredentials.is_client_error());
        assert!(AppError::NotFound("test".to_string()).is_client_error());
        assert!(!AppError::Database("test".to_string()).is_client_error());
    }

    #[test]
    fn test_is_server_error() {
        assert!(!AppError::InvalidCredentials.is_server_error());
        assert!(AppError::Database("test".to_string()).is_server_error());
        assert!(AppError::Cache("test".to_string()).is_server_error());
    }

    #[test]
    fn test_error_response() {
        let err = AppError::NotFound("user".to_string());
        let response = ErrorResponse::from(&err);

        assert_eq!(response.code, "NOT_FOUND");
        assert_eq!(response.message, "Resource not found: user");
        assert!(response.details.is_none());
    }

    #[test]
    fn test_helper_methods() {
        let err = AppError::not_found("user 123");
        assert_eq!(err.to_string(), "Resource not found: user 123");

        let err = AppError::validation("email is required");
        assert_eq!(err.to_string(), "Validation error: email is required");
    }
}
