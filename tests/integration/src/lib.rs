//! Integration test utilities for the chat server
//!
//! This crate provides helpers for running end-to-end tests against
//! the REST API and WebSocket gateway.

pub mod helpers;
pub mod fixtures;

pub use helpers::*;
pub use fixtures::*;
