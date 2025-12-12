//! Authentication extractor
//!
//! Extracts and validates JWT tokens from the Authorization header.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chat_core::Snowflake;

use crate::response::ApiError;
use crate::state::AppState;

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthUser {
    /// User ID from the JWT token
    pub user_id: Snowflake,
}

impl AuthUser {
    /// Create a new AuthUser
    pub fn new(user_id: Snowflake) -> Self {
        Self { user_id }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| ApiError::MissingAuth)?;

        // Get the app state to access JWT service
        let app_state = AppState::from_ref(state);

        // Validate the token
        let claims = app_state
            .jwt_service()
            .validate_access_token(bearer.token())
            .map_err(|e| {
                tracing::warn!(error = %e, "Invalid access token");
                ApiError::InvalidAuthFormat
            })?;

        // Extract user ID from claims
        let user_id = claims.user_id().map_err(|e| {
            tracing::warn!(error = %e, "Invalid user ID in token");
            ApiError::InvalidAuthFormat
        })?;

        Ok(AuthUser::new(user_id))
    }
}

/// Optional authenticated user
///
/// Returns None if no authorization header is present,
/// or an error if the token is invalid.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Try to extract the Authorization header
        let auth_result =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await;

        match auth_result {
            Ok(TypedHeader(Authorization(bearer))) => {
                // Get the app state to access JWT service
                let app_state = AppState::from_ref(state);

                // Validate the token
                let claims = app_state
                    .jwt_service()
                    .validate_access_token(bearer.token())
                    .map_err(|e| {
                        tracing::warn!(error = %e, "Invalid access token");
                        ApiError::InvalidAuthFormat
                    })?;

                // Extract user ID from claims
                let user_id = claims.user_id().map_err(|e| {
                    tracing::warn!(error = %e, "Invalid user ID in token");
                    ApiError::InvalidAuthFormat
                })?;

                Ok(OptionalAuthUser(Some(AuthUser::new(user_id))))
            }
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
