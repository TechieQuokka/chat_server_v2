//! Connection manager
//!
//! Manages all active WebSocket connections using DashMap for thread-safe access.

use super::{Connection, ConnectionState};
use crate::protocol::GatewayMessage;
use chat_core::Snowflake;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Manages all active WebSocket connections
///
/// Uses `DashMap` for concurrent access to connection state.
pub struct ConnectionManager {
    /// Active connections by session ID
    connections: DashMap<String, Arc<Connection>>,

    /// User ID to session IDs mapping
    user_connections: DashMap<Snowflake, HashSet<String>>,

    /// Guild ID to session IDs mapping
    guild_connections: DashMap<Snowflake, HashSet<String>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            user_connections: DashMap::new(),
            guild_connections: DashMap::new(),
        }
    }

    /// Create a new connection manager wrapped in Arc
    #[must_use]
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Register a new connection
    pub fn add_connection(
        &self,
        session_id: String,
        sender: mpsc::Sender<GatewayMessage>,
    ) -> Arc<Connection> {
        let connection = Connection::new(session_id.clone(), sender);
        self.connections.insert(session_id.clone(), connection.clone());

        tracing::debug!(session_id = %session_id, "Connection added");

        connection
    }

    /// Remove a connection
    ///
    /// Uses `alter` for atomic modify-and-cleanup operations to avoid TOCTOU race conditions.
    pub async fn remove_connection(&self, session_id: &str) {
        if let Some((_, connection)) = self.connections.remove(session_id) {
            // Remove from user mapping
            if let Some(user_id) = connection.user_id().await {
                // Atomically modify the sessions set
                self.user_connections.alter(&user_id, |_, mut sessions| {
                    sessions.remove(session_id);
                    sessions
                });

                // Clean up empty entry - use retain for atomic removal
                self.user_connections.retain(|_, sessions| !sessions.is_empty());
            }

            // Remove from guild mappings
            for guild_id in connection.guilds().await {
                // Atomically modify the sessions set
                self.guild_connections.alter(&guild_id, |_, mut sessions| {
                    sessions.remove(session_id);
                    sessions
                });
            }

            // Clean up all empty guild entries atomically
            self.guild_connections.retain(|_, sessions| !sessions.is_empty());

            tracing::debug!(session_id = %session_id, "Connection removed");
        }
    }

    /// Get a connection by session ID
    pub fn get_connection(&self, session_id: &str) -> Option<Arc<Connection>> {
        self.connections.get(session_id).map(|r| r.clone())
    }

    /// Authenticate a connection (link to user)
    pub async fn authenticate_connection(&self, session_id: &str, user_id: Snowflake) -> bool {
        if let Some(connection) = self.connections.get(session_id) {
            connection.set_user_id(user_id).await;
            connection.set_state(ConnectionState::Connected).await;

            // Add to user mapping
            self.user_connections
                .entry(user_id)
                .or_default()
                .insert(session_id.to_string());

            tracing::debug!(
                session_id = %session_id,
                user_id = %user_id,
                "Connection authenticated"
            );

            true
        } else {
            false
        }
    }

    /// Subscribe a connection to a guild
    pub async fn subscribe_to_guild(&self, session_id: &str, guild_id: Snowflake) -> bool {
        if let Some(connection) = self.connections.get(session_id) {
            connection.subscribe_guild(guild_id).await;

            self.guild_connections
                .entry(guild_id)
                .or_default()
                .insert(session_id.to_string());

            tracing::trace!(
                session_id = %session_id,
                guild_id = %guild_id,
                "Connection subscribed to guild"
            );

            true
        } else {
            false
        }
    }

    /// Unsubscribe a connection from a guild
    ///
    /// Uses atomic operations to avoid race conditions when cleaning up empty guild mappings.
    pub async fn unsubscribe_from_guild(&self, session_id: &str, guild_id: Snowflake) -> bool {
        if let Some(connection) = self.connections.get(session_id) {
            connection.unsubscribe_guild(guild_id).await;

            // Atomically modify the sessions set
            self.guild_connections.alter(&guild_id, |_, mut sessions| {
                sessions.remove(session_id);
                sessions
            });

            // Clean up empty entry
            self.guild_connections.retain(|_, sessions| !sessions.is_empty());

            tracing::trace!(
                session_id = %session_id,
                guild_id = %guild_id,
                "Connection unsubscribed from guild"
            );

            true
        } else {
            false
        }
    }

    /// Get all connections for a user
    pub fn get_user_connections(&self, user_id: Snowflake) -> Vec<Arc<Connection>> {
        self.user_connections
            .get(&user_id)
            .map(|sessions| {
                sessions
                    .iter()
                    .filter_map(|sid| self.connections.get(sid).map(|c| c.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all connections subscribed to a guild
    pub fn get_guild_connections(&self, guild_id: Snowflake) -> Vec<Arc<Connection>> {
        self.guild_connections
            .get(&guild_id)
            .map(|sessions| {
                sessions
                    .iter()
                    .filter_map(|sid| self.connections.get(sid).map(|c| c.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Send a message to all connections of a user
    pub async fn send_to_user(&self, user_id: Snowflake, message: GatewayMessage) -> usize {
        let connections = self.get_user_connections(user_id);
        let mut sent = 0;

        for conn in connections {
            if conn.send(message.clone()).await.is_ok() {
                sent += 1;
            }
        }

        tracing::trace!(
            user_id = %user_id,
            sent = sent,
            "Message sent to user connections"
        );

        sent
    }

    /// Send a message to all connections subscribed to a guild
    pub async fn send_to_guild(
        &self,
        guild_id: Snowflake,
        message: GatewayMessage,
        exclude_user: Option<Snowflake>,
    ) -> usize {
        let connections = self.get_guild_connections(guild_id);
        let mut sent = 0;

        for conn in connections {
            // Skip excluded user
            if let Some(exclude) = exclude_user {
                if conn.user_id().await == Some(exclude) {
                    continue;
                }
            }

            if conn.send(message.clone()).await.is_ok() {
                sent += 1;
            }
        }

        tracing::trace!(
            guild_id = %guild_id,
            sent = sent,
            "Message sent to guild connections"
        );

        sent
    }

    /// Broadcast a message to all connections
    pub async fn broadcast(&self, message: GatewayMessage) -> usize {
        let mut sent = 0;

        for entry in self.connections.iter() {
            if entry.send(message.clone()).await.is_ok() {
                sent += 1;
            }
        }

        tracing::debug!(sent = sent, "Message broadcast to all connections");

        sent
    }

    /// Get the total number of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get the number of unique authenticated users
    pub fn user_count(&self) -> usize {
        self.user_connections.len()
    }

    /// Get the number of guilds with active connections
    pub fn guild_count(&self) -> usize {
        self.guild_connections.len()
    }

    /// Get all session IDs
    pub fn all_sessions(&self) -> Vec<String> {
        self.connections.iter().map(|r| r.key().clone()).collect()
    }

    /// Check if a session exists
    pub fn has_session(&self, session_id: &str) -> bool {
        self.connections.contains_key(session_id)
    }

    /// Clean up closed connections
    pub async fn cleanup_closed_connections(&self) -> usize {
        let closed: Vec<String> = self
            .connections
            .iter()
            .filter(|r| r.is_closed())
            .map(|r| r.key().clone())
            .collect();

        let count = closed.len();

        for session_id in closed {
            self.remove_connection(&session_id).await;
        }

        if count > 0 {
            tracing::info!(count = count, "Cleaned up closed connections");
        }

        count
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionManager")
            .field("connections", &self.connections.len())
            .field("users", &self.user_connections.len())
            .field("guilds", &self.guild_connections.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.connection_count(), 0);
        assert_eq!(manager.user_count(), 0);
        assert_eq!(manager.guild_count(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_connection() {
        let manager = ConnectionManager::new();
        let (tx, _rx) = mpsc::channel(10);

        let conn = manager.add_connection("session1".to_string(), tx);
        assert_eq!(conn.session_id(), "session1");
        assert_eq!(manager.connection_count(), 1);
        assert!(manager.has_session("session1"));

        manager.remove_connection("session1").await;
        assert_eq!(manager.connection_count(), 0);
        assert!(!manager.has_session("session1"));
    }

    #[tokio::test]
    async fn test_authenticate_connection() {
        let manager = ConnectionManager::new();
        let (tx, _rx) = mpsc::channel(10);

        manager.add_connection("session1".to_string(), tx);

        let user_id = Snowflake::from(12345i64);
        assert!(manager.authenticate_connection("session1", user_id).await);
        assert_eq!(manager.user_count(), 1);

        let connections = manager.get_user_connections(user_id);
        assert_eq!(connections.len(), 1);
    }

    #[tokio::test]
    async fn test_guild_subscriptions() {
        let manager = ConnectionManager::new();
        let (tx, _rx) = mpsc::channel(10);

        manager.add_connection("session1".to_string(), tx);

        let guild_id = Snowflake::from(67890i64);
        assert!(manager.subscribe_to_guild("session1", guild_id).await);
        assert_eq!(manager.guild_count(), 1);

        let connections = manager.get_guild_connections(guild_id);
        assert_eq!(connections.len(), 1);

        assert!(manager.unsubscribe_from_guild("session1", guild_id).await);
        let connections = manager.get_guild_connections(guild_id);
        assert_eq!(connections.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_user_connections() {
        let manager = ConnectionManager::new();
        let (tx1, _rx1) = mpsc::channel(10);
        let (tx2, _rx2) = mpsc::channel(10);

        manager.add_connection("session1".to_string(), tx1);
        manager.add_connection("session2".to_string(), tx2);

        let user_id = Snowflake::from(12345i64);
        manager.authenticate_connection("session1", user_id).await;
        manager.authenticate_connection("session2", user_id).await;

        let connections = manager.get_user_connections(user_id);
        assert_eq!(connections.len(), 2);
        assert_eq!(manager.user_count(), 1);
    }
}
