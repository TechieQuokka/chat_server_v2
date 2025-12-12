//! Session management
//!
//! Handles session lifecycle and resume capability.

use chat_cache::{ClientProperties, SessionEvent, SessionState, WebSocketSessionData, WebSocketSessionStore};
use chat_core::Snowflake;

/// Session helper for managing WebSocket sessions
pub struct Session;

impl Session {
    /// Generate a new session ID
    #[must_use]
    pub fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Create a new session in the store
    pub async fn create(
        store: &WebSocketSessionStore,
        session_id: &str,
        user_id: Snowflake,
        properties: Option<ClientProperties>,
        resume_url: Option<String>,
    ) -> Result<WebSocketSessionData, chat_cache::RedisPoolError> {
        let mut session = WebSocketSessionData::new(session_id.to_string(), user_id);

        if let Some(props) = properties {
            session = session.with_properties(props);
        }

        if let Some(url) = resume_url {
            session = session.with_resume_url(url);
        }

        store.create(&session).await?;

        tracing::info!(
            session_id = %session_id,
            user_id = %user_id,
            "Created new WebSocket session"
        );

        Ok(session)
    }

    /// Mark session as disconnected (starts 2-minute resume window)
    pub async fn disconnect(
        store: &WebSocketSessionStore,
        session_id: &str,
    ) -> Result<bool, chat_cache::RedisPoolError> {
        let result = store.mark_disconnected(session_id).await?;

        if result {
            tracing::info!(
                session_id = %session_id,
                "Session marked as disconnected (2min resume window)"
            );
        }

        Ok(result)
    }

    /// Attempt to resume a session
    ///
    /// Returns the session if valid and resumable, None otherwise.
    pub async fn resume(
        store: &WebSocketSessionStore,
        session_id: &str,
        user_id: Snowflake,
        last_sequence: u64,
    ) -> Result<Option<(WebSocketSessionData, Vec<SessionEvent>)>, chat_cache::RedisPoolError> {
        // Validate session belongs to user and is resumable
        let session = match store.validate_for_resume(session_id, user_id).await? {
            Some(s) => s,
            None => {
                tracing::debug!(
                    session_id = %session_id,
                    user_id = %user_id,
                    "Session not found or not resumable"
                );
                return Ok(None);
            }
        };

        // Get missed events
        let events = store.get_events_since(session_id, last_sequence).await?;

        // Mark session as connected again
        store.mark_connected(session_id).await?;

        tracing::info!(
            session_id = %session_id,
            user_id = %user_id,
            missed_events = events.len(),
            "Session resumed successfully"
        );

        Ok(Some((session, events)))
    }

    /// Delete a session
    pub async fn delete(
        store: &WebSocketSessionStore,
        session_id: &str,
    ) -> Result<(), chat_cache::RedisPoolError> {
        store.delete(session_id).await?;

        tracing::debug!(session_id = %session_id, "Session deleted");

        Ok(())
    }

    /// Queue an event for potential resume
    pub async fn queue_event(
        store: &WebSocketSessionStore,
        session_id: &str,
        sequence: u64,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<(), chat_cache::RedisPoolError> {
        let event = SessionEvent {
            sequence,
            event_type: event_type.to_string(),
            data,
            timestamp: chrono::Utc::now().timestamp(),
        };

        store.queue_event(session_id, &event).await?;

        Ok(())
    }

    /// Update session sequence number
    pub async fn update_sequence(
        store: &WebSocketSessionStore,
        session_id: &str,
        sequence: u64,
    ) -> Result<(), chat_cache::RedisPoolError> {
        store.update_sequence(session_id, sequence).await?;
        Ok(())
    }

    /// Add guild to session subscriptions
    pub async fn subscribe_guild(
        store: &WebSocketSessionStore,
        session_id: &str,
        guild_id: Snowflake,
    ) -> Result<(), chat_cache::RedisPoolError> {
        store.add_guild(session_id, guild_id).await?;
        Ok(())
    }

    /// Remove guild from session subscriptions
    pub async fn unsubscribe_guild(
        store: &WebSocketSessionStore,
        session_id: &str,
        guild_id: Snowflake,
    ) -> Result<(), chat_cache::RedisPoolError> {
        store.remove_guild(session_id, guild_id).await?;
        Ok(())
    }

    /// Get session state
    pub async fn get_state(
        store: &WebSocketSessionStore,
        session_id: &str,
    ) -> Result<Option<SessionState>, chat_cache::RedisPoolError> {
        if let Some(session) = store.get(session_id).await? {
            Ok(Some(session.state))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_id() {
        let id1 = Session::generate_id();
        let id2 = Session::generate_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID format
    }
}
