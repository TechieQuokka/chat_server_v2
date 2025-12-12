//! Role handlers
//!
//! Endpoints for guild role management.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{CreateRoleRequest, PermissionService, RoleResponse, RoleService, UpdateRoleRequest};

use crate::extractors::{AuthUser, ValidatedJson};
use crate::response::{ApiError, ApiResult, Created, NoContent};
use crate::state::AppState;

/// Get guild roles
///
/// GET /guilds/{guild_id}/roles
pub async fn get_guild_roles(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<Json<Vec<RoleResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = RoleService::new(state.service_context());
    let roles = service.get_guild_roles(guild_id, auth.user_id).await?;
    Ok(Json(roles))
}

/// Create role in guild
///
/// POST /guilds/{guild_id}/roles
pub async fn create_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
    ValidatedJson(request): ValidatedJson<CreateRoleRequest>,
) -> ApiResult<Created<Json<RoleResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = RoleService::new(state.service_context());
    let response = service.create_role(guild_id, auth.user_id, request).await?;
    Ok(Created(Json(response)))
}

/// Get role by ID
///
/// GET /guilds/{guild_id}/roles/{role_id}
pub async fn get_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, role_id)): Path<(String, String)>,
) -> ApiResult<Json<RoleResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let role_id = role_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid role_id format"))?;

    // Check if user is a member
    let permission_service = PermissionService::new(state.service_context());
    if !permission_service.is_guild_member(guild_id, auth.user_id).await? {
        return Err(ApiError::Service(chat_service::ServiceError::not_found(
            "Guild",
            guild_id.to_string(),
        )));
    }

    let service = RoleService::new(state.service_context());
    let response = service.get_role(role_id).await?;

    Ok(Json(response))
}

/// Update role
///
/// PATCH /guilds/{guild_id}/roles/{role_id}
pub async fn update_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, role_id)): Path<(String, String)>,
    ValidatedJson(request): ValidatedJson<UpdateRoleRequest>,
) -> ApiResult<Json<RoleResponse>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let role_id = role_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid role_id format"))?;

    let service = RoleService::new(state.service_context());
    let response = service
        .update_role(guild_id, role_id, auth.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Delete role
///
/// DELETE /guilds/{guild_id}/roles/{role_id}
pub async fn delete_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((guild_id, role_id)): Path<(String, String)>,
) -> ApiResult<NoContent> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;
    let role_id = role_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid role_id format"))?;

    let service = RoleService::new(state.service_context());
    service
        .delete_role(guild_id, role_id, auth.user_id)
        .await?;
    Ok(NoContent)
}
