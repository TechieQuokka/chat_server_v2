//! Integration tests for chat-db repositories
//!
//! These tests require a running PostgreSQL database.
//! Set DATABASE_URL environment variable before running:
//!
//! ```bash
//! export DATABASE_URL="postgres://postgres:password@localhost:5432/chat_test"
//! cargo test -p chat-db --test integration_tests
//! ```

use chrono::Utc;
use sqlx::PgPool;

use chat_core::entities::{
    Channel, ChannelType, Guild, GuildMember, Invite, Message, Reaction, Role, User,
};
use chat_core::traits::{
    ChannelRepository, GuildRepository, InviteRepository, MemberRepository, MessageQuery,
    MessageRepository, ReactionRepository, RoleRepository, UserRepository,
};
use chat_core::value_objects::{Permissions, Snowflake};
use chat_db::{
    PgChannelRepository, PgGuildRepository, PgInviteRepository, PgMemberRepository,
    PgMessageRepository, PgReactionRepository, PgRoleRepository, PgUserRepository,
};

/// Helper to create a test database pool
async fn get_test_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

/// Generate a test Snowflake ID
fn test_snowflake() -> Snowflake {
    use std::sync::atomic::{AtomicI64, Ordering};
    static COUNTER: AtomicI64 = AtomicI64::new(1000000);
    Snowflake::new(COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Create a test user
fn create_test_user() -> User {
    let id = test_snowflake();
    User {
        id,
        username: format!("test_user_{}", id.into_inner()),
        discriminator: "0001".to_string(),
        email: format!("test_{}@example.com", id.into_inner()),
        avatar: None,
        bot: false,
        system: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test guild
fn create_test_guild(owner_id: Snowflake) -> Guild {
    let id = test_snowflake();
    Guild {
        id,
        name: format!("Test Guild {}", id.into_inner()),
        icon: None,
        description: Some("A test guild".to_string()),
        owner_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test channel
fn create_test_channel(guild_id: Snowflake) -> Channel {
    let id = test_snowflake();
    Channel {
        id,
        guild_id: Some(guild_id),
        name: Some(format!("test-channel-{}", id.into_inner())),
        channel_type: ChannelType::GuildText,
        topic: Some("Test topic".to_string()),
        position: 0,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test role
fn create_test_role(guild_id: Snowflake, is_everyone: bool) -> Role {
    let id = test_snowflake();
    Role {
        id,
        guild_id,
        name: if is_everyone {
            "@everyone".to_string()
        } else {
            format!("Test Role {}", id.into_inner())
        },
        color: 0x3498db,
        hoist: false,
        position: if is_everyone { 0 } else { 1 },
        permissions: Permissions::default(),
        mentionable: true,
        is_everyone,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test message
fn create_test_message(channel_id: Snowflake, author_id: Snowflake) -> Message {
    let id = test_snowflake();
    Message {
        id,
        channel_id,
        author_id,
        content: format!("Test message {}", id.into_inner()),
        created_at: Utc::now(),
        edited_at: None,
        reference_id: None,
    }
}

// ============================================================================
// User Repository Tests
// ============================================================================

#[tokio::test]
async fn test_user_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let repo = PgUserRepository::new(pool);
    let user = create_test_user();
    let password_hash = "hashed_password_123";

    // Create user
    repo.create(&user, password_hash).await.unwrap();

    // Find by ID
    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, user.id);
    assert_eq!(found.username, user.username);
    assert_eq!(found.email, user.email);

    // Find by email
    let found_by_email = repo.find_by_email(&user.email).await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().id, user.id);

    // Get password hash
    let hash = repo.get_password_hash(user.id).await.unwrap();
    assert_eq!(hash, Some(password_hash.to_string()));

    // Clean up
    repo.delete(user.id).await.unwrap();
}

#[tokio::test]
async fn test_user_email_exists() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let repo = PgUserRepository::new(pool);
    let user = create_test_user();

    // Email should not exist
    assert!(!repo.email_exists(&user.email).await.unwrap());

    // Create user
    repo.create(&user, "password").await.unwrap();

    // Email should exist now
    assert!(repo.email_exists(&user.email).await.unwrap());

    // Clean up
    repo.delete(user.id).await.unwrap();
}

// ============================================================================
// Guild Repository Tests
// ============================================================================

#[tokio::test]
async fn test_guild_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool);

    // Create owner first
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    // Create guild
    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    // Find by ID
    let found = guild_repo.find_by_id(guild.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, guild.id);
    assert_eq!(found.name, guild.name);
    assert_eq!(found.owner_id, owner.id);

    // Clean up
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Channel Repository Tests
// ============================================================================

#[tokio::test]
async fn test_channel_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let channel_repo = PgChannelRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    // Create channel
    let channel = create_test_channel(guild.id);
    channel_repo.create(&channel).await.unwrap();

    // Find by ID
    let found = channel_repo.find_by_id(channel.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, channel.id);
    assert_eq!(found.name, channel.name);

    // Find by guild
    let guild_channels = channel_repo.find_by_guild(guild.id).await.unwrap();
    assert!(!guild_channels.is_empty());
    assert!(guild_channels.iter().any(|c| c.id == channel.id));

    // Clean up
    channel_repo.delete(channel.id).await.unwrap();
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Message Repository Tests
// ============================================================================

#[tokio::test]
async fn test_message_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let channel_repo = PgChannelRepository::new(pool.clone());
    let message_repo = PgMessageRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    let channel = create_test_channel(guild.id);
    channel_repo.create(&channel).await.unwrap();

    // Create message
    let message = create_test_message(channel.id, owner.id);
    message_repo.create(&message).await.unwrap();

    // Find by ID
    let found = message_repo.find_by_id(message.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, message.id);
    assert_eq!(found.content, message.content);

    // Find by channel
    let query = MessageQuery {
        before: None,
        after: None,
        limit: 50,
    };
    let messages = message_repo.find_by_channel(channel.id, query).await.unwrap();
    assert!(!messages.is_empty());
    assert!(messages.iter().any(|m| m.id == message.id));

    // Clean up
    message_repo.delete(message.id).await.unwrap();
    channel_repo.delete(channel.id).await.unwrap();
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Role Repository Tests
// ============================================================================

#[tokio::test]
async fn test_role_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let role_repo = PgRoleRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    // Create @everyone role first
    let everyone_role = create_test_role(guild.id, true);
    role_repo.create(&everyone_role).await.unwrap();

    // Create regular role
    let role = create_test_role(guild.id, false);
    role_repo.create(&role).await.unwrap();

    // Find by ID
    let found = role_repo.find_by_id(role.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, role.id);
    assert_eq!(found.name, role.name);

    // Find @everyone
    let everyone = role_repo.find_everyone(guild.id).await.unwrap();
    assert!(everyone.is_some());
    assert!(everyone.unwrap().is_everyone);

    // Find by guild
    let roles = role_repo.find_by_guild(guild.id).await.unwrap();
    assert!(roles.len() >= 2);

    // Clean up
    role_repo.delete(role.id).await.unwrap();
    // Cannot delete @everyone role, so we delete guild
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Member Repository Tests
// ============================================================================

#[tokio::test]
async fn test_member_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let member_repo = PgMemberRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    // Create member
    let member = GuildMember {
        guild_id: guild.id,
        user_id: owner.id,
        nickname: Some("Test Nick".to_string()),
        role_ids: vec![],
        joined_at: Utc::now(),
        updated_at: Utc::now(),
    };
    member_repo.create(&member).await.unwrap();

    // Find member
    let found = member_repo.find(guild.id, owner.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.user_id, owner.id);
    assert_eq!(found.nickname, Some("Test Nick".to_string()));

    // Check is_member
    assert!(member_repo.is_member(guild.id, owner.id).await.unwrap());

    // Clean up
    member_repo.delete(guild.id, owner.id).await.unwrap();
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Reaction Repository Tests
// ============================================================================

#[tokio::test]
async fn test_reaction_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let channel_repo = PgChannelRepository::new(pool.clone());
    let message_repo = PgMessageRepository::new(pool.clone());
    let reaction_repo = PgReactionRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    let channel = create_test_channel(guild.id);
    channel_repo.create(&channel).await.unwrap();

    let message = create_test_message(channel.id, owner.id);
    message_repo.create(&message).await.unwrap();

    // Create reaction
    let reaction = Reaction {
        message_id: message.id,
        user_id: owner.id,
        emoji: "👍".to_string(),
        created_at: Utc::now(),
    };
    reaction_repo.create(&reaction).await.unwrap();

    // Find reaction
    let found = reaction_repo
        .find(message.id, owner.id, &reaction.emoji)
        .await
        .unwrap();
    assert!(found.is_some());

    // Count by emoji
    let counts = reaction_repo.count_by_emoji(message.id).await.unwrap();
    assert!(counts.iter().any(|(emoji, count)| emoji == "👍" && *count == 1));

    // Clean up
    reaction_repo.delete_all(message.id).await.unwrap();
    message_repo.delete(message.id).await.unwrap();
    channel_repo.delete(channel.id).await.unwrap();
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}

// ============================================================================
// Invite Repository Tests
// ============================================================================

#[tokio::test]
async fn test_invite_create_and_find() {
    let Some(pool) = get_test_pool().await else {
        eprintln!("Skipping test: DATABASE_URL not set");
        return;
    };

    let user_repo = PgUserRepository::new(pool.clone());
    let guild_repo = PgGuildRepository::new(pool.clone());
    let channel_repo = PgChannelRepository::new(pool.clone());
    let invite_repo = PgInviteRepository::new(pool);

    // Setup
    let owner = create_test_user();
    user_repo.create(&owner, "password").await.unwrap();

    let guild = create_test_guild(owner.id);
    guild_repo.create(&guild).await.unwrap();

    let channel = create_test_channel(guild.id);
    channel_repo.create(&channel).await.unwrap();

    // Create invite
    let code = format!("test{}", test_snowflake().into_inner());
    let invite = Invite {
        code: code.clone(),
        guild_id: guild.id,
        channel_id: channel.id,
        inviter_id: owner.id,
        uses: 0,
        max_uses: Some(10),
        max_age: Some(86400),
        temporary: false,
        created_at: Utc::now(),
        expires_at: None,
    };
    invite_repo.create(&invite).await.unwrap();

    // Find by code
    let found = invite_repo.find_by_code(&code).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.code, code);
    assert_eq!(found.uses, 0);

    // Increment uses
    invite_repo.increment_uses(&code).await.unwrap();
    let updated = invite_repo.find_by_code(&code).await.unwrap().unwrap();
    assert_eq!(updated.uses, 1);

    // Clean up
    invite_repo.delete(&code).await.unwrap();
    channel_repo.delete(channel.id).await.unwrap();
    guild_repo.delete(guild.id).await.unwrap();
    user_repo.delete(owner.id).await.unwrap();
}
