//! # chat-core
//!
//! Domain layer containing entities, value objects, repository traits, and domain events.
//! This crate has zero dependencies on infrastructure (database, web framework, etc.).

pub mod entities;
pub mod error;
pub mod events;
pub mod traits;
pub mod value_objects;

// Re-export commonly used types at crate root
pub use entities::{
    Attachment, Channel, ChannelType, Guild, GuildMember, Invite, Message, Reaction,
    ReactionCount, Role, User, generate_invite_code,
};
pub use error::DomainError;
pub use events::DomainEvent;
pub use traits::{
    AttachmentRepository, Ban, BanRepository, ChannelRepository, GuildRepository, InviteRepository,
    MemberRepository, MessageQuery, MessageRepository, ReactionRepository, RepoResult,
    RoleRepository, UserRepository,
};
pub use value_objects::{Permissions, Snowflake, SnowflakeGenerator, SnowflakeParseError};
