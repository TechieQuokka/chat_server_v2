//! Entity to model mappers
//!
//! This module provides conversions between domain entities (chat-core) and database models.
//! - `From<Model> for Entity`: Convert database rows to domain objects
//! - `*Insert`/`*Update` structs: Prepare entity data for database operations

mod channel;
mod guild;
mod invite;
mod member;
mod message;
mod reaction;
mod role;
mod user;

pub use channel::{channel_type_to_str, ChannelInsert, ChannelUpdate};
pub use guild::{GuildInsert, GuildUpdate};
pub use invite::InviteInsert;
pub use member::{member_with_roles, MemberInsert, MemberUpdate};
pub use message::{AttachmentInsert, MessageInsert};
pub use reaction::ReactionInsert;
pub use role::{RoleInsert, RoleUpdate};
pub use user::{UserInsert, UserUpdate};
