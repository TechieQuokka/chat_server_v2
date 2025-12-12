//! Invite handlers
//!
//! Endpoints for guild invite management.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{CreateInviteRequest, InviteResponse, InviteService};

use crate::extractors::AuthUser;
use crate::response::{ApiError, ApiResult, Created, NoContent};
use crate::state::AppState;

/// Get guild invites
///
/// GET /guilds/{guild_id}/invites
pub async fn get_guild_invites(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<Json<Vec<InviteResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = InviteService::new(state.service_context());
    let invites = service.get_guild_invites(guild_id, auth.user_id).await?;
    Ok(Json(invites))
}

/// Get channel invites
///
/// GET /channels/{channel_id}/invites
pub async fn get_channel_invites(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<Vec<InviteResponse>>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    let service = InviteService::new(state.service_context());
    let invites = service
        .get_channel_invites(channel_id, auth.user_id)
        .await?;
    Ok(Json(invites))
}

/// Create channel invite
///
/// POST /channels/{channel_id}/invites
pub async fn create_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    body: Option<Json<CreateInviteRequest>>,
) -> ApiResult<Created<Json<InviteResponse>>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    // Use default values if no body provided
    let request = body.map(|j| j.0).unwrap_or_default();

    let service = InviteService::new(state.service_context());
    let response = service
        .create_invite(channel_id, auth.user_id, request)
        .await?;
    Ok(Created(Json(response)))
}

/// Get invite by code (no auth required)
///
/// GET /invites/{invite_code}
pub async fn get_invite(
    State(state): State<AppState>,
    Path(invite_code): Path<String>,
) -> ApiResult<Json<InviteResponse>> {
    let service = InviteService::new(state.service_context());
    let response = service.get_invite(&invite_code).await?;
    Ok(Json(response))
}

/// Accept invite (join guild)
///
/// POST /invites/{invite_code}
pub async fn accept_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(invite_code): Path<String>,
) -> ApiResult<Json<InviteResponse>> {
    let service = InviteService::new(state.service_context());
    let response = service.use_invite(&invite_code, auth.user_id).await?;
    Ok(Json(response))
}

/// Delete invite
///
/// DELETE /invites/{invite_code}
pub async fn delete_invite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(invite_code): Path<String>,
) -> ApiResult<NoContent> {
    let service = InviteService::new(state.service_context());
    service.delete_invite(&invite_code, auth.user_id).await?;
    Ok(NoContent)
}
