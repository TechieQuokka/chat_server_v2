//! Redis Pub/Sub publisher.
//!
//! Publishes events to Redis channels for distribution to WebSocket clients.

use crate::pool::{RedisPool, RedisResult};
use crate::pubsub::PubSubChannel;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Event wrapper for Pub/Sub messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubEvent {
    /// Event type name (e.g., "MESSAGE_CREATE", "PRESENCE_UPDATE")
    pub event_type: String,
    /// Event payload
    pub data: serde_json::Value,
    /// Optional target information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<EventTarget>,
}

/// Target information for event routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTarget {
    /// Guild ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    /// Channel ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// User IDs to exclude from receiving this event
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub exclude_users: Vec<String>,
}

impl PubSubEvent {
    /// Create a new event
    #[must_use]
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            event_type: event_type.into(),
            data,
            target: None,
        }
    }

    /// Add target information
    #[must_use]
    pub fn with_target(mut self, target: EventTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl EventTarget {
    /// Create an empty target
    #[must_use]
    pub fn empty() -> Self {
        Self {
            guild_id: None,
            channel_id: None,
            exclude_users: Vec::new(),
        }
    }

    /// Set guild ID
    #[must_use]
    pub fn with_guild(mut self, guild_id: impl Into<String>) -> Self {
        self.guild_id = Some(guild_id.into());
        self
    }

    /// Set channel ID
    #[must_use]
    pub fn with_channel(mut self, channel_id: impl Into<String>) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Add user to exclude list
    #[must_use]
    pub fn exclude_user(mut self, user_id: impl Into<String>) -> Self {
        self.exclude_users.push(user_id.into());
        self
    }
}

/// Redis Pub/Sub publisher
#[derive(Clone)]
pub struct Publisher {
    pool: RedisPool,
}

impl Publisher {
    /// Create a new publisher
    #[must_use]
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Publish an event to a channel
    pub async fn publish(&self, channel: &PubSubChannel, event: &PubSubEvent) -> RedisResult<u32> {
        let mut conn = self.pool.get().await?;
        let channel_name = channel.name();
        let payload = event.to_json()?;

        let receivers: u32 = conn.publish(&channel_name, &payload).await?;

        tracing::debug!(
            channel = %channel_name,
            event_type = %event.event_type,
            receivers = receivers,
            "Published event"
        );

        Ok(receivers)
    }

    /// Publish a raw message to a channel
    pub async fn publish_raw(&self, channel: &PubSubChannel, message: &str) -> RedisResult<u32> {
        let mut conn = self.pool.get().await?;
        let channel_name = channel.name();

        let receivers: u32 = conn.publish(&channel_name, message).await?;

        tracing::debug!(
            channel = %channel_name,
            receivers = receivers,
            "Published raw message"
        );

        Ok(receivers)
    }

    /// Publish to multiple channels
    pub async fn publish_many(
        &self,
        channels: &[PubSubChannel],
        event: &PubSubEvent,
    ) -> RedisResult<u32> {
        let payload = event.to_json()?;
        let mut total_receivers = 0;
        let mut conn = self.pool.get().await?;

        for channel in channels {
            let channel_name = channel.name();
            let receivers: u32 = conn.publish(&channel_name, &payload).await?;
            total_receivers += receivers;
        }

        tracing::debug!(
            channels = channels.len(),
            event_type = %event.event_type,
            total_receivers = total_receivers,
            "Published event to multiple channels"
        );

        Ok(total_receivers)
    }
}

/// Convenience methods for common event types
impl Publisher {
    /// Publish a message create event
    pub async fn publish_message_create(
        &self,
        channel_id: chat_core::Snowflake,
        guild_id: Option<chat_core::Snowflake>,
        message_data: serde_json::Value,
    ) -> RedisResult<u32> {
        let event = PubSubEvent::new("MESSAGE_CREATE", message_data);

        // Publish to channel
        let channel = PubSubChannel::channel(channel_id);
        let mut receivers = self.publish(&channel, &event).await?;

        // Also publish to guild if applicable
        if let Some(gid) = guild_id {
            let guild_channel = PubSubChannel::guild(gid);
            receivers += self.publish(&guild_channel, &event).await?;
        }

        Ok(receivers)
    }

    /// Publish a presence update event
    pub async fn publish_presence_update(
        &self,
        guild_ids: &[chat_core::Snowflake],
        presence_data: serde_json::Value,
    ) -> RedisResult<u32> {
        let event = PubSubEvent::new("PRESENCE_UPDATE", presence_data);

        let channels: Vec<PubSubChannel> = guild_ids.iter().map(|&id| PubSubChannel::guild(id)).collect();
        self.publish_many(&channels, &event).await
    }

    /// Publish a typing start event
    pub async fn publish_typing_start(
        &self,
        channel_id: chat_core::Snowflake,
        typing_data: serde_json::Value,
    ) -> RedisResult<u32> {
        let event = PubSubEvent::new("TYPING_START", typing_data);
        let channel = PubSubChannel::channel(channel_id);
        self.publish(&channel, &event).await
    }

    /// Publish a guild member event
    pub async fn publish_member_event(
        &self,
        event_type: &str,
        guild_id: chat_core::Snowflake,
        member_data: serde_json::Value,
    ) -> RedisResult<u32> {
        let event = PubSubEvent::new(event_type, member_data);
        let channel = PubSubChannel::guild(guild_id);
        self.publish(&channel, &event).await
    }

    /// Publish a user-specific event
    pub async fn publish_to_user(
        &self,
        user_id: chat_core::Snowflake,
        event_type: &str,
        data: serde_json::Value,
    ) -> RedisResult<u32> {
        let event = PubSubEvent::new(event_type, data);
        let channel = PubSubChannel::user(user_id);
        self.publish(&channel, &event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubsub_event_creation() {
        let data = serde_json::json!({
            "id": "12345",
            "content": "Hello!"
        });

        let event = PubSubEvent::new("MESSAGE_CREATE", data.clone());
        assert_eq!(event.event_type, "MESSAGE_CREATE");
        assert_eq!(event.data, data);
        assert!(event.target.is_none());
    }

    #[test]
    fn test_pubsub_event_with_target() {
        let data = serde_json::json!({"id": "12345"});
        let target = EventTarget::empty()
            .with_guild("111")
            .with_channel("222")
            .exclude_user("333");

        let event = PubSubEvent::new("MESSAGE_CREATE", data).with_target(target);

        assert!(event.target.is_some());
        let target = event.target.unwrap();
        assert_eq!(target.guild_id, Some("111".to_string()));
        assert_eq!(target.channel_id, Some("222".to_string()));
        assert_eq!(target.exclude_users, vec!["333".to_string()]);
    }

    #[test]
    fn test_event_serialization() {
        let data = serde_json::json!({"content": "test"});
        let event = PubSubEvent::new("TEST_EVENT", data);

        let json = event.to_json().unwrap();
        assert!(json.contains("TEST_EVENT"));
        assert!(json.contains("test"));
    }
}
