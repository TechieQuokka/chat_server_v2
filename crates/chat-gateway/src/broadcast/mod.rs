//! Event broadcasting
//!
//! Handles distribution of events from Redis Pub/Sub to WebSocket connections.

mod dispatcher;

pub use dispatcher::{EventDispatcher, EventDispatcherConfig};
