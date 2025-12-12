//! # chat-gateway
//!
//! WebSocket gateway for real-time bidirectional communication.
//!
//! This crate implements the Discord-like WebSocket gateway protocol,
//! enabling real-time events such as messages, presence updates, and typing indicators.
//!
//! ## Features
//!
//! - **WebSocket Protocol**: Full implementation of gateway op codes (0-11)
//! - **Connection Management**: Thread-safe connection handling with DashMap
//! - **Session Resume**: 2-minute window for reconnecting without losing events
//! - **Heartbeat System**: Keep-alive mechanism with automatic zombie detection
//! - **Event Distribution**: Redis Pub/Sub integration for cross-instance events
//!
//! ## Op Codes
//!
//! | Op | Name | Direction | Description |
//! |----|------|-----------|-------------|
//! | 0 | Dispatch | S→C | Server dispatches an event |
//! | 1 | Heartbeat | C↔S | Keep connection alive |
//! | 2 | Identify | C→S | Authenticate session |
//! | 3 | Presence Update | C→S | Update online status |
//! | 4 | Resume | C→S | Resume dropped connection |
//! | 5 | Reconnect | S→C | Server requests reconnect |
//! | 7 | Invalid Session | S→C | Session is invalid |
//! | 10 | Hello | S→C | Sent on connect |
//! | 11 | Heartbeat ACK | S→C | Heartbeat acknowledged |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use chat_gateway::run;
//! use chat_common::AppConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = AppConfig::from_env()?;
//!     run(config).await?;
//!     Ok(())
//! }
//! ```

pub mod broadcast;
pub mod connection;
pub mod events;
pub mod handlers;
pub mod protocol;
pub mod server;

// Re-export main server function
pub use server::run;

// Re-export commonly used types
pub use broadcast::{EventDispatcher, EventDispatcherConfig};
pub use connection::{Connection, ConnectionManager, ConnectionState, Session};
pub use events::{GatewayEventType, ReadyEvent, UnavailableGuild};
pub use handlers::{HandlerError, HandlerResult, MessageDispatcher};
pub use protocol::{CloseCode, GatewayMessage, HelloPayload, IdentifyPayload, OpCode};
pub use server::{create_app, create_gateway_state, GatewayState};
