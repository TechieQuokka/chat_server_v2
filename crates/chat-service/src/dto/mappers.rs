//! Entity to DTO mappers
//!
//! Implements `From` conversions from domain entities to response DTOs.

use chat_core::entities::{
    Attachment, Channel, ChannelType, Guild, GuildMember, Invite, Message, Reaction, Role, User,
};
use chat_core::Snowflake;

use super::responses::{
    AttachmentResponse, ChannelResponse, CurrentUserResponse, DmChannelResponse,
    GuildPreviewResponse, GuildResponse, GuildWithCountsResponse, InviteChannelResponse,
    InviteResponse, MemberResponse, MessageReferenceResponse, MessageResponse, PublicUserResponse,
    ReactionResponse, RoleResponse, UserResponse,
};

// ============================================================================
// User Mappers
// ============================================================================

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            avatar: user.avatar.clone(),
            bot: user.bot,
            system: user.system,
            created_at: user.created_at,
        }
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self::from(&user)
    }
}

impl From<&User> for CurrentUserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            email: user.email.clone(),
            avatar: user.avatar.clone(),
            bot: user.bot,
            system: user.system,
            created_at: user.created_at,
        }
    }
}

impl From<User> for CurrentUserResponse {
    fn from(user: User) -> Self {
        Self::from(&user)
    }
}

impl From<&User> for PublicUserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            discriminator: user.discriminator.clone(),
            avatar: user.avatar.clone(),
            bot: user.bot,
        }
    }
}

impl From<User> for PublicUserResponse {
    fn from(user: User) -> Self {
        Self::from(&user)
    }
}

// ============================================================================
// Guild Mappers
// ============================================================================

impl From<&Guild> for GuildResponse {
    fn from(guild: &Guild) -> Self {
        Self {
            id: guild.id.to_string(),
            name: guild.name.clone(),
            icon: guild.icon.clone(),
            description: guild.description.clone(),
            owner_id: guild.owner_id.to_string(),
            created_at: guild.created_at,
        }
    }
}

impl From<Guild> for GuildResponse {
    fn from(guild: Guild) -> Self {
        Self::from(&guild)
    }
}

impl From<&Guild> for GuildPreviewResponse {
    fn from(guild: &Guild) -> Self {
        Self {
            id: guild.id.to_string(),
            name: guild.name.clone(),
            icon: guild.icon.clone(),
            description: guild.description.clone(),
            member_count: 0, // Must be set separately
        }
    }
}

/// Helper struct for creating GuildWithCountsResponse
pub struct GuildWithCounts {
    pub guild: Guild,
    pub member_count: i64,
    pub channel_count: i64,
}

impl From<GuildWithCounts> for GuildWithCountsResponse {
    fn from(gwc: GuildWithCounts) -> Self {
        Self {
            id: gwc.guild.id.to_string(),
            name: gwc.guild.name,
            icon: gwc.guild.icon,
            description: gwc.guild.description,
            owner_id: gwc.guild.owner_id.to_string(),
            member_count: gwc.member_count,
            channel_count: gwc.channel_count,
            created_at: gwc.guild.created_at,
        }
    }
}

// ============================================================================
// Channel Mappers
// ============================================================================

impl From<&Channel> for ChannelResponse {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id.to_string(),
            guild_id: channel.guild_id.map(|id| id.to_string()),
            name: channel.name.clone(),
            channel_type: channel_type_to_i32(channel.channel_type),
            topic: channel.topic.clone(),
            position: channel.position,
            parent_id: channel.parent_id.map(|id| id.to_string()),
            created_at: channel.created_at,
        }
    }
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self::from(&channel)
    }
}

impl From<&Channel> for InviteChannelResponse {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id.to_string(),
            name: channel.name.clone().unwrap_or_default(),
            channel_type: channel_type_to_i32(channel.channel_type),
        }
    }
}

/// Helper struct for creating DmChannelResponse
pub struct DmChannelWithRecipients {
    pub channel: Channel,
    pub recipients: Vec<User>,
    pub last_message_id: Option<Snowflake>,
}

impl From<DmChannelWithRecipients> for DmChannelResponse {
    fn from(dm: DmChannelWithRecipients) -> Self {
        Self {
            id: dm.channel.id.to_string(),
            channel_type: channel_type_to_i32(dm.channel.channel_type),
            recipients: dm.recipients.into_iter().map(UserResponse::from).collect(),
            last_message_id: dm.last_message_id.map(|id| id.to_string()),
        }
    }
}

fn channel_type_to_i32(channel_type: ChannelType) -> i32 {
    match channel_type {
        ChannelType::GuildText => 0,
        ChannelType::Dm => 1,
        ChannelType::GuildCategory => 4,
    }
}

// ============================================================================
// Message Mappers
// ============================================================================

/// Helper struct for creating MessageResponse with all related data
pub struct MessageWithDetails {
    pub message: Message,
    pub author: User,
    pub guild_id: Option<Snowflake>,
    pub attachments: Vec<Attachment>,
    pub reactions: Vec<(String, i64, bool)>, // (emoji, count, me)
    pub reference: Option<MessageReference>,
}

pub struct MessageReference {
    pub message_id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
}

impl From<MessageWithDetails> for MessageResponse {
    fn from(details: MessageWithDetails) -> Self {
        Self {
            id: details.message.id.to_string(),
            channel_id: details.message.channel_id.to_string(),
            guild_id: details.guild_id.map(|id| id.to_string()),
            author: UserResponse::from(details.author),
            content: details.message.content,
            timestamp: details.message.created_at,
            edited_timestamp: details.message.edited_at,
            attachments: details
                .attachments
                .into_iter()
                .map(AttachmentResponse::from)
                .collect(),
            reactions: details
                .reactions
                .into_iter()
                .map(|(emoji, count, me)| ReactionResponse { emoji, count, me })
                .collect(),
            message_reference: details.reference.map(|r| MessageReferenceResponse {
                message_id: r.message_id.to_string(),
                channel_id: r.channel_id.to_string(),
                guild_id: r.guild_id.map(|id| id.to_string()),
            }),
        }
    }
}

impl From<Attachment> for AttachmentResponse {
    fn from(attachment: Attachment) -> Self {
        Self {
            id: attachment.id.to_string(),
            filename: attachment.filename,
            content_type: attachment.content_type,
            size: attachment.size,
            url: attachment.url,
            proxy_url: attachment.proxy_url,
            width: attachment.width,
            height: attachment.height,
        }
    }
}

impl From<&Attachment> for AttachmentResponse {
    fn from(attachment: &Attachment) -> Self {
        Self {
            id: attachment.id.to_string(),
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            size: attachment.size,
            url: attachment.url.clone(),
            proxy_url: attachment.proxy_url.clone(),
            width: attachment.width,
            height: attachment.height,
        }
    }
}

// ============================================================================
// Role Mappers
// ============================================================================

impl From<&Role> for RoleResponse {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id.to_string(),
            name: role.name.clone(),
            color: role.color,
            hoist: role.hoist,
            position: role.position,
            permissions: role.permissions.to_string(),
            mentionable: role.mentionable,
        }
    }
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        Self::from(&role)
    }
}

// ============================================================================
// Member Mappers
// ============================================================================

/// Helper struct for creating MemberResponse
pub struct MemberWithUser {
    pub member: GuildMember,
    pub user: User,
}

impl From<MemberWithUser> for MemberResponse {
    fn from(mwu: MemberWithUser) -> Self {
        Self {
            user: UserResponse::from(mwu.user),
            nickname: mwu.member.nickname,
            roles: mwu.member.role_ids.into_iter().map(|id| id.to_string()).collect(),
            joined_at: mwu.member.joined_at,
        }
    }
}

// ============================================================================
// Invite Mappers
// ============================================================================

/// Helper struct for creating InviteResponse
pub struct InviteWithDetails {
    pub invite: Invite,
    pub guild: Guild,
    pub channel: Channel,
    pub inviter: User,
    pub member_count: i64,
}

impl From<InviteWithDetails> for InviteResponse {
    fn from(details: InviteWithDetails) -> Self {
        Self {
            code: details.invite.code,
            guild: GuildPreviewResponse {
                id: details.guild.id.to_string(),
                name: details.guild.name,
                icon: details.guild.icon,
                description: details.guild.description,
                member_count: details.member_count,
            },
            channel: InviteChannelResponse::from(&details.channel),
            inviter: UserResponse::from(details.inviter),
            uses: details.invite.uses,
            max_uses: details.invite.max_uses,
            max_age: details.invite.max_age,
            temporary: details.invite.temporary,
            created_at: details.invite.created_at,
            expires_at: details.invite.expires_at,
        }
    }
}

// ============================================================================
// Reaction Mappers
// ============================================================================

impl From<&Reaction> for ReactionResponse {
    fn from(reaction: &Reaction) -> Self {
        Self {
            emoji: reaction.emoji.clone(),
            count: 1, // Single reaction, count must be aggregated separately
            me: false, // Must be set based on current user
        }
    }
}

/// Helper struct for creating ReactionResponse with count and me flag
pub struct ReactionWithMeta {
    pub emoji: String,
    pub count: i64,
    pub me: bool,
}

impl From<ReactionWithMeta> for ReactionResponse {
    fn from(rwm: ReactionWithMeta) -> Self {
        Self {
            emoji: rwm.emoji,
            count: rwm.count,
            me: rwm.me,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chat_core::Permissions;
    use chrono::Utc;

    fn create_test_user() -> User {
        User {
            id: Snowflake::new(123456789),
            username: "testuser".to_string(),
            discriminator: "0001".to_string(),
            email: "test@example.com".to_string(),
            avatar: Some("avatar_hash".to_string()),
            bot: false,
            system: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_guild() -> Guild {
        Guild {
            id: Snowflake::new(987654321),
            name: "Test Guild".to_string(),
            icon: Some("icon_hash".to_string()),
            description: Some("A test guild".to_string()),
            owner_id: Snowflake::new(123456789),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_user_to_user_response() {
        let user = create_test_user();
        let response = UserResponse::from(&user);

        assert_eq!(response.id, "123456789");
        assert_eq!(response.username, "testuser");
        assert_eq!(response.discriminator, "0001");
        assert!(response.avatar.is_some());
        assert!(!response.bot);
    }

    #[test]
    fn test_user_to_current_user_response() {
        let user = create_test_user();
        let response = CurrentUserResponse::from(&user);

        assert_eq!(response.email, "test@example.com");
    }

    #[test]
    fn test_guild_to_guild_response() {
        let guild = create_test_guild();
        let response = GuildResponse::from(&guild);

        assert_eq!(response.id, "987654321");
        assert_eq!(response.name, "Test Guild");
        assert_eq!(response.owner_id, "123456789");
    }

    #[test]
    fn test_channel_type_mapping() {
        assert_eq!(channel_type_to_i32(ChannelType::GuildText), 0);
        assert_eq!(channel_type_to_i32(ChannelType::Dm), 1);
        assert_eq!(channel_type_to_i32(ChannelType::GuildCategory), 4);
    }

    #[test]
    fn test_role_to_role_response() {
        let role = Role {
            id: Snowflake::new(111222333),
            guild_id: Snowflake::new(987654321),
            name: "Moderator".to_string(),
            color: 0x3498db,
            hoist: true,
            position: 5,
            permissions: Permissions::MANAGE_MESSAGES | Permissions::KICK_MEMBERS,
            mentionable: true,
            is_everyone: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = RoleResponse::from(&role);

        assert_eq!(response.id, "111222333");
        assert_eq!(response.name, "Moderator");
        assert_eq!(response.color, 0x3498db);
        assert!(response.hoist);
        assert!(response.mentionable);
    }

    #[test]
    fn test_member_with_user_to_response() {
        let user = create_test_user();
        let member = GuildMember {
            guild_id: Snowflake::new(987654321),
            user_id: user.id,
            nickname: Some("TestNick".to_string()),
            role_ids: vec![Snowflake::new(111), Snowflake::new(222)],
            joined_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response = MemberResponse::from(MemberWithUser { member, user });

        assert_eq!(response.nickname, Some("TestNick".to_string()));
        assert_eq!(response.roles.len(), 2);
        assert!(response.roles.contains(&"111".to_string()));
        assert!(response.roles.contains(&"222".to_string()));
    }
}
