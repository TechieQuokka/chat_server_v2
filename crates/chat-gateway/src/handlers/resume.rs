//! Resume handler (op 4)

use super::{HandlerError, HandlerResult};
use crate::connection::{Connection, Session};
use crate::events::{GatewayEventType, ResumedEvent};
use crate::protocol::{CloseCode, GatewayMessage, ResumePayload};
use crate::server::GatewayState;
use std::sync::Arc;

/// Handles Resume messages
pub struct ResumeHandler;

impl ResumeHandler {
    /// Handle a Resume message
    pub async fn handle(
        state: &GatewayState,
        connection: &Arc<Connection>,
        payload: ResumePayload,
    ) -> HandlerResult<Option<CloseCode>> {
        // Check if already authenticated
        if connection.is_authenticated().await {
            tracing::warn!(
                session_id = %connection.session_id(),
                "Client sent Resume while already authenticated"
            );
            return Ok(Some(CloseCode::AlreadyAuthenticated));
        }

        // Extract token (remove "Bearer " prefix if present)
        let token = payload.token.strip_prefix("Bearer ").unwrap_or(&payload.token);

        // Validate the token
        let claims = match state.service_context().jwt_service().validate_access_token(token) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!(error = %e, "Token validation failed during resume");
                // Send Invalid Session with resumable = false
                connection
                    .send(GatewayMessage::invalid_session(false))
                    .await
                    .ok();
                return Ok(None);
            }
        };

        let user_id = match claims.user_id() {
            Ok(id) => id,
            Err(_) => {
                // Invalid token claims
                connection
                    .send(GatewayMessage::invalid_session(false))
                    .await
                    .ok();
                return Ok(None);
            }
        };

        // Attempt to resume the session
        let resume_result = Session::resume(
            state.service_context().session_store(),
            &payload.session_id,
            user_id,
            payload.seq,
        )
        .await;

        match resume_result {
            Ok(Some((session, missed_events))) => {
                // Session resumed successfully
                tracing::info!(
                    old_session_id = %payload.session_id,
                    new_session_id = %connection.session_id(),
                    user_id = %user_id,
                    missed_events = missed_events.len(),
                    "Session resumed"
                );

                // Authenticate the new connection
                state
                    .connection_manager()
                    .authenticate_connection(connection.session_id(), user_id)
                    .await;

                // Restore sequence number
                connection.set_sequence(session.sequence);

                // Restore guild subscriptions
                for guild_id in &session.guilds {
                    state
                        .connection_manager()
                        .subscribe_to_guild(connection.session_id(), *guild_id)
                        .await;
                }
                connection.set_guilds(session.guilds).await;

                // Replay missed events
                for event in missed_events {
                    let msg = GatewayMessage::dispatch(event.event_type, event.sequence, event.data);
                    if connection.send(msg).await.is_err() {
                        tracing::warn!(
                            session_id = %connection.session_id(),
                            "Failed to replay missed event"
                        );
                        break;
                    }
                }

                // Send RESUMED event
                let resumed = ResumedEvent {};
                let seq = connection.next_sequence();
                let resumed_data = serde_json::to_value(&resumed).unwrap_or_default();

                connection
                    .send(GatewayMessage::dispatch(
                        GatewayEventType::Resumed.as_str(),
                        seq,
                        resumed_data,
                    ))
                    .await
                    .map_err(|e| HandlerError::Internal(format!("Failed to send RESUMED: {e}")))?;

                // Delete the old session (we're using a new session ID now)
                Session::delete(state.service_context().session_store(), &payload.session_id)
                    .await
                    .ok();

                // Create new session in Redis for this connection
                Session::create(
                    state.service_context().session_store(),
                    connection.session_id(),
                    user_id,
                    None,
                    Some(format!("ws://{}/gateway", state.config().gateway.address())),
                )
                .await
                .ok();

                // Set user presence to online
                let presence_data =
                    chat_cache::PresenceData::new(user_id, chat_cache::UserStatus::Online);
                state
                    .service_context()
                    .presence_store()
                    .set_presence(&presence_data)
                    .await
                    .ok();

                Ok(None)
            }
            Ok(None) => {
                // Session not found or not resumable
                tracing::debug!(
                    session_id = %payload.session_id,
                    user_id = %user_id,
                    "Resume failed: session not found or not resumable"
                );

                // Send Invalid Session with resumable = false
                connection
                    .send(GatewayMessage::invalid_session(false))
                    .await
                    .ok();

                Ok(None)
            }
            Err(e) => {
                tracing::error!(
                    session_id = %payload.session_id,
                    error = %e,
                    "Resume failed: cache error"
                );

                // Send Invalid Session
                connection
                    .send(GatewayMessage::invalid_session(false))
                    .await
                    .ok();

                Ok(None)
            }
        }
    }
}
