//! User handlers
//!
//! Endpoints for user profile management and user's guilds.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{
    CurrentUserResponse, DmChannelResponse, DmService, GuildResponse, GuildService,
    PublicUserResponse, UpdateUserRequest, UserService,
};

use crate::extractors::{AuthUser, ValidatedJson};
use crate::response::ApiResult;
use crate::state::AppState;

/// Get current user
///
/// GET /users/@me
pub async fn get_current_user(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<CurrentUserResponse>> {
    let service = UserService::new(state.service_context());
    let response = service.get_current_user(auth.user_id).await?;
    Ok(Json(response))
}

/// Update current user
///
/// PATCH /users/@me
pub async fn update_current_user(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<UpdateUserRequest>,
) -> ApiResult<Json<CurrentUserResponse>> {
    let service = UserService::new(state.service_context());
    let response = service.update_user(auth.user_id, request).await?;
    Ok(Json(response))
}

/// Get user by ID (public profile)
///
/// GET /users/{user_id}
pub async fn get_user(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<PublicUserResponse>> {
    let user_id = user_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid user_id format"))?;

    let service = UserService::new(state.service_context());
    let response = service.get_user(user_id).await?;
    Ok(Json(response))
}

/// Get current user's guilds
///
/// GET /users/@me/guilds
pub async fn get_current_user_guilds(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<GuildResponse>>> {
    let service = GuildService::new(state.service_context());
    let guilds = service.get_user_guilds(auth.user_id).await?;
    Ok(Json(guilds))
}

/// Get current user's DM channels
///
/// GET /users/@me/channels
pub async fn get_dm_channels(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<DmChannelResponse>>> {
    let service = DmService::new(state.service_context());
    let channels = service.get_user_dms(auth.user_id).await?;
    Ok(Json(channels))
}

/// Create DM channel request
#[derive(Debug, serde::Deserialize)]
pub struct CreateDmRequest {
    pub recipient_id: String,
}

/// Create DM channel
///
/// POST /users/@me/channels
pub async fn create_dm_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(request): Json<CreateDmRequest>,
) -> ApiResult<Json<DmChannelResponse>> {
    let recipient_id = request
        .recipient_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_query("Invalid recipient_id format"))?;

    let service = DmService::new(state.service_context());
    let channel = service.create_dm(auth.user_id, recipient_id).await?;
    Ok(Json(channel))
}
