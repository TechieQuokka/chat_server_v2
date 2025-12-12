//! WebSocket session storage in Redis.
//!
//! Manages WebSocket sessions for real-time communication, including:
//! - Session state persistence
//! - Session resume capability (2-minute window after disconnect)
//! - Event queue for replay on resume

use crate::pool::{RedisPool, RedisResult};
use chat_core::Snowflake;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Key prefix for WebSocket sessions
const WS_SESSION_PREFIX: &str = "ws_session:";
/// Key prefix for session event queues
const WS_EVENTS_PREFIX: &str = "ws_events:";
/// Key prefix for user-to-sessions mapping
const USER_SESSIONS_PREFIX: &str = "user_ws_sessions:";

/// Default TTL for disconnected sessions (2 minutes for resume)
const SESSION_RESUME_TTL: u64 = 120;
/// Maximum events to store for resume
const MAX_RESUME_EVENTS: usize = 1000;

/// WebSocket session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is connected and active
    Connected,
    /// Session is disconnected but resumable
    Disconnected,
    /// Session is invalid and cannot be resumed
    Invalid,
}

/// Stored WebSocket session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSessionData {
    /// Session ID (unique identifier for this connection)
    pub session_id: String,
    /// User ID this session belongs to
    pub user_id: Snowflake,
    /// Current sequence number (for event ordering)
    pub sequence: u64,
    /// Guilds this session is subscribed to
    pub guilds: Vec<Snowflake>,
    /// Session creation timestamp (Unix epoch seconds)
    pub created_at: i64,
    /// Last activity timestamp (Unix epoch seconds)
    pub last_active_at: i64,
    /// Current session state
    pub state: SessionState,
    /// Resume gateway URL hint
    pub resume_url: Option<String>,
    /// Client properties (os, browser, device)
    pub properties: Option<ClientProperties>,
}

/// Client connection properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProperties {
    pub os: Option<String>,
    pub browser: Option<String>,
    pub device: Option<String>,
}

impl WebSocketSessionData {
    /// Create a new session
    #[must_use]
    pub fn new(session_id: String, user_id: Snowflake) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            session_id,
            user_id,
            sequence: 0,
            guilds: Vec::new(),
            created_at: now,
            last_active_at: now,
            state: SessionState::Connected,
            resume_url: None,
            properties: None,
        }
    }

    /// Set client properties
    #[must_use]
    pub fn with_properties(mut self, properties: ClientProperties) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set resume URL
    #[must_use]
    pub fn with_resume_url(mut self, url: impl Into<String>) -> Self {
        self.resume_url = Some(url.into());
        self
    }

    /// Add guild subscription
    pub fn add_guild(&mut self, guild_id: Snowflake) {
        if !self.guilds.contains(&guild_id) {
            self.guilds.push(guild_id);
        }
    }

    /// Remove guild subscription
    pub fn remove_guild(&mut self, guild_id: Snowflake) {
        self.guilds.retain(|g| *g != guild_id);
    }

    /// Increment and return the next sequence number
    pub fn next_sequence(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_active_at = chrono::Utc::now().timestamp();
    }

    /// Check if session is resumable
    #[must_use]
    pub fn is_resumable(&self) -> bool {
        self.state == SessionState::Disconnected
    }
}

/// Event stored for session resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Event sequence number
    pub sequence: u64,
    /// Event type name (e.g., "MESSAGE_CREATE")
    pub event_type: String,
    /// Event payload as JSON
    pub data: serde_json::Value,
    /// Event timestamp
    pub timestamp: i64,
}

/// WebSocket session store
#[derive(Clone)]
pub struct WebSocketSessionStore {
    pool: RedisPool,
}

impl WebSocketSessionStore {
    /// Create a new session store
    #[must_use]
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Generate Redis key for a session
    fn session_key(session_id: &str) -> String {
        format!("{WS_SESSION_PREFIX}{session_id}")
    }

    /// Generate Redis key for session events
    fn events_key(session_id: &str) -> String {
        format!("{WS_EVENTS_PREFIX}{session_id}")
    }

    /// Generate Redis key for user sessions
    fn user_sessions_key(user_id: Snowflake) -> String {
        format!("{USER_SESSIONS_PREFIX}{user_id}")
    }

    /// Create a new session
    pub async fn create(&self, session: &WebSocketSessionData) -> RedisResult<()> {
        let key = Self::session_key(&session.session_id);
        // Connected sessions don't expire
        self.pool.set(&key, session, None).await?;

        // Add to user's sessions set
        let user_key = Self::user_sessions_key(session.user_id);
        let mut conn = self.pool.get().await?;
        conn.sadd::<_, _, ()>(&user_key, &session.session_id).await?;

        tracing::debug!(
            session_id = %session.session_id,
            user_id = %session.user_id,
            "Created WebSocket session"
        );

        Ok(())
    }

    /// Get session by ID
    pub async fn get(&self, session_id: &str) -> RedisResult<Option<WebSocketSessionData>> {
        let key = Self::session_key(session_id);
        self.pool.get_value(&key).await
    }

    /// Update session
    pub async fn update(&self, session: &WebSocketSessionData) -> RedisResult<()> {
        let key = Self::session_key(&session.session_id);
        let ttl = if session.state == SessionState::Disconnected {
            Some(SESSION_RESUME_TTL)
        } else {
            None
        };
        self.pool.set(&key, session, ttl).await
    }

    /// Update session sequence number
    pub async fn update_sequence(&self, session_id: &str, sequence: u64) -> RedisResult<bool> {
        if let Some(mut session) = self.get(session_id).await? {
            session.sequence = sequence;
            session.touch();
            self.update(&session).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Mark session as disconnected (starts resume TTL)
    pub async fn mark_disconnected(&self, session_id: &str) -> RedisResult<bool> {
        if let Some(mut session) = self.get(session_id).await? {
            session.state = SessionState::Disconnected;
            session.touch();

            let key = Self::session_key(session_id);
            self.pool.set(&key, &session, Some(SESSION_RESUME_TTL)).await?;

            // Also set TTL on events queue
            let events_key = Self::events_key(session_id);
            self.pool.expire(&events_key, SESSION_RESUME_TTL).await?;

            tracing::debug!(
                session_id = %session_id,
                ttl = SESSION_RESUME_TTL,
                "Marked session as disconnected"
            );

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Mark session as connected (removes resume TTL)
    pub async fn mark_connected(&self, session_id: &str) -> RedisResult<bool> {
        if let Some(mut session) = self.get(session_id).await? {
            session.state = SessionState::Connected;
            session.touch();

            // Remove TTL by setting without expiration
            let key = Self::session_key(session_id);
            self.pool.set(&key, &session, None).await?;

            tracing::debug!(
                session_id = %session_id,
                "Marked session as connected"
            );

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a session
    pub async fn delete(&self, session_id: &str) -> RedisResult<bool> {
        // Get session to find user_id
        if let Some(session) = self.get(session_id).await? {
            // Remove from user's sessions set
            let user_key = Self::user_sessions_key(session.user_id);
            let mut conn = self.pool.get().await?;
            conn.srem::<_, _, ()>(&user_key, session_id).await?;
        }

        let key = Self::session_key(session_id);
        let events_key = Self::events_key(session_id);

        // Delete session and events
        self.pool.delete(&key).await?;
        self.pool.delete(&events_key).await?;

        tracing::debug!(session_id = %session_id, "Deleted WebSocket session");

        Ok(true)
    }

    /// Delete all sessions for a user
    pub async fn delete_all_for_user(&self, user_id: Snowflake) -> RedisResult<u32> {
        let user_key = Self::user_sessions_key(user_id);
        let mut conn = self.pool.get().await?;

        let session_ids: Vec<String> = conn.smembers(&user_key).await?;
        let count = session_ids.len() as u32;

        for session_id in &session_ids {
            let key = Self::session_key(session_id);
            let events_key = Self::events_key(session_id);
            self.pool.delete(&key).await?;
            self.pool.delete(&events_key).await?;
        }

        // Delete user's sessions set
        conn.del::<_, ()>(&user_key).await?;

        tracing::info!(
            user_id = %user_id,
            count = count,
            "Deleted all WebSocket sessions for user"
        );

        Ok(count)
    }

    /// Get all session IDs for a user
    pub async fn get_user_sessions(&self, user_id: Snowflake) -> RedisResult<Vec<String>> {
        let user_key = Self::user_sessions_key(user_id);
        let mut conn = self.pool.get().await?;
        let sessions: Vec<String> = conn.smembers(&user_key).await?;
        Ok(sessions)
    }

    /// Queue an event for potential session resume
    pub async fn queue_event(&self, session_id: &str, event: &SessionEvent) -> RedisResult<()> {
        let key = Self::events_key(session_id);
        let mut conn = self.pool.get().await?;

        let serialized = serde_json::to_string(event)?;
        conn.lpush::<_, _, ()>(&key, &serialized).await?;

        // Trim to max length
        conn.ltrim::<_, ()>(&key, 0, (MAX_RESUME_EVENTS - 1) as isize).await?;

        Ok(())
    }

    /// Get queued events for session resume (returns events after given sequence)
    pub async fn get_events_since(
        &self,
        session_id: &str,
        since_sequence: u64,
    ) -> RedisResult<Vec<SessionEvent>> {
        let key = Self::events_key(session_id);
        let mut conn = self.pool.get().await?;

        let events: Vec<String> = conn.lrange(&key, 0, -1).await?;
        let mut result = Vec::new();

        // Events are stored newest-first, so reverse and filter
        for event_str in events.into_iter().rev() {
            if let Ok(event) = serde_json::from_str::<SessionEvent>(&event_str) {
                if event.sequence > since_sequence {
                    result.push(event);
                }
            }
        }

        Ok(result)
    }

    /// Clear event queue for a session
    pub async fn clear_events(&self, session_id: &str) -> RedisResult<()> {
        let key = Self::events_key(session_id);
        self.pool.delete(&key).await?;
        Ok(())
    }

    /// Validate session for resume (checks state and returns session if valid)
    pub async fn validate_for_resume(
        &self,
        session_id: &str,
        user_id: Snowflake,
    ) -> RedisResult<Option<WebSocketSessionData>> {
        if let Some(session) = self.get(session_id).await? {
            // Verify user matches and session is resumable
            if session.user_id == user_id && session.is_resumable() {
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    /// Add guild to session's subscriptions
    pub async fn add_guild(&self, session_id: &str, guild_id: Snowflake) -> RedisResult<bool> {
        if let Some(mut session) = self.get(session_id).await? {
            session.add_guild(guild_id);
            self.update(&session).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Remove guild from session's subscriptions
    pub async fn remove_guild(&self, session_id: &str, guild_id: Snowflake) -> RedisResult<bool> {
        if let Some(mut session) = self.get(session_id).await? {
            session.remove_guild(guild_id);
            self.update(&session).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all sessions subscribed to a guild
    ///
    /// Note: This implementation scans all sessions. For production with many sessions,
    /// consider maintaining a guild-to-sessions index (guild_sessions:{guild_id} set).
    pub async fn get_guild_sessions(&self, guild_id: Snowflake) -> RedisResult<Vec<String>> {
        let pattern = format!("{WS_SESSION_PREFIX}*");
        let keys = self.pool.scan_keys(&pattern, 100).await?;
        let mut sessions = Vec::new();

        for key in keys {
            if let Some(session_id) = key.strip_prefix(WS_SESSION_PREFIX) {
                if let Some(session) = self.get(session_id).await? {
                    if session.guilds.contains(&guild_id) && session.state == SessionState::Connected {
                        sessions.push(session_id.to_string());
                    }
                }
            }
        }

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_creation() {
        let user_id = Snowflake::from(12345i64);
        let session = WebSocketSessionData::new("session123".to_string(), user_id)
            .with_properties(ClientProperties {
                os: Some("Windows".to_string()),
                browser: Some("Chrome".to_string()),
                device: Some("desktop".to_string()),
            })
            .with_resume_url("wss://gateway.example.com");

        assert_eq!(session.session_id, "session123");
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.sequence, 0);
        assert!(session.guilds.is_empty());
        assert_eq!(session.state, SessionState::Connected);
        assert!(session.properties.is_some());
        assert!(session.resume_url.is_some());
    }

    #[test]
    fn test_session_guild_management() {
        let user_id = Snowflake::from(12345i64);
        let mut session = WebSocketSessionData::new("session123".to_string(), user_id);

        let guild_id = Snowflake::from(67890i64);
        session.add_guild(guild_id);
        assert!(session.guilds.contains(&guild_id));

        // Adding same guild twice should not duplicate
        session.add_guild(guild_id);
        assert_eq!(session.guilds.len(), 1);

        session.remove_guild(guild_id);
        assert!(!session.guilds.contains(&guild_id));
    }

    #[test]
    fn test_session_sequence() {
        let user_id = Snowflake::from(12345i64);
        let mut session = WebSocketSessionData::new("session123".to_string(), user_id);

        assert_eq!(session.next_sequence(), 1);
        assert_eq!(session.next_sequence(), 2);
        assert_eq!(session.sequence, 2);
    }

    #[test]
    fn test_key_generation() {
        assert_eq!(
            WebSocketSessionStore::session_key("abc123"),
            "ws_session:abc123"
        );
        assert_eq!(
            WebSocketSessionStore::events_key("abc123"),
            "ws_events:abc123"
        );
    }
}
