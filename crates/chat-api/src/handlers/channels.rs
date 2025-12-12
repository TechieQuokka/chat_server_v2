//! Channel handlers
//!
//! Endpoints for channel management.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_cache::{PubSubChannel, PubSubEvent};
use chat_service::{
    ChannelResponse, ChannelService, CreateChannelRequest, TypingResponse, UpdateChannelRequest,
};
use serde_json::json;

use crate::extractors::{AuthUser, ValidatedJson};
use crate::response::{ApiError, ApiResult, Created, NoContent};
use crate::state::AppState;

/// Get guild channels
///
/// GET /guilds/{guild_id}/channels
pub async fn get_guild_channels(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
) -> ApiResult<Json<Vec<ChannelResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = ChannelService::new(state.service_context());
    let channels = service.get_guild_channels(guild_id, auth.user_id).await?;
    Ok(Json(channels))
}

/// Create channel in guild
///
/// POST /guilds/{guild_id}/channels
pub async fn create_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(guild_id): Path<String>,
    ValidatedJson(request): ValidatedJson<CreateChannelRequest>,
) -> ApiResult<Created<Json<ChannelResponse>>> {
    let guild_id = guild_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid guild_id format"))?;

    let service = ChannelService::new(state.service_context());
    let response = service
        .create_channel(guild_id, auth.user_id, request)
        .await?;
    Ok(Created(Json(response)))
}

/// Get channel by ID
///
/// GET /channels/{channel_id}
pub async fn get_channel(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<ChannelResponse>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    let service = ChannelService::new(state.service_context());
    let response = service.get_channel(channel_id).await?;
    Ok(Json(response))
}

/// Update channel
///
/// PATCH /channels/{channel_id}
pub async fn update_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    ValidatedJson(request): ValidatedJson<UpdateChannelRequest>,
) -> ApiResult<Json<ChannelResponse>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    let service = ChannelService::new(state.service_context());
    let response = service
        .update_channel(channel_id, auth.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Delete channel
///
/// DELETE /channels/{channel_id}
pub async fn delete_channel(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    let service = ChannelService::new(state.service_context());
    service.delete_channel(channel_id, auth.user_id).await?;
    Ok(NoContent)
}

/// Trigger typing indicator
///
/// POST /channels/{channel_id}/typing
pub async fn typing_indicator(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<TypingResponse>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;

    // Get channel to verify access and get guild_id
    let service = ChannelService::new(state.service_context());
    let channel = service.get_channel_with_permission(channel_id, auth.user_id).await?;

    // Publish typing event
    let timestamp = chrono::Utc::now();
    let event = PubSubEvent::new(
        "TYPING_START",
        json!({
            "channel_id": channel_id.to_string(),
            "user_id": auth.user_id.to_string(),
            "guild_id": channel.guild_id.map(|id| id.to_string()),
            "timestamp": timestamp.to_rfc3339()
        }),
    );

    if let Some(guild_id) = channel.guild_id {
        state
            .service_context()
            .publisher()
            .publish(&PubSubChannel::guild(guild_id), &event)
            .await
            .ok();
    }

    Ok(Json(TypingResponse {
        channel_id: channel_id.to_string(),
        user_id: auth.user_id.to_string(),
        timestamp,
    }))
}
