//! Presence storage module.
//!
//! Tracks user online status and typing indicators.

mod user_presence;

pub use user_presence::{PresenceData, PresenceStore, TypingData, UserStatus};
