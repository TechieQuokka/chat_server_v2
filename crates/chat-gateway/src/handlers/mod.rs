//! Op code handlers
//!
//! Handles incoming WebSocket messages based on their operation code.

mod error;
mod heartbeat;
mod identify;
mod presence;
mod resume;

pub use error::{HandlerError, HandlerResult};
pub use heartbeat::HeartbeatHandler;
pub use identify::IdentifyHandler;
pub use presence::PresenceHandler;
pub use resume::ResumeHandler;

use crate::connection::Connection;
use crate::protocol::{CloseCode, GatewayMessage, OpCode};
use crate::server::GatewayState;
use std::sync::Arc;

/// Dispatch incoming client messages to appropriate handlers
pub struct MessageDispatcher;

impl MessageDispatcher {
    /// Handle an incoming client message
    pub async fn dispatch(
        state: &GatewayState,
        connection: &Arc<Connection>,
        message: GatewayMessage,
    ) -> HandlerResult<Option<CloseCode>> {
        // Validate that this is a client-sendable op code
        if !message.op.is_client_op() {
            tracing::warn!(
                session_id = %connection.session_id(),
                op = %message.op,
                "Received server-only op code from client"
            );
            return Ok(Some(CloseCode::UnknownOpcode));
        }

        match message.op {
            OpCode::Identify => {
                let payload = message.as_identify().ok_or_else(|| {
                    HandlerError::InvalidPayload("Invalid Identify payload".to_string())
                })?;

                IdentifyHandler::handle(state, connection, payload).await
            }
            OpCode::Resume => {
                let payload = message.as_resume().ok_or_else(|| {
                    HandlerError::InvalidPayload("Invalid Resume payload".to_string())
                })?;

                ResumeHandler::handle(state, connection, payload).await
            }
            OpCode::Heartbeat => {
                let seq = message.as_heartbeat_seq().ok_or_else(|| {
                    HandlerError::InvalidPayload("Invalid Heartbeat payload".to_string())
                })?;

                HeartbeatHandler::handle(connection, seq).await
            }
            OpCode::PresenceUpdate => {
                let payload = message.as_presence_update().ok_or_else(|| {
                    HandlerError::InvalidPayload("Invalid PresenceUpdate payload".to_string())
                })?;

                PresenceHandler::handle(state, connection, payload).await
            }
            // These ops should never reach here due to is_client_op check
            _ => {
                tracing::error!(op = %message.op, "Unhandled client op code");
                Ok(Some(CloseCode::UnknownOpcode))
            }
        }
    }
}
