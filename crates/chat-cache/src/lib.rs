//! # chat-cache
//!
//! Redis caching layer for sessions, presence, and pub/sub messaging.
//!
//! ## Features
//!
//! - **Connection Pool**: Managed Redis connection pool with deadpool
//! - **Session Storage**: Refresh tokens and WebSocket session management
//! - **Presence**: User online status and typing indicators
//! - **Pub/Sub**: Real-time event distribution across server instances
//!
//! ## Example
//!
//! ```ignore
//! use chat_cache::{RedisPool, RedisPoolConfig, PresenceStore, Publisher};
//!
//! // Create Redis pool
//! let config = RedisPoolConfig::default();
//! let pool = RedisPool::new(config)?;
//!
//! // Create stores
//! let presence_store = PresenceStore::new(pool.clone());
//! let publisher = Publisher::new(pool.clone());
//!
//! // Set user presence
//! let presence = PresenceData::new(user_id, UserStatus::Online);
//! presence_store.set_presence(&presence).await?;
//!
//! // Publish event
//! let event = PubSubEvent::new("PRESENCE_UPDATE", data);
//! publisher.publish(&PubSubChannel::guild(guild_id), &event).await?;
//! ```

pub mod pool;
pub mod presence;
pub mod pubsub;
pub mod session;

// Re-export pool types
pub use pool::{
    create_shared_pool, RedisPool, RedisPoolConfig, RedisPoolError, RedisResult, SharedRedisPool,
};

// Re-export session types
pub use session::{
    ClientProperties, RefreshTokenData, RefreshTokenStore, SessionEvent, SessionState,
    WebSocketSessionData, WebSocketSessionStore,
};

// Re-export presence types
pub use presence::{PresenceData, PresenceStore, TypingData, UserStatus};

// Re-export pubsub types
pub use pubsub::{
    EventTarget, PubSubChannel, PubSubEvent, Publisher, ReceivedMessage, Subscriber,
    SubscriberBuilder, SubscriberConfig, SubscriberError, SubscriberResult,
    BROADCAST_CHANNEL, CHANNEL_PREFIX, GUILD_CHANNEL_PREFIX, USER_CHANNEL_PREFIX,
};
