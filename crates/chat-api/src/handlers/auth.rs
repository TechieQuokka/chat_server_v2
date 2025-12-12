//! Authentication handlers
//!
//! Endpoints for user registration, login, logout, and token refresh.

use axum::{extract::State, Json};
use chat_service::{
    AuthResponse, AuthService, LoginRequest, RefreshTokenRequest, RegisterRequest,
};

use crate::extractors::{AuthUser, ValidatedJson};
use crate::response::{ApiResult, Created, NoContent};
use crate::state::AppState;

/// Register a new user
///
/// POST /auth/register
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<RegisterRequest>,
) -> ApiResult<Created<Json<AuthResponse>>> {
    let service = AuthService::new(state.service_context());
    let response = service.register(request).await?;
    Ok(Created(Json(response)))
}

/// Login with email and password
///
/// POST /auth/login
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let service = AuthService::new(state.service_context());
    let response = service.login(request).await?;
    Ok(Json(response))
}

/// Refresh access token
///
/// POST /auth/refresh
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let service = AuthService::new(state.service_context());
    let response = service.refresh_tokens(request).await?;
    Ok(Json(response))
}

/// Logout request body
#[derive(Debug, serde::Deserialize, Default)]
pub struct LogoutRequestBody {
    pub refresh_token: Option<String>,
}

/// Logout user
///
/// POST /auth/logout
pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
    body: Option<Json<LogoutRequestBody>>,
) -> ApiResult<NoContent> {
    let service = AuthService::new(state.service_context());
    let refresh_token = body.and_then(|b| b.0.refresh_token);
    service.logout(auth.user_id, refresh_token).await?;
    Ok(NoContent)
}
