//! Message handlers
//!
//! Endpoints for message operations.

use axum::{
    extract::{Path, State},
    Json,
};
use chat_service::{
    BulkDeleteMessagesRequest, CreateMessageRequest, MessageResponse, MessageService,
    UpdateMessageRequest,
};

use crate::extractors::{AuthUser, Pagination, ValidatedJson};
use crate::response::{ApiResult, Created, NoContent};
use crate::state::AppState;

/// Get messages in channel
///
/// GET /channels/{channel_id}/messages
pub async fn get_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    pagination: Pagination,
) -> ApiResult<Json<Vec<MessageResponse>>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;

    let service = MessageService::new(state.service_context());
    let messages = service
        .get_channel_messages(
            channel_id,
            auth.user_id,
            pagination.before,
            pagination.after,
            i64::from(pagination.limit),
        )
        .await?;
    Ok(Json(messages))
}

/// Create message
///
/// POST /channels/{channel_id}/messages
pub async fn create_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    ValidatedJson(request): ValidatedJson<CreateMessageRequest>,
) -> ApiResult<Created<Json<MessageResponse>>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;

    let service = MessageService::new(state.service_context());
    let response = service
        .create_message(channel_id, auth.user_id, request)
        .await?;
    Ok(Created(Json(response)))
}

/// Get message by ID
///
/// GET /channels/{channel_id}/messages/{message_id}
pub async fn get_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid message_id format"))?;

    let service = MessageService::new(state.service_context());
    let response = service
        .get_message(channel_id, message_id, auth.user_id)
        .await?;
    Ok(Json(response))
}

/// Edit message
///
/// PATCH /channels/{channel_id}/messages/{message_id}
pub async fn update_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
    ValidatedJson(request): ValidatedJson<UpdateMessageRequest>,
) -> ApiResult<Json<MessageResponse>> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid message_id format"))?;

    let service = MessageService::new(state.service_context());
    let response = service
        .update_message(channel_id, message_id, auth.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Delete message
///
/// DELETE /channels/{channel_id}/messages/{message_id}
pub async fn delete_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((channel_id, message_id)): Path<(String, String)>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;
    let message_id = message_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid message_id format"))?;

    let service = MessageService::new(state.service_context());
    service
        .delete_message(channel_id, message_id, auth.user_id)
        .await?;
    Ok(NoContent)
}

/// Bulk delete messages
///
/// POST /channels/{channel_id}/messages/bulk-delete
pub async fn bulk_delete_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(channel_id): Path<String>,
    ValidatedJson(request): ValidatedJson<BulkDeleteMessagesRequest>,
) -> ApiResult<NoContent> {
    let channel_id = channel_id
        .parse()
        .map_err(|_| crate::response::ApiError::invalid_path("Invalid channel_id format"))?;

    let service = MessageService::new(state.service_context());
    service
        .bulk_delete_messages(channel_id, auth.user_id, request.messages)
        .await?;
    Ok(NoContent)
}
