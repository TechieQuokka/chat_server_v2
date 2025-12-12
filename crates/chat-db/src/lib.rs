//! # chat-db
//!
//! Database layer implementing repository traits with PostgreSQL via SQLx.
//!
//! ## Overview
//!
//! This crate provides PostgreSQL implementations for all repository traits
//! defined in `chat-core`. It handles:
//!
//! - Connection pool management
//! - Database models with SQLx `FromRow` derives
//! - Entity ↔ Model mappers
//! - Repository implementations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use chat_db::pool::{create_pool, DatabaseConfig};
//! use chat_db::repositories::PgUserRepository;
//! use chat_core::traits::UserRepository;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DatabaseConfig::from_env();
//!     let pool = create_pool(&config).await?;
//!     let user_repo = PgUserRepository::new(pool);
//!
//!     // Use the repository...
//!     Ok(())
//! }
//! ```

pub mod mappers;
pub mod models;
pub mod pool;
pub mod repositories;

// Re-export commonly used types
pub use pool::{create_pool, create_pool_from_env, DatabaseConfig, PgPool};
pub use repositories::{
    PgAttachmentRepository, PgBanRepository, PgChannelRepository, PgGuildRepository,
    PgInviteRepository, PgMemberRepository, PgMessageRepository, PgReactionRepository,
    PgRoleRepository, PgUserRepository,
};
