//! Axum extractors for request handling
//!
//! Custom extractors for authentication, validation, and pagination.

mod auth;
mod pagination;
mod path;
mod validated;

pub use auth::{AuthUser, OptionalAuthUser};
pub use pagination::{Pagination, PaginationParams};
pub use path::{
    ChannelIdPath, GuildIdPath, GuildRolePath, GuildUserPath, InviteCodePath, MessageIdPath,
    ReactionPath, SnowflakePath, UserIdPath, UserReactionPath,
};
pub use validated::{OptionalValidatedJson, ValidatedJson};
