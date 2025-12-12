//! Presence Update handler (op 3)

use super::{HandlerError, HandlerResult};
use crate::connection::Connection;
use crate::events::{GatewayEventType, PresenceEvent, UserIdPayload};
use crate::protocol::{CloseCode, PresenceUpdatePayload};
use crate::server::GatewayState;
use chat_cache::{PubSubChannel, PubSubEvent, UserStatus};
use std::sync::Arc;

/// Handles Presence Update messages
pub struct PresenceHandler;

impl PresenceHandler {
    /// Handle a Presence Update message
    pub async fn handle(
        state: &GatewayState,
        connection: &Arc<Connection>,
        payload: PresenceUpdatePayload,
    ) -> HandlerResult<Option<CloseCode>> {
        // Must be authenticated
        let user_id = match connection.user_id().await {
            Some(id) => id,
            None => {
                tracing::warn!(
                    session_id = %connection.session_id(),
                    "Presence update from unauthenticated client"
                );
                return Ok(Some(CloseCode::NotAuthenticated));
            }
        };

        // Validate status
        if !payload.is_valid_status() {
            tracing::debug!(
                session_id = %connection.session_id(),
                status = %payload.status,
                "Invalid presence status"
            );
            return Err(HandlerError::InvalidPayload(format!(
                "Invalid status: {}. Must be one of: online, idle, dnd, offline",
                payload.status
            )));
        }

        // Parse status
        let status = match payload.status.as_str() {
            "online" => UserStatus::Online,
            "idle" => UserStatus::Idle,
            "dnd" => UserStatus::Dnd,
            "offline" => UserStatus::Offline,
            _ => UserStatus::Online, // Already validated above
        };

        // Update presence in Redis
        let presence_data = chat_cache::PresenceData::new(user_id, status);
        state
            .service_context()
            .presence_store()
            .set_presence(&presence_data)
            .await
            .map_err(HandlerError::CacheError)?;

        tracing::debug!(
            session_id = %connection.session_id(),
            user_id = %user_id,
            status = %payload.status,
            "Presence updated"
        );

        // Broadcast presence update to all guilds the user is in
        let guilds = connection.guilds().await;

        if !guilds.is_empty() {
            for guild_id in &guilds {
                let presence_event = PresenceEvent {
                    user: UserIdPayload { id: user_id },
                    guild_id: *guild_id,
                    status: payload.status.clone(),
                };

                let event_data = serde_json::to_value(&presence_event).unwrap_or_default();
                let pubsub_event = PubSubEvent::new(GatewayEventType::PresenceUpdate.as_str(), event_data);

                // Publish to guild channel
                state
                    .service_context()
                    .publisher()
                    .publish(&PubSubChannel::guild(*guild_id), &pubsub_event)
                    .await
                    .ok();
            }

            tracing::trace!(
                user_id = %user_id,
                guilds = guilds.len(),
                "Presence update broadcast to guilds"
            );
        }

        Ok(None)
    }
}
