//! # chat-api
//!
//! REST API server built with Axum framework.
//!
//! This crate provides the HTTP API layer for the chat server, including:
//! - Authentication endpoints (register, login, logout, refresh)
//! - User management endpoints
//! - Guild (server) management
//! - Channel operations
//! - Message operations
//! - Role management
//! - Invite handling
//! - Reaction management
//!
//! ## Architecture
//!
//! - `handlers/` - Route handlers organized by domain
//! - `extractors/` - Custom Axum extractors (AuthUser, Pagination, ValidatedJson)
//! - `middleware/` - Request middleware (logging, CORS, request ID)
//! - `response/` - Response types and error handling
//! - `routes/` - Route definitions
//! - `server/` - Server setup and initialization
//! - `state/` - Application state

pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod response;
pub mod routes;
pub mod server;
pub mod state;

// Re-export commonly used types
pub use extractors::{AuthUser, Pagination, ValidatedJson};
pub use response::{ApiError, ApiResult, Created, NoContent};
pub use server::{create_app, create_app_state, run, run_server};
pub use state::AppState;
