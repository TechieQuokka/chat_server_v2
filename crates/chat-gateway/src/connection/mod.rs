//! Connection management
//!
//! Manages WebSocket connections, sessions, and message routing.

mod connection;
mod manager;
mod session;

pub use connection::{Connection, ConnectionState};
pub use manager::ConnectionManager;
pub use session::Session;
