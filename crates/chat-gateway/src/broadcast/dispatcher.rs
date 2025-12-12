//! Event dispatcher
//!
//! Receives events from Redis Pub/Sub and dispatches them to WebSocket connections.

use crate::connection::ConnectionManager;
use crate::protocol::GatewayMessage;
use chat_cache::{PubSubChannel, ReceivedMessage, Subscriber, SubscriberBuilder};
use chat_core::Snowflake;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Configuration for the event dispatcher
#[derive(Debug, Clone)]
pub struct EventDispatcherConfig {
    /// Redis URL
    pub redis_url: String,
    /// Broadcast buffer size
    pub broadcast_buffer: usize,
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for EventDispatcherConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            broadcast_buffer: 1024,
            reconnect_delay_ms: 1000,
        }
    }
}

/// Event dispatcher that routes Redis Pub/Sub messages to WebSocket connections
pub struct EventDispatcher {
    /// Connection manager for sending messages
    connection_manager: Arc<ConnectionManager>,
    /// Redis subscriber
    subscriber: Subscriber,
    /// Whether the dispatcher is running
    running: Arc<AtomicBool>,
    /// Sequence number for events
    sequence: Arc<AtomicU64>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub async fn new(
        config: EventDispatcherConfig,
        connection_manager: Arc<ConnectionManager>,
    ) -> Result<Self, chat_cache::SubscriberError> {
        let subscriber = SubscriberBuilder::new()
            .redis_url(&config.redis_url)
            .broadcast_buffer(config.broadcast_buffer)
            .reconnect_delay_ms(config.reconnect_delay_ms)
            // Subscribe to broadcast channel by default
            .subscribe(PubSubChannel::broadcast())
            .build()
            .await?;

        Ok(Self {
            connection_manager,
            subscriber,
            running: Arc::new(AtomicBool::new(false)),
            sequence: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Subscribe to a guild's events
    pub async fn subscribe_guild(
        &self,
        guild_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .subscribe(&[PubSubChannel::guild(guild_id)])
            .await
    }

    /// Unsubscribe from a guild's events
    pub async fn unsubscribe_guild(
        &self,
        guild_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .unsubscribe(&[PubSubChannel::guild(guild_id)])
            .await
    }

    /// Subscribe to a channel's events
    pub async fn subscribe_channel(
        &self,
        channel_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .subscribe(&[PubSubChannel::channel(channel_id)])
            .await
    }

    /// Unsubscribe from a channel's events
    pub async fn unsubscribe_channel(
        &self,
        channel_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .unsubscribe(&[PubSubChannel::channel(channel_id)])
            .await
    }

    /// Subscribe to a user's events
    pub async fn subscribe_user(
        &self,
        user_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .subscribe(&[PubSubChannel::user(user_id)])
            .await
    }

    /// Unsubscribe from a user's events
    pub async fn unsubscribe_user(
        &self,
        user_id: Snowflake,
    ) -> Result<(), chat_cache::SubscriberError> {
        self.subscriber
            .unsubscribe(&[PubSubChannel::user(user_id)])
            .await
    }

    /// Get the next sequence number
    fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Start the event dispatcher
    ///
    /// This spawns a background task that receives messages from Redis
    /// and dispatches them to appropriate WebSocket connections.
    pub fn start(self: Arc<Self>) {
        if self.running.swap(true, Ordering::SeqCst) {
            tracing::warn!("Event dispatcher is already running");
            return;
        }

        let dispatcher = self.clone();
        tokio::spawn(async move {
            dispatcher.run().await;
        });

        tracing::info!("Event dispatcher started");
    }

    /// Stop the event dispatcher
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.subscriber.shutdown().await.ok();
        tracing::info!("Event dispatcher stopped");
    }

    /// Run the event dispatcher loop
    async fn run(&self) {
        let mut receiver = self.subscriber.receiver();

        while self.running.load(Ordering::SeqCst) {
            match receiver.recv().await {
                Ok(msg) => {
                    self.handle_message(msg).await;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(lagged = n, "Event dispatcher lagged behind");
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::warn!("Event dispatcher channel closed");
                    break;
                }
            }
        }

        self.running.store(false, Ordering::SeqCst);
        tracing::info!("Event dispatcher loop ended");
    }

    /// Handle a received message from Redis
    async fn handle_message(&self, msg: ReceivedMessage) {
        // Parse the event
        let event = match &msg.event {
            Some(e) => e,
            None => {
                tracing::debug!(
                    channel = ?msg.channel,
                    "Received non-event message, ignoring"
                );
                return;
            }
        };

        let event_type = &event.event_type;
        let data = &event.data;

        tracing::trace!(
            channel = ?msg.channel,
            event_type = %event_type,
            "Dispatching event"
        );

        // Create the gateway message
        let seq = self.next_sequence();
        let gateway_msg = GatewayMessage::dispatch(event_type, seq, data.clone());

        // Route based on channel type
        match &msg.channel {
            PubSubChannel::Guild(guild_id) => {
                // Send to all connections subscribed to this guild
                let exclude_user = event.target.as_ref().and_then(|t| {
                    t.exclude_users
                        .first()
                        .and_then(|u| u.parse::<i64>().ok())
                        .map(Snowflake::from)
                });

                let sent = self
                    .connection_manager
                    .send_to_guild(*guild_id, gateway_msg, exclude_user)
                    .await;

                tracing::trace!(
                    guild_id = %guild_id,
                    event_type = %event_type,
                    sent = sent,
                    "Event dispatched to guild"
                );
            }
            PubSubChannel::Channel(channel_id) => {
                // For channel-specific events, we need to find which guild this belongs to
                // For now, we'll broadcast to all connections (can be optimized later)
                // In production, you'd look up the guild from the channel
                tracing::trace!(
                    channel_id = %channel_id,
                    event_type = %event_type,
                    "Channel event received (routing via guild)"
                );
            }
            PubSubChannel::User(user_id) => {
                // Send to all connections of this user
                let sent = self
                    .connection_manager
                    .send_to_user(*user_id, gateway_msg)
                    .await;

                tracing::trace!(
                    user_id = %user_id,
                    event_type = %event_type,
                    sent = sent,
                    "Event dispatched to user"
                );
            }
            PubSubChannel::Broadcast => {
                // Send to all connections
                let sent = self.connection_manager.broadcast(gateway_msg).await;

                tracing::trace!(
                    event_type = %event_type,
                    sent = sent,
                    "Event broadcast to all"
                );
            }
            PubSubChannel::Custom(name) => {
                tracing::debug!(
                    channel = %name,
                    event_type = %event_type,
                    "Received event on custom channel, ignoring"
                );
            }
        }
    }

    /// Check if the dispatcher is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_config_default() {
        let config = EventDispatcherConfig::default();
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.broadcast_buffer, 1024);
        assert_eq!(config.reconnect_delay_ms, 1000);
    }
}
