//! Redis Pub/Sub module.
//!
//! Provides publish/subscribe functionality for real-time event distribution.

mod channels;
mod publisher;
mod subscriber;

pub use channels::{
    PubSubChannel, BROADCAST_CHANNEL, CHANNEL_PREFIX, GUILD_CHANNEL_PREFIX, USER_CHANNEL_PREFIX,
};
pub use publisher::{EventTarget, PubSubEvent, Publisher};
pub use subscriber::{
    ReceivedMessage, Subscriber, SubscriberBuilder, SubscriberConfig, SubscriberError,
    SubscriberResult,
};
