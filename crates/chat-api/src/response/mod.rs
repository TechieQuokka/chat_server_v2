//! Response types and error handling for API endpoints
//!
//! Provides unified error handling and JSON response formatting.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chat_common::AppError;
use chat_core::DomainError;
use chat_service::ServiceError;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use validator::ValidationErrors;

/// API error type for consistent error responses
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    App(#[from] AppError),

    #[error("{0}")]
    Service(#[from] ServiceError),

    #[error("{0}")]
    Domain(#[from] DomainError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),

    #[error("Invalid query parameter: {0}")]
    InvalidQuery(String),

    #[error("Missing authorization header")]
    MissingAuth,

    #[error("Invalid authorization header format")]
    InvalidAuthFormat,

    #[error("Internal server error")]
    Internal(#[source] anyhow::Error),
}

impl ApiError {
    /// Get HTTP status code for this error
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::App(e) => StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Self::Service(e) => StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Self::Domain(e) => {
                if e.is_not_found() {
                    StatusCode::NOT_FOUND
                } else if e.is_authorization() {
                    StatusCode::FORBIDDEN
                } else if e.is_validation() {
                    StatusCode::BAD_REQUEST
                } else if e.is_conflict() {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            Self::Validation(_) | Self::InvalidPath(_) | Self::InvalidQuery(_) => StatusCode::BAD_REQUEST,
            Self::MissingAuth | Self::InvalidAuthFormat => StatusCode::UNAUTHORIZED,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for API responses
    #[must_use]
    pub fn error_code(&self) -> &str {
        match self {
            Self::App(e) => e.error_code(),
            Self::Service(e) => e.error_code(),
            Self::Domain(e) => e.code(),
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::InvalidPath(_) => "INVALID_PATH_PARAMETER",
            Self::InvalidQuery(_) => "INVALID_QUERY_PARAMETER",
            Self::MissingAuth => "MISSING_AUTHORIZATION",
            Self::InvalidAuthFormat => "INVALID_AUTHORIZATION_FORMAT",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Create an internal error from any error
    pub fn internal(err: impl Into<anyhow::Error>) -> Self {
        Self::Internal(err.into())
    }

    /// Create a validation error with a custom message
    pub fn invalid_path(msg: impl Into<String>) -> Self {
        Self::InvalidPath(msg.into())
    }

    /// Create an invalid query error
    pub fn invalid_query(msg: impl Into<String>) -> Self {
        Self::InvalidQuery(msg.into())
    }
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// Error detail for API responses
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code().to_string();
        let message = self.to_string();

        // Log server errors
        if status.is_server_error() {
            error!(error = ?self, "Server error occurred");
        }

        // Build details for validation errors
        let details = if let Self::Validation(errors) = &self {
            Some(serde_json::to_value(errors).unwrap_or_default())
        } else {
            None
        };

        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message,
                details,
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Type alias for API results
pub type ApiResult<T> = Result<T, ApiError>;

/// Wrapper for successful JSON responses
pub struct ApiJson<T>(pub T);

impl<T: Serialize> IntoResponse for ApiJson<T> {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

/// Created response (201) with JSON body
pub struct Created<T>(pub T);

impl<T: IntoResponse> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        *response.status_mut() = StatusCode::CREATED;
        response
    }
}

/// No content response (204)
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(ApiError::MissingAuth.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            ApiError::InvalidPath("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_api_error_codes() {
        assert_eq!(ApiError::MissingAuth.error_code(), "MISSING_AUTHORIZATION");
        assert_eq!(
            ApiError::InvalidPath("test".to_string()).error_code(),
            "INVALID_PATH_PARAMETER"
        );
    }
}
