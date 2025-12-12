//! Heartbeat handler (op 1)

use super::{HandlerError, HandlerResult};
use crate::connection::Connection;
use crate::protocol::{CloseCode, GatewayMessage};
use std::sync::Arc;

/// Handles heartbeat messages
pub struct HeartbeatHandler;

impl HeartbeatHandler {
    /// Handle a heartbeat from the client
    ///
    /// The `last_sequence` is the client's last received sequence number (or None if none received).
    pub async fn handle(
        connection: &Arc<Connection>,
        last_sequence: Option<u64>,
    ) -> HandlerResult<Option<CloseCode>> {
        // Record the heartbeat
        connection.record_heartbeat().await;

        // Log the heartbeat
        tracing::trace!(
            session_id = %connection.session_id(),
            client_seq = ?last_sequence,
            server_seq = connection.current_sequence(),
            "Heartbeat received"
        );

        // Send heartbeat ACK
        if let Err(e) = connection.send(GatewayMessage::heartbeat_ack()).await {
            tracing::warn!(
                session_id = %connection.session_id(),
                error = %e,
                "Failed to send heartbeat ACK"
            );
            return Err(HandlerError::Internal("Failed to send heartbeat ACK".to_string()));
        }

        // Mark heartbeat as acknowledged
        connection.ack_heartbeat().await;

        Ok(None)
    }
}
