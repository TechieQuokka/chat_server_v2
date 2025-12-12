//! WebSocket handler
//!
//! Handles WebSocket connections and message processing.

use crate::connection::{Connection, ConnectionState, Session};
use crate::handlers::MessageDispatcher;
use crate::protocol::{CloseCode, GatewayMessage, HelloPayload};
use crate::server::GatewayState;
use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Default heartbeat interval in milliseconds
const HEARTBEAT_INTERVAL_MS: u64 = 45_000;

/// Timeout for no heartbeat before considering connection dead
const HEARTBEAT_TIMEOUT_MS: u64 = 90_000;

/// Channel buffer size for outgoing messages
const MESSAGE_BUFFER_SIZE: usize = 100;

/// WebSocket gateway handler
pub async fn gateway_handler(
    State(state): State<GatewayState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(state, socket))
}

/// Handle an upgraded WebSocket connection
async fn handle_socket(state: GatewayState, socket: axum::extract::ws::WebSocket) {
    // Generate session ID
    let session_id = Session::generate_id();

    // Create message channel for outgoing messages
    let (tx, mut rx) = mpsc::channel::<GatewayMessage>(MESSAGE_BUFFER_SIZE);

    // Register connection
    let connection = state
        .connection_manager()
        .add_connection(session_id.clone(), tx);

    tracing::info!(session_id = %session_id, "WebSocket connection established");

    // Split the WebSocket
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Send Hello message immediately
    let hello = GatewayMessage::hello(HelloPayload::with_interval(HEARTBEAT_INTERVAL_MS));
    if let Ok(json) = hello.to_json() {
        if ws_sink.send(Message::Text(json.into())).await.is_err() {
            tracing::warn!(session_id = %session_id, "Failed to send Hello message");
            cleanup_connection(&state, &session_id, &connection).await;
            return;
        }
    }

    // Clone state for tasks
    let state_recv = state.clone();
    let session_id_recv = session_id.clone();
    let connection_recv = connection.clone();

    // Spawn task to receive messages from WebSocket
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(close_code) =
                        handle_text_message(&state_recv, &connection_recv, &text).await
                    {
                        tracing::debug!(
                            session_id = %session_id_recv,
                            close_code = ?close_code,
                            "Closing connection due to error"
                        );
                        return Some(close_code);
                    }
                }
                Ok(Message::Binary(_)) => {
                    tracing::debug!(
                        session_id = %session_id_recv,
                        "Binary messages not supported"
                    );
                    return Some(CloseCode::DecodeError);
                }
                Ok(Message::Ping(_)) => {
                    tracing::trace!(session_id = %session_id_recv, "Ping received");
                    // Pong is handled automatically by axum
                }
                Ok(Message::Pong(_)) => {
                    tracing::trace!(session_id = %session_id_recv, "Pong received");
                }
                Ok(Message::Close(_)) => {
                    tracing::info!(session_id = %session_id_recv, "Client closed connection");
                    return None;
                }
                Err(e) => {
                    tracing::warn!(
                        session_id = %session_id_recv,
                        error = %e,
                        "WebSocket error"
                    );
                    return Some(CloseCode::UnknownError);
                }
            }
        }
        None
    });

    // Clone for send task
    let session_id_send = session_id.clone();

    // Spawn task to send messages to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = msg.to_json() {
                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!(
                        session_id = %session_id_send,
                        "Failed to send message to WebSocket"
                    );
                    break;
                }
            }
        }

        // Close the WebSocket when channel is closed
        let _ = ws_sink.close().await;
    });

    // Clone for heartbeat task
    let session_id_hb = session_id.clone();
    let connection_hb = connection.clone();

    // Spawn heartbeat monitoring task
    let heartbeat_task = tokio::spawn(async move {
        let mut check_interval = interval(Duration::from_millis(HEARTBEAT_INTERVAL_MS / 2));

        loop {
            check_interval.tick().await;

            // Check if connection is dead (no heartbeat for too long)
            let time_since = connection_hb.time_since_heartbeat().await;
            if time_since > Duration::from_millis(HEARTBEAT_TIMEOUT_MS) {
                tracing::warn!(
                    session_id = %session_id_hb,
                    time_since_ms = time_since.as_millis(),
                    "Connection timed out (no heartbeat)"
                );
                break;
            }

            // Check if we're waiting for a heartbeat ACK that never came
            if !connection_hb.is_heartbeat_acked().await
                && time_since > Duration::from_millis(HEARTBEAT_INTERVAL_MS)
            {
                tracing::warn!(
                    session_id = %session_id_hb,
                    "Connection zombied (heartbeat not ACKed)"
                );
                break;
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        result = recv_task => {
            if let Ok(Some(close_code)) = result {
                tracing::debug!(
                    session_id = %session_id,
                    close_code = ?close_code,
                    "Receive task ended with close code"
                );
            }
        }
        _ = send_task => {
            tracing::debug!(session_id = %session_id, "Send task ended");
        }
        _ = heartbeat_task => {
            tracing::debug!(session_id = %session_id, "Heartbeat task ended");
        }
    }

    // Clean up
    cleanup_connection(&state, &session_id, &connection).await;
}

/// Handle a text message from the client
async fn handle_text_message(
    state: &GatewayState,
    connection: &Arc<Connection>,
    text: &str,
) -> Result<(), CloseCode> {
    // Parse the message
    let message = match GatewayMessage::from_json(text) {
        Ok(m) => m,
        Err(e) => {
            tracing::debug!(
                session_id = %connection.session_id(),
                error = %e,
                "Failed to parse message"
            );
            return Err(CloseCode::DecodeError);
        }
    };

    tracing::trace!(
        session_id = %connection.session_id(),
        op = %message.op,
        "Received message"
    );

    // Dispatch to handler
    match MessageDispatcher::dispatch(state, connection, message).await {
        Ok(Some(close_code)) => Err(close_code),
        Ok(None) => Ok(()),
        Err(e) => {
            tracing::warn!(
                session_id = %connection.session_id(),
                error = %e,
                "Handler error"
            );
            Err(e.to_close_code().unwrap_or(CloseCode::UnknownError))
        }
    }
}

/// Clean up a connection on disconnect
async fn cleanup_connection(state: &GatewayState, session_id: &str, connection: &Arc<Connection>) {
    tracing::info!(session_id = %session_id, "Cleaning up connection");

    // Set connection state
    connection.set_state(ConnectionState::Disconnected).await;

    // Mark session as disconnected in Redis (starts 2-minute resume window)
    if connection.is_authenticated().await {
        Session::disconnect(state.service_context().session_store(), session_id)
            .await
            .ok();

        // Update presence to offline
        if let Some(user_id) = connection.user_id().await {
            // Check if user has other active connections
            let other_connections = state.connection_manager().get_user_connections(user_id);
            let has_other_connections = other_connections
                .iter()
                .any(|c| c.session_id() != session_id);

            if !has_other_connections {
                // No other connections, set user to offline
                let presence_data =
                    chat_cache::PresenceData::new(user_id, chat_cache::UserStatus::Offline);
                state
                    .service_context()
                    .presence_store()
                    .set_presence(&presence_data)
                    .await
                    .ok();

                tracing::debug!(
                    user_id = %user_id,
                    "User presence set to offline"
                );
            }
        }
    }

    // Remove from connection manager
    state.connection_manager().remove_connection(session_id).await;
}
