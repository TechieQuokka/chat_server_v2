//! Member handlers
//!
//! Endpoints for guild member management.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{
    GuildService, MemberResponse, MemberService, PermissionService, UpdateMemberRequest,
};

use crate::extractors::{AuthUser, Pagination, ValidatedJson};
use crate::response::{ApiError, ApiResult, NoContent};
use crate::state::AppState;

/// Get guild members
///
/// GET /guilds/{guild_id}/members
pub async fn get_guild_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
    pagination: Pagination,
) -> ApiResult<Json<Vec<MemberResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = MemberService::new(state.service_context());
    let members = service
        .get_guild_members(
            guild_id,
            auth.user_id,
            i64::from(pagination.limit),
            pagination.after,
        )
        .await?;
    Ok(Json(members))
}

/// Get guild member by user ID
///
/// GET /guilds/{guild_id}/members/{user_id}
pub async fn get_guild_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, user_id)): Path<(String, String)>,
) -> ApiResult<Json<MemberResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let user_id = user_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid user_id format"))?;

    // Check if requesting user is a member
    let permission_service = PermissionService::new(state.service_context());
    if !permission_service.is_guild_member(guild_id, auth.user_id).await? {
        return Err(ApiError::Service(chat_service::ServiceError::not_found(
            "Guild",
            guild_id.to_string(),
        )));
    }

    let service = MemberService::new(state.service_context());
    let response = service.get_member(guild_id, user_id).await?;
    Ok(Json(response))
}

/// Update guild member
///
/// PATCH /guilds/{guild_id}/members/{user_id}
pub async fn update_guild_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, user_id)): Path<(String, String)>,
    ValidatedJson(request): ValidatedJson<UpdateMemberRequest>,
) -> ApiResult<Json<MemberResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let user_id = user_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid user_id format"))?;

    let service = MemberService::new(state.service_context());
    let response = service
        .update_member(guild_id, user_id, auth.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Remove guild member (kick)
///
/// DELETE /guilds/{guild_id}/members/{user_id}
pub async fn remove_guild_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, user_id)): Path<(String, String)>,
) -> ApiResult<NoContent> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let user_id = user_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid user_id format"))?;

    let service = MemberService::new(state.service_context());
    service
        .remove_member(guild_id, user_id, auth.user_id)
        .await?;
    Ok(NoContent)
}

/// Leave guild
///
/// DELETE /guilds/{guild_id}/members/@me
pub async fn leave_guild(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<NoContent> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = GuildService::new(state.service_context());
    service.leave_guild(guild_id, auth.user_id).await?;
    Ok(NoContent)
}
