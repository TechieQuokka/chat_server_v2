//! Service layer error types
//!
//! Provides a unified error type for all service operations.

use chat_common::AppError;
use chat_core::DomainError;
use std::fmt;

/// Service layer error type
#[derive(Debug)]
pub enum ServiceError {
    /// Domain rule violation
    Domain(DomainError),

    /// Application error (auth, validation, etc.)
    App(AppError),

    /// Resource not found
    NotFound { resource: &'static str, id: String },

    /// Permission denied
    PermissionDenied { permission: String },

    /// Validation error
    Validation(String),

    /// Conflict (e.g., duplicate resource)
    Conflict(String),

    /// Internal error
    Internal(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(e) => write!(f, "{e}"),
            Self::App(e) => write!(f, "{e}"),
            Self::NotFound { resource, id } => write!(f, "{resource} not found: {id}"),
            Self::PermissionDenied { permission } => {
                write!(f, "Missing required permission: {permission}")
            }
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::Conflict(msg) => write!(f, "Conflict: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Domain(e) => Some(e),
            Self::App(e) => Some(e),
            _ => None,
        }
    }
}

impl ServiceError {
    /// Create a not found error
    pub fn not_found(resource: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            id: id.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(permission: impl Into<String>) -> Self {
        Self::PermissionDenied {
            permission: permission.into(),
        }
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a conflict error
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
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
            Self::App(e) => e.status_code(),
            Self::NotFound { .. } => 404,
            Self::PermissionDenied { .. } => 403,
            Self::Validation(_) => 400,
            Self::Conflict(_) => 409,
            Self::Internal(_) => 500,
        }
    }

    /// Get the error code for API responses
    pub fn error_code(&self) -> &str {
        match self {
            Self::Domain(e) => e.code(),
            Self::App(e) => e.error_code(),
            Self::NotFound { .. } => "NOT_FOUND",
            Self::PermissionDenied { .. } => "MISSING_PERMISSIONS",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Conflict(_) => "CONFLICT",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl From<DomainError> for ServiceError {
    fn from(err: DomainError) -> Self {
        Self::Domain(err)
    }
}

impl From<AppError> for ServiceError {
    fn from(err: AppError) -> Self {
        Self::App(err)
    }
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Domain(e) => AppError::Domain(e),
            ServiceError::App(e) => e,
            ServiceError::NotFound { resource, id } => {
                AppError::NotFound(format!("{resource} {id}"))
            }
            ServiceError::PermissionDenied { permission: _ } => {
                AppError::InsufficientPermissions
            }
            ServiceError::Validation(msg) => AppError::Validation(msg),
            ServiceError::Conflict(msg) => AppError::Conflict(msg),
            ServiceError::Internal(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        }
    }
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = ServiceError::not_found("User", "123");
        assert_eq!(err.status_code(), 404);
        assert_eq!(err.error_code(), "NOT_FOUND");
        assert!(err.to_string().contains("User not found: 123"));
    }

    #[test]
    fn test_permission_denied_error() {
        let err = ServiceError::permission_denied("MANAGE_GUILD");
        assert_eq!(err.status_code(), 403);
        assert_eq!(err.error_code(), "MISSING_PERMISSIONS");
    }

    #[test]
    fn test_validation_error() {
        let err = ServiceError::validation("Invalid email format");
        assert_eq!(err.status_code(), 400);
        assert_eq!(err.error_code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_conflict_error() {
        let err = ServiceError::conflict("Email already exists");
        assert_eq!(err.status_code(), 409);
        assert_eq!(err.error_code(), "CONFLICT");
    }

    #[test]
    fn test_convert_to_app_error() {
        let service_err = ServiceError::not_found("Guild", "456");
        let app_err: AppError = service_err.into();
        assert_eq!(app_err.status_code(), 404);
    }
}
