//! Business logic services
//!
//! This module contains all service layer implementations that handle
//! business logic, validation, and orchestration of domain operations.

pub mod auth;
pub mod channel;
pub mod context;
pub mod dm;
pub mod error;
pub mod guild;
pub mod invite;
pub mod member;
pub mod message;
pub mod permission;
pub mod presence;
pub mod reaction;
pub mod role;
pub mod user;

// Re-export all services for convenience
pub use auth::AuthService;
pub use channel::ChannelService;
pub use context::{ServiceContext, ServiceContextBuilder};
pub use dm::DmService;
pub use error::{ServiceError, ServiceResult};
pub use guild::GuildService;
pub use invite::InviteService;
pub use member::MemberService;
pub use message::MessageService;
pub use permission::PermissionService;
pub use presence::PresenceService;
pub use reaction::ReactionService;
pub use role::RoleService;
pub use user::UserService;
