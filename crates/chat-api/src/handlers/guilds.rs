//! Guild handlers
//!
//! Endpoints for guild management.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{
    CreateGuildRequest, GuildResponse, GuildService, GuildWithCountsResponse, PermissionService,
    UpdateGuildRequest,
};

use crate::extractors::{AuthUser, ValidatedJson};
use crate::response::{ApiError, ApiResult, Created, NoContent};
use crate::state::AppState;

/// Create a new guild
///
/// POST /guilds
pub async fn create_guild(
    State(state): State<AppState>,
    auth: AuthUser,
    ValidatedJson(request): ValidatedJson<CreateGuildRequest>,
) -> ApiResult<Created<Json<GuildResponse>>> {
    let service = GuildService::new(state.service_context());
    let response = service.create_guild(auth.user_id, request).await?;
    Ok(Created(Json(response)))
}

/// Get guild by ID
///
/// GET /guilds/{guild_id}
pub async fn get_guild(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<Json<GuildWithCountsResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    // Check if user is a member
    let permission_service = PermissionService::new(state.service_context());
    if !permission_service.is_guild_member(guild_id, auth.user_id).await? {
        return Err(ApiError::Service(chat_service::ServiceError::not_found(
            "Guild",
            guild_id.to_string(),
        )));
    }

    let service = GuildService::new(state.service_context());
    let response = service.get_guild_with_counts(guild_id).await?;
    Ok(Json(response))
}

/// Update guild settings
///
/// PATCH /guilds/{guild_id}
pub async fn update_guild(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
    ValidatedJson(request): ValidatedJson<UpdateGuildRequest>,
) -> ApiResult<Json<GuildResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = GuildService::new(state.service_context());
    let response = service.update_guild(guild_id, auth.user_id, request).await?;
    Ok(Json(response))
}

/// Delete guild
///
/// DELETE /guilds/{guild_id}
pub async fn delete_guild(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<NoContent> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = GuildService::new(state.service_context());
    service.delete_guild(guild_id, auth.user_id).await?;
    Ok(NoContent)
}
