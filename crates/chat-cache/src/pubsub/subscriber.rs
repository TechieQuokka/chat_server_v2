//! Redis Pub/Sub subscriber.
//!
//! Subscribes to Redis channels and receives events for WebSocket distribution.

use crate::pubsub::{PubSubChannel, PubSubEvent};
use futures_util::StreamExt;
use redis::Client;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Error type for subscriber operations
#[derive(Debug, thiserror::Error)]
pub enum SubscriberError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Failed to parse event: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Connection error: {0}")]
    Connection(String),
}

/// Result type for subscriber operations
pub type SubscriberResult<T> = Result<T, SubscriberError>;

/// Received message from Pub/Sub
#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    /// Channel the message was received on
    pub channel: PubSubChannel,
    /// Parsed event (if valid JSON)
    pub event: Option<PubSubEvent>,
    /// Raw payload
    pub payload: String,
}

impl ReceivedMessage {
    /// Create from raw Redis message
    fn from_redis(channel_name: String, payload: String) -> Self {
        let channel = PubSubChannel::parse(&channel_name);
        let event = serde_json::from_str(&payload).ok();

        Self {
            channel,
            event,
            payload,
        }
    }
}

/// Subscriber configuration
#[derive(Debug, Clone)]
pub struct SubscriberConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Channel buffer size for broadcast
    pub broadcast_buffer: usize,
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            broadcast_buffer: 1024,
            reconnect_delay_ms: 1000,
        }
    }
}

/// Redis Pub/Sub subscriber
pub struct Subscriber {
    #[allow(dead_code)]
    config: SubscriberConfig,
    /// Currently subscribed channels
    subscribed: Arc<RwLock<HashSet<String>>>,
    /// Broadcast sender for messages
    broadcast_tx: broadcast::Sender<ReceivedMessage>,
    /// Control channel for subscription management
    control_tx: mpsc::Sender<SubscriberCommand>,
}

/// Commands for subscription management
#[derive(Debug)]
enum SubscriberCommand {
    Subscribe(Vec<String>),
    Unsubscribe(Vec<String>),
    Shutdown,
}

impl Subscriber {
    /// Create a new subscriber and start the background listener
    pub async fn new(config: SubscriberConfig) -> SubscriberResult<Self> {
        let (broadcast_tx, _) = broadcast::channel(config.broadcast_buffer);
        let (control_tx, control_rx) = mpsc::channel(32);
        let subscribed = Arc::new(RwLock::new(HashSet::new()));

        let subscriber = Self {
            config: config.clone(),
            subscribed: subscribed.clone(),
            broadcast_tx: broadcast_tx.clone(),
            control_tx,
        };

        // Start background listener
        tokio::spawn(Self::listener_loop(
            config,
            subscribed,
            broadcast_tx,
            control_rx,
        ));

        Ok(subscriber)
    }

    /// Background listener loop
    async fn listener_loop(
        config: SubscriberConfig,
        subscribed: Arc<RwLock<HashSet<String>>>,
        broadcast_tx: broadcast::Sender<ReceivedMessage>,
        mut control_rx: mpsc::Receiver<SubscriberCommand>,
    ) {
        loop {
            match Self::run_listener(&config, &subscribed, &broadcast_tx, &mut control_rx).await {
                Ok(should_stop) => {
                    if should_stop {
                        tracing::info!("Subscriber shutting down");
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Subscriber error, reconnecting...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        config.reconnect_delay_ms,
                    ))
                    .await;
                }
            }
        }
    }

    /// Run the listener until error or shutdown
    async fn run_listener(
        config: &SubscriberConfig,
        subscribed: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<ReceivedMessage>,
        control_rx: &mut mpsc::Receiver<SubscriberCommand>,
    ) -> SubscriberResult<bool> {
        let client = Client::open(config.redis_url.as_str())?;
        let mut pubsub = client.get_async_pubsub().await?;

        // Subscribe to existing channels
        {
            let channels = subscribed.read().await;
            for channel in channels.iter() {
                pubsub.subscribe(channel).await?;
            }
        }

        tracing::info!("Subscriber connected to Redis");

        let mut stream = pubsub.on_message();

        loop {
            tokio::select! {
                // Handle incoming messages
                msg = stream.next() => {
                    match msg {
                        Some(msg) => {
                            let channel_name: String = msg.get_channel_name().to_string();
                            let payload: String = msg.get_payload().unwrap_or_default();

                            let received = ReceivedMessage::from_redis(channel_name.clone(), payload);

                            // Broadcast to all receivers (ignore send errors - no receivers)
                            let _ = broadcast_tx.send(received);

                            tracing::trace!(
                                channel = %channel_name,
                                "Received Pub/Sub message"
                            );
                        }
                        None => {
                            tracing::warn!("Pub/Sub stream ended");
                            return Ok(false);
                        }
                    }
                }

                // Handle control commands
                cmd = control_rx.recv() => {
                    match cmd {
                        Some(SubscriberCommand::Subscribe(channels)) => {
                            // Need to drop stream to access pubsub
                            drop(stream);
                            for channel in &channels {
                                if let Err(e) = pubsub.subscribe(channel).await {
                                    tracing::error!(channel = %channel, error = %e, "Failed to subscribe");
                                } else {
                                    subscribed.write().await.insert(channel.clone());
                                    tracing::debug!(channel = %channel, "Subscribed to channel");
                                }
                            }
                            stream = pubsub.on_message();
                        }
                        Some(SubscriberCommand::Unsubscribe(channels)) => {
                            drop(stream);
                            for channel in &channels {
                                if let Err(e) = pubsub.unsubscribe(channel).await {
                                    tracing::error!(channel = %channel, error = %e, "Failed to unsubscribe");
                                } else {
                                    subscribed.write().await.remove(channel);
                                    tracing::debug!(channel = %channel, "Unsubscribed from channel");
                                }
                            }
                            stream = pubsub.on_message();
                        }
                        Some(SubscriberCommand::Shutdown) => {
                            return Ok(true);
                        }
                        None => {
                            tracing::warn!("Control channel closed");
                            return Ok(true);
                        }
                    }
                }
            }
        }
    }

    /// Subscribe to channels
    pub async fn subscribe(&self, channels: &[PubSubChannel]) -> SubscriberResult<()> {
        let channel_names: Vec<String> = channels.iter().map(PubSubChannel::name).collect();

        self.control_tx
            .send(SubscriberCommand::Subscribe(channel_names))
            .await
            .map_err(|_| SubscriberError::ChannelClosed)
    }

    /// Unsubscribe from channels
    pub async fn unsubscribe(&self, channels: &[PubSubChannel]) -> SubscriberResult<()> {
        let channel_names: Vec<String> = channels.iter().map(PubSubChannel::name).collect();

        self.control_tx
            .send(SubscriberCommand::Unsubscribe(channel_names))
            .await
            .map_err(|_| SubscriberError::ChannelClosed)
    }

    /// Get a receiver for broadcast messages
    #[must_use]
    pub fn receiver(&self) -> broadcast::Receiver<ReceivedMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Get currently subscribed channels
    pub async fn subscribed_channels(&self) -> Vec<String> {
        self.subscribed.read().await.iter().cloned().collect()
    }

    /// Shutdown the subscriber
    pub async fn shutdown(&self) -> SubscriberResult<()> {
        self.control_tx
            .send(SubscriberCommand::Shutdown)
            .await
            .map_err(|_| SubscriberError::ChannelClosed)
    }
}

/// Builder for subscriber
pub struct SubscriberBuilder {
    config: SubscriberConfig,
    initial_channels: Vec<PubSubChannel>,
}

impl SubscriberBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SubscriberConfig::default(),
            initial_channels: Vec::new(),
        }
    }

    /// Set Redis URL
    #[must_use]
    pub fn redis_url(mut self, url: impl Into<String>) -> Self {
        self.config.redis_url = url.into();
        self
    }

    /// Set broadcast buffer size
    #[must_use]
    pub fn broadcast_buffer(mut self, size: usize) -> Self {
        self.config.broadcast_buffer = size;
        self
    }

    /// Set reconnection delay
    #[must_use]
    pub fn reconnect_delay_ms(mut self, delay: u64) -> Self {
        self.config.reconnect_delay_ms = delay;
        self
    }

    /// Add initial channel subscription
    #[must_use]
    pub fn subscribe(mut self, channel: PubSubChannel) -> Self {
        self.initial_channels.push(channel);
        self
    }

    /// Build and start the subscriber
    pub async fn build(self) -> SubscriberResult<Subscriber> {
        let subscriber = Subscriber::new(self.config).await?;

        if !self.initial_channels.is_empty() {
            subscriber.subscribe(&self.initial_channels).await?;
        }

        Ok(subscriber)
    }
}

impl Default for SubscriberBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_received_message_parsing() {
        let payload = r#"{"event_type":"TEST","data":{}}"#.to_string();
        let msg = ReceivedMessage::from_redis("guild:12345".to_string(), payload.clone());

        assert_eq!(msg.channel, PubSubChannel::Guild(chat_core::Snowflake::from(12345i64)));
        assert!(msg.event.is_some());
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_received_message_invalid_json() {
        let msg = ReceivedMessage::from_redis("user:123".to_string(), "invalid".to_string());

        assert_eq!(msg.channel, PubSubChannel::User(chat_core::Snowflake::from(123i64)));
        assert!(msg.event.is_none());
        assert_eq!(msg.payload, "invalid");
    }

    #[test]
    fn test_subscriber_config_default() {
        let config = SubscriberConfig::default();
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.broadcast_buffer, 1024);
        assert_eq!(config.reconnect_delay_ms, 1000);
    }

    #[test]
    fn test_subscriber_builder() {
        let builder = SubscriberBuilder::new()
            .redis_url("redis://localhost:6380")
            .broadcast_buffer(2048)
            .reconnect_delay_ms(500)
            .subscribe(PubSubChannel::broadcast());

        assert_eq!(builder.config.redis_url, "redis://localhost:6380");
        assert_eq!(builder.config.broadcast_buffer, 2048);
        assert_eq!(builder.config.reconnect_delay_ms, 500);
        assert_eq!(builder.initial_channels.len(), 1);
    }
}
