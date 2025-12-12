//! Repository traits (ports) - define the interface for data access
//!
//! These traits follow the Repository pattern from Domain-Driven Design.
//! The domain layer defines what it needs, and the infrastructure layer
//! provides the implementation.

use async_trait::async_trait;

use crate::entities::{
    Attachment, Channel, Guild, GuildMember, Invite, Message, Reaction, Role, User,
};
use crate::error::DomainError;
use crate::value_objects::Snowflake;

/// Result type for repository operations
pub type RepoResult<T> = Result<T, DomainError>;

// ============================================================================
// User Repository
// ============================================================================

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<User>>;

    /// Find user by email
    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>>;

    /// Find user by username and discriminator
    async fn find_by_tag(&self, username: &str, discriminator: &str) -> RepoResult<Option<User>>;

    /// Check if email is already taken
    async fn email_exists(&self, email: &str) -> RepoResult<bool>;

    /// Create a new user
    async fn create(&self, user: &User, password_hash: &str) -> RepoResult<()>;

    /// Update an existing user
    async fn update(&self, user: &User) -> RepoResult<()>;

    /// Soft delete a user
    async fn delete(&self, id: Snowflake) -> RepoResult<()>;

    /// Get password hash for authentication
    async fn get_password_hash(&self, id: Snowflake) -> RepoResult<Option<String>>;

    /// Update password hash
    async fn update_password(&self, id: Snowflake, password_hash: &str) -> RepoResult<()>;

    /// Generate next available discriminator for username
    async fn next_discriminator(&self, username: &str) -> RepoResult<String>;
}

// ============================================================================
// Guild Repository
// ============================================================================

#[async_trait]
pub trait GuildRepository: Send + Sync {
    /// Find guild by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Guild>>;

    /// List all guilds a user is a member of
    async fn find_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<Guild>>;

    /// Create a new guild
    async fn create(&self, guild: &Guild) -> RepoResult<()>;

    /// Update an existing guild
    async fn update(&self, guild: &Guild) -> RepoResult<()>;

    /// Soft delete a guild
    async fn delete(&self, id: Snowflake) -> RepoResult<()>;

    /// Get member count for a guild
    async fn member_count(&self, guild_id: Snowflake) -> RepoResult<i64>;
}

// ============================================================================
// Channel Repository
// ============================================================================

#[async_trait]
pub trait ChannelRepository: Send + Sync {
    /// Find channel by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Channel>>;

    /// List all channels in a guild
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Channel>>;

    /// Find DM channel between two users
    async fn find_dm(&self, user1_id: Snowflake, user2_id: Snowflake) -> RepoResult<Option<Channel>>;

    /// List all DM channels for a user
    async fn find_dms_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<Channel>>;

    /// Create a new channel
    async fn create(&self, channel: &Channel) -> RepoResult<()>;

    /// Update an existing channel
    async fn update(&self, channel: &Channel) -> RepoResult<()>;

    /// Soft delete a channel
    async fn delete(&self, id: Snowflake) -> RepoResult<()>;

    /// Add user to a DM channel
    async fn add_dm_recipient(&self, channel_id: Snowflake, user_id: Snowflake) -> RepoResult<()>;

    /// Get DM recipients
    async fn get_dm_recipients(&self, channel_id: Snowflake) -> RepoResult<Vec<Snowflake>>;
}

// ============================================================================
// Message Repository
// ============================================================================

/// Pagination options for message queries
#[derive(Debug, Clone, Default)]
pub struct MessageQuery {
    pub before: Option<Snowflake>,
    pub after: Option<Snowflake>,
    pub limit: i64,
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// Find message by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Message>>;

    /// List messages in a channel with pagination
    async fn find_by_channel(&self, channel_id: Snowflake, query: MessageQuery)
        -> RepoResult<Vec<Message>>;

    /// Create a new message
    async fn create(&self, message: &Message) -> RepoResult<()>;

    /// Update message content (edit)
    async fn update(&self, message: &Message) -> RepoResult<()>;

    /// Soft delete a message
    async fn delete(&self, id: Snowflake) -> RepoResult<()>;

    /// Bulk delete messages
    async fn bulk_delete(&self, channel_id: Snowflake, message_ids: &[Snowflake]) -> RepoResult<u64>;

    /// Get message with attachments
    async fn find_with_attachments(&self, id: Snowflake) -> RepoResult<Option<(Message, Vec<Attachment>)>>;
}

// ============================================================================
// Attachment Repository
// ============================================================================

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    /// Find attachment by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Attachment>>;

    /// Find attachments for a message
    async fn find_by_message(&self, message_id: Snowflake) -> RepoResult<Vec<Attachment>>;

    /// Create a new attachment
    async fn create(&self, attachment: &Attachment) -> RepoResult<()>;

    /// Delete attachments for a message
    async fn delete_by_message(&self, message_id: Snowflake) -> RepoResult<()>;
}

// ============================================================================
// Role Repository
// ============================================================================

#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Find role by ID
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Role>>;

    /// List all roles in a guild (ordered by position)
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Role>>;

    /// Find the @everyone role for a guild
    async fn find_everyone(&self, guild_id: Snowflake) -> RepoResult<Option<Role>>;

    /// Create a new role
    async fn create(&self, role: &Role) -> RepoResult<()>;

    /// Update an existing role
    async fn update(&self, role: &Role) -> RepoResult<()>;

    /// Soft delete a role
    async fn delete(&self, id: Snowflake) -> RepoResult<()>;

    /// Update role positions in bulk
    async fn update_positions(&self, guild_id: Snowflake, positions: &[(Snowflake, i32)]) -> RepoResult<()>;
}

// ============================================================================
// Member Repository
// ============================================================================

#[async_trait]
pub trait MemberRepository: Send + Sync {
    /// Find member by guild and user ID
    async fn find(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Option<GuildMember>>;

    /// List all members in a guild
    async fn find_by_guild(&self, guild_id: Snowflake, limit: i64, after: Option<Snowflake>) -> RepoResult<Vec<GuildMember>>;

    /// List all guilds a user is a member of (as member records)
    async fn find_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<GuildMember>>;

    /// Check if user is a member of guild
    async fn is_member(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<bool>;

    /// Add member to guild
    async fn create(&self, member: &GuildMember) -> RepoResult<()>;

    /// Update member (nickname, etc.)
    async fn update(&self, member: &GuildMember) -> RepoResult<()>;

    /// Remove member from guild
    async fn delete(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<()>;

    /// Add role to member
    async fn add_role(&self, guild_id: Snowflake, user_id: Snowflake, role_id: Snowflake) -> RepoResult<()>;

    /// Remove role from member
    async fn remove_role(&self, guild_id: Snowflake, user_id: Snowflake, role_id: Snowflake) -> RepoResult<()>;

    /// Get all role IDs for a member
    async fn get_role_ids(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Vec<Snowflake>>;
}

// ============================================================================
// Reaction Repository
// ============================================================================

#[async_trait]
pub trait ReactionRepository: Send + Sync {
    /// Find reaction by message, user, and emoji
    async fn find(&self, message_id: Snowflake, user_id: Snowflake, emoji: &str) -> RepoResult<Option<Reaction>>;

    /// Get all reactions for a message
    async fn find_by_message(&self, message_id: Snowflake) -> RepoResult<Vec<Reaction>>;

    /// Get users who reacted with a specific emoji
    async fn find_users_by_emoji(&self, message_id: Snowflake, emoji: &str, limit: i64) -> RepoResult<Vec<Snowflake>>;

    /// Add a reaction
    async fn create(&self, reaction: &Reaction) -> RepoResult<()>;

    /// Remove a reaction
    async fn delete(&self, message_id: Snowflake, user_id: Snowflake, emoji: &str) -> RepoResult<()>;

    /// Remove all reactions from a message
    async fn delete_all(&self, message_id: Snowflake) -> RepoResult<()>;

    /// Remove all reactions of a specific emoji from a message
    async fn delete_by_emoji(&self, message_id: Snowflake, emoji: &str) -> RepoResult<()>;

    /// Count reactions by emoji for a message
    async fn count_by_emoji(&self, message_id: Snowflake) -> RepoResult<Vec<(String, i64)>>;
}

// ============================================================================
// Invite Repository
// ============================================================================

#[async_trait]
pub trait InviteRepository: Send + Sync {
    /// Find invite by code
    async fn find_by_code(&self, code: &str) -> RepoResult<Option<Invite>>;

    /// List invites for a guild
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Invite>>;

    /// List invites for a channel
    async fn find_by_channel(&self, channel_id: Snowflake) -> RepoResult<Vec<Invite>>;

    /// List invites created by a user
    async fn find_by_inviter(&self, inviter_id: Snowflake) -> RepoResult<Vec<Invite>>;

    /// Create a new invite
    async fn create(&self, invite: &Invite) -> RepoResult<()>;

    /// Increment invite use count
    async fn increment_uses(&self, code: &str) -> RepoResult<()>;

    /// Delete an invite
    async fn delete(&self, code: &str) -> RepoResult<()>;

    /// Delete expired invites for a guild
    async fn delete_expired(&self, guild_id: Snowflake) -> RepoResult<u64>;
}

// ============================================================================
// Ban Repository
// ============================================================================

/// Ban record
#[derive(Debug, Clone)]
pub struct Ban {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub reason: Option<String>,
}

#[async_trait]
pub trait BanRepository: Send + Sync {
    /// Check if user is banned from guild
    async fn is_banned(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<bool>;

    /// Get ban record
    async fn find(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Option<Ban>>;

    /// List all bans for a guild
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Ban>>;

    /// Create a ban
    async fn create(&self, ban: &Ban) -> RepoResult<()>;

    /// Remove a ban
    async fn delete(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<()>;
}
