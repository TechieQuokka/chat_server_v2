//! Reaction handlers
//!
//! Endpoints for message reactions.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{ReactionService, UserResponse};

use crate::extractors::{AuthUser, Pagination};
use crate::response::{ApiError, ApiResult, NoContent};
use crate::state::AppState;

/// Add reaction to message
///
/// PUT /channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me
pub async fn add_reaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;

    let service = ReactionService::new(state.service_context());
    service
        .add_reaction(channel_id, message_id, auth.user_id, emoji)
        .await?;
    Ok(NoContent)
}

/// Remove own reaction
///
/// DELETE /channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me
pub async fn remove_own_reaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;

    let service = ReactionService::new(state.service_context());
    service
        .remove_reaction(channel_id, message_id, auth.user_id, emoji)
        .await?;
    Ok(NoContent)
}

/// Remove user's reaction
///
/// DELETE /channels/{channel_id}/messages/{message_id}/reactions/{emoji}/{user_id}
pub async fn remove_user_reaction(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id, emoji, user_id)): Path<(String, String, String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;
    let user_id = user_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid user_id format"))?;

    let service = ReactionService::new(state.service_context());
    service
        .remove_user_reaction(channel_id, message_id, auth.user_id, user_id, emoji)
        .await?;
    Ok(NoContent)
}

/// Get users who reacted with emoji
///
/// GET /channels/{channel_id}/messages/{message_id}/reactions/{emoji}
pub async fn get_reactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
    pagination: Pagination,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;

    let service = ReactionService::new(state.service_context());
    let users = service
        .get_reaction_users(
            channel_id,
            message_id,
            auth.user_id,
            emoji,
            i64::from(pagination.limit),
            pagination.after,
        )
        .await?;
    Ok(Json(users))
}

/// Delete all reactions for emoji
///
/// DELETE /channels/{channel_id}/messages/{message_id}/reactions/{emoji}
pub async fn delete_all_reactions_for_emoji(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id, emoji)): Path<(String, String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;

    let service = ReactionService::new(state.service_context());
    service
        .remove_all_reactions_for_emoji(channel_id, message_id, auth.user_id, emoji)
        .await?;
    Ok(NoContent)
}

/// Delete all reactions on message
///
/// DELETE /channels/{channel_id}/messages/{message_id}/reactions
pub async fn delete_all_reactions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| ApiError::invalid_path("Invalid message_id format"))?;

    let service = ReactionService::new(state.service_context());
    service
        .remove_all_reactions(channel_id, message_id, auth.user_id)
        .await?;
    Ok(NoContent)
}
