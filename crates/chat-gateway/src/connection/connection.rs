//! Individual WebSocket connection
//!
//! Represents a single WebSocket connection and its state.

use crate::protocol::GatewayMessage;
use chat_core::Snowflake;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection established, waiting for Identify
    Connecting,
    /// Successfully authenticated
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection is closed
    Disconnected,
}

/// A single WebSocket connection
pub struct Connection {
    /// Unique session ID
    session_id: String,

    /// Authenticated user ID (None until Identify)
    user_id: RwLock<Option<Snowflake>>,

    /// Current connection state
    state: RwLock<ConnectionState>,

    /// Channel to send messages to the WebSocket
    sender: mpsc::Sender<GatewayMessage>,

    /// Last sequence number sent
    sequence: AtomicU64,

    /// Last heartbeat received
    last_heartbeat: RwLock<Instant>,

    /// Whether we've received a heartbeat ACK for the last heartbeat
    heartbeat_acked: RwLock<bool>,

    /// Guilds this connection is subscribed to
    guilds: RwLock<HashSet<Snowflake>>,

    /// Connection creation time
    created_at: Instant,
}

impl Connection {
    /// Create a new connection
    pub fn new(session_id: String, sender: mpsc::Sender<GatewayMessage>) -> Arc<Self> {
        Arc::new(Self {
            session_id,
            user_id: RwLock::new(None),
            state: RwLock::new(ConnectionState::Connecting),
            sender,
            sequence: AtomicU64::new(0),
            last_heartbeat: RwLock::new(Instant::now()),
            heartbeat_acked: RwLock::new(true),
            guilds: RwLock::new(HashSet::new()),
            created_at: Instant::now(),
        })
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the user ID (if authenticated)
    pub async fn user_id(&self) -> Option<Snowflake> {
        *self.user_id.read().await
    }

    /// Set the user ID (on successful authentication)
    pub async fn set_user_id(&self, user_id: Snowflake) {
        *self.user_id.write().await = Some(user_id);
    }

    /// Get the current state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Set the connection state
    pub async fn set_state(&self, state: ConnectionState) {
        *self.state.write().await = state;
    }

    /// Check if the connection is authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.user_id.read().await.is_some()
    }

    /// Get the next sequence number
    pub fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Get the current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    /// Set the sequence number (for resume)
    pub fn set_sequence(&self, seq: u64) {
        self.sequence.store(seq, Ordering::SeqCst);
    }

    /// Record a heartbeat received
    pub async fn record_heartbeat(&self) {
        *self.last_heartbeat.write().await = Instant::now();
    }

    /// Get time since last heartbeat
    pub async fn time_since_heartbeat(&self) -> std::time::Duration {
        self.last_heartbeat.read().await.elapsed()
    }

    /// Check if heartbeat was acknowledged
    pub async fn is_heartbeat_acked(&self) -> bool {
        *self.heartbeat_acked.read().await
    }

    /// Mark heartbeat as acknowledged
    pub async fn ack_heartbeat(&self) {
        *self.heartbeat_acked.write().await = true;
    }

    /// Mark heartbeat as pending (sent, waiting for ACK)
    pub async fn await_heartbeat_ack(&self) {
        *self.heartbeat_acked.write().await = false;
    }

    /// Add a guild subscription
    pub async fn subscribe_guild(&self, guild_id: Snowflake) {
        self.guilds.write().await.insert(guild_id);
    }

    /// Remove a guild subscription
    pub async fn unsubscribe_guild(&self, guild_id: Snowflake) {
        self.guilds.write().await.remove(&guild_id);
    }

    /// Get all subscribed guilds
    pub async fn guilds(&self) -> Vec<Snowflake> {
        self.guilds.read().await.iter().copied().collect()
    }

    /// Check if subscribed to a guild
    pub async fn is_subscribed_to(&self, guild_id: Snowflake) -> bool {
        self.guilds.read().await.contains(&guild_id)
    }

    /// Set guild subscriptions (for resume)
    pub async fn set_guilds(&self, guilds: Vec<Snowflake>) {
        *self.guilds.write().await = guilds.into_iter().collect();
    }

    /// Get connection age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Send a message to this connection
    pub async fn send(&self, message: GatewayMessage) -> Result<(), mpsc::error::SendError<GatewayMessage>> {
        self.sender.send(message).await
    }

    /// Try to send a message (non-blocking)
    pub fn try_send(&self, message: GatewayMessage) -> Result<(), mpsc::error::TrySendError<GatewayMessage>> {
        self.sender.try_send(message)
    }

    /// Get a clone of the sender channel
    pub fn sender(&self) -> mpsc::Sender<GatewayMessage> {
        self.sender.clone()
    }

    /// Check if the sender channel is closed
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("session_id", &self.session_id)
            .field("sequence", &self.sequence.load(Ordering::SeqCst))
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = Connection::new("session123".to_string(), tx);

        assert_eq!(conn.session_id(), "session123");
        assert!(conn.user_id().await.is_none());
        assert_eq!(conn.state().await, ConnectionState::Connecting);
        assert!(!conn.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_connection_authentication() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = Connection::new("session123".to_string(), tx);

        let user_id = Snowflake::from(12345i64);
        conn.set_user_id(user_id).await;
        conn.set_state(ConnectionState::Connected).await;

        assert!(conn.is_authenticated().await);
        assert_eq!(conn.user_id().await, Some(user_id));
        assert_eq!(conn.state().await, ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_connection_sequence() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = Connection::new("session123".to_string(), tx);

        assert_eq!(conn.current_sequence(), 0);
        assert_eq!(conn.next_sequence(), 1);
        assert_eq!(conn.next_sequence(), 2);
        assert_eq!(conn.current_sequence(), 2);

        conn.set_sequence(100);
        assert_eq!(conn.current_sequence(), 100);
    }

    #[tokio::test]
    async fn test_connection_guilds() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = Connection::new("session123".to_string(), tx);

        let guild1 = Snowflake::from(1i64);
        let guild2 = Snowflake::from(2i64);

        conn.subscribe_guild(guild1).await;
        conn.subscribe_guild(guild2).await;

        assert!(conn.is_subscribed_to(guild1).await);
        assert!(conn.is_subscribed_to(guild2).await);
        assert_eq!(conn.guilds().await.len(), 2);

        conn.unsubscribe_guild(guild1).await;
        assert!(!conn.is_subscribed_to(guild1).await);
        assert!(conn.is_subscribed_to(guild2).await);
    }

    #[tokio::test]
    async fn test_connection_heartbeat() {
        let (tx, _rx) = mpsc::channel(10);
        let conn = Connection::new("session123".to_string(), tx);

        assert!(conn.is_heartbeat_acked().await);

        conn.await_heartbeat_ack().await;
        assert!(!conn.is_heartbeat_acked().await);

        conn.ack_heartbeat().await;
        assert!(conn.is_heartbeat_acked().await);
    }
}
