//! Session storage module.
//!
//! Provides Redis-backed storage for:
//! - Refresh tokens (authentication sessions)
//! - WebSocket sessions (real-time connection state)

mod refresh_token;
mod websocket_session;

pub use refresh_token::{RefreshTokenData, RefreshTokenStore};
pub use websocket_session::{
    ClientProperties, SessionEvent, SessionState, WebSocketSessionData, WebSocketSessionStore,
};
