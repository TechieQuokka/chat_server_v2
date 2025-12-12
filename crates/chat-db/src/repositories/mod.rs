//! Repository implementations
//!
//! PostgreSQL implementations of the repository traits defined in chat-core.
//! Each repository handles database operations for a specific domain entity.

mod attachment;
mod ban;
mod channel;
mod error;
mod guild;
mod invite;
mod member;
mod message;
mod reaction;
mod role;
mod user;

pub use attachment::PgAttachmentRepository;
pub use ban::PgBanRepository;
pub use channel::PgChannelRepository;
pub use guild::PgGuildRepository;
pub use invite::PgInviteRepository;
pub use member::PgMemberRepository;
pub use message::PgMessageRepository;
pub use reaction::PgReactionRepository;
pub use role::PgRoleRepository;
pub use user::PgUserRepository;
