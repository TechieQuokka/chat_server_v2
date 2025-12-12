//! API Integration Tests
//!
//! These tests require:
//! - Running PostgreSQL instance
//! - Running Redis instance
//! - Environment variables: DATABASE_URL, REDIS_URL, JWT_SECRET
//!
//! Run with: cargo test -p integration-tests --test api_tests

use integration_tests::{
    assert_json, assert_status, check_test_env, fixtures::*, TestServer,
};
use reqwest::StatusCode;

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");
    let response = server.get("/health").await.expect("Request failed");
    assert_status(response, StatusCode::OK).await.unwrap();
}

#[tokio::test]
async fn test_health_ready() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");
    let response = server.get("/health/ready").await.expect("Request failed");
    assert_status(response, StatusCode::OK).await.unwrap();
}

// ============================================================================
// Auth Tests
// ============================================================================

#[tokio::test]
async fn test_register_user() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");
    let request = RegisterRequest::unique();

    let response = server.post("/auth/register", &request).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert_eq!(auth.user.username, request.username);
    assert!(!auth.access_token.is_empty());
    assert!(!auth.refresh_token.is_empty());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");
    let request = RegisterRequest::unique();

    // First registration
    server.post("/auth/register", &request).await.unwrap();

    // Second registration with same email
    let response = server.post("/auth/register", &request).await.unwrap();
    assert_status(response, StatusCode::CONFLICT).await.unwrap();
}

#[tokio::test]
async fn test_login() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register first
    let register_req = RegisterRequest::unique();
    server.post("/auth/register", &register_req).await.unwrap();

    // Login
    let login_req = LoginRequest::from_register(&register_req);
    let response = server.post("/auth/login", &login_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::OK).await.unwrap();

    assert_eq!(auth.user.username, register_req.username);
    assert!(!auth.access_token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");
    let login_req = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpass".to_string(),
    };

    let response = server.post("/auth/login", &login_req).await.unwrap();
    assert_status(response, StatusCode::UNAUTHORIZED).await.unwrap();
}

#[tokio::test]
async fn test_refresh_token() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Refresh
    let refresh_req = RefreshTokenRequest {
        refresh_token: auth.refresh_token,
    };
    let response = server.post("/auth/refresh", &refresh_req).await.unwrap();
    let tokens: TokenPairResponse = assert_json(response, StatusCode::OK).await.unwrap();

    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());
}

#[tokio::test]
async fn test_logout() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Logout
    let response = server
        .post_auth("/auth/logout", &auth.access_token, &())
        .await
        .unwrap();
    assert_status(response, StatusCode::NO_CONTENT).await.unwrap();
}

// ============================================================================
// User Tests
// ============================================================================

#[tokio::test]
async fn test_get_current_user() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Get current user
    let response = server
        .get_auth("/users/@me", &auth.access_token)
        .await
        .unwrap();
    let user: UserResponse = assert_json(response, StatusCode::OK).await.unwrap();

    assert_eq!(user.id, auth.user.id);
    assert_eq!(user.username, register_req.username);
}

#[tokio::test]
async fn test_get_current_user_unauthorized() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    let response = server.get("/users/@me").await.unwrap();
    assert_status(response, StatusCode::UNAUTHORIZED).await.unwrap();
}

// ============================================================================
// Guild Tests
// ============================================================================

#[tokio::test]
async fn test_create_guild() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create guild
    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert_eq!(guild.name, guild_req.name);
    assert_eq!(guild.owner_id, auth.user.id);
}

#[tokio::test]
async fn test_get_guild() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register and create guild
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let created: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Get guild
    let response = server
        .get_auth(&format!("/guilds/{}", created.id), &auth.access_token)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::OK).await.unwrap();

    assert_eq!(guild.id, created.id);
    assert_eq!(guild.name, guild_req.name);
}

#[tokio::test]
async fn test_delete_guild() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Register and create guild
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Delete guild
    let response = server
        .delete_auth(&format!("/guilds/{}", guild.id), &auth.access_token)
        .await
        .unwrap();
    assert_status(response, StatusCode::NO_CONTENT).await.unwrap();

    // Verify deleted
    let response = server
        .get_auth(&format!("/guilds/{}", guild.id), &auth.access_token)
        .await
        .unwrap();
    assert_status(response, StatusCode::NOT_FOUND).await.unwrap();
}

// ============================================================================
// Channel Tests
// ============================================================================

#[tokio::test]
async fn test_create_channel() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup: register and create guild
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create channel
    let channel_req = CreateChannelRequest::text_channel();
    let response = server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();
    let channel: ChannelResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert_eq!(channel.name, Some(channel_req.name));
    assert_eq!(channel.guild_id, Some(guild.id));
}

#[tokio::test]
async fn test_get_guild_channels() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create some channels
    let channel_req = CreateChannelRequest::text_channel();
    server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();

    // Get channels
    let response = server
        .get_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
        )
        .await
        .unwrap();
    let channels: Vec<ChannelResponse> = assert_json(response, StatusCode::OK).await.unwrap();

    // Should have at least the one we created
    assert!(!channels.is_empty());
}

// ============================================================================
// Message Tests
// ============================================================================

#[tokio::test]
async fn test_create_message() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup: register, create guild, create channel
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let channel_req = CreateChannelRequest::text_channel();
    let response = server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();
    let channel: ChannelResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create message
    let message_req = CreateMessageRequest::simple("Hello, World!");
    let response = server
        .post_auth(
            &format!("/channels/{}/messages", channel.id),
            &auth.access_token,
            &message_req,
        )
        .await
        .unwrap();
    let message: MessageResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert_eq!(message.content, "Hello, World!");
    assert_eq!(message.channel_id, channel.id);
    assert_eq!(message.author.id, auth.user.id);
}

#[tokio::test]
async fn test_get_channel_messages() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let channel_req = CreateChannelRequest::text_channel();
    let response = server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();
    let channel: ChannelResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create some messages
    for i in 0..3 {
        let message_req = CreateMessageRequest::simple(&format!("Message {}", i));
        server
            .post_auth(
                &format!("/channels/{}/messages", channel.id),
                &auth.access_token,
                &message_req,
            )
            .await
            .unwrap();
    }

    // Get messages
    let response = server
        .get_auth(
            &format!("/channels/{}/messages", channel.id),
            &auth.access_token,
        )
        .await
        .unwrap();
    let messages: Vec<MessageResponse> = assert_json(response, StatusCode::OK).await.unwrap();

    assert_eq!(messages.len(), 3);
}

// ============================================================================
// Role Tests
// ============================================================================

#[tokio::test]
async fn test_create_role() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create role
    let role_req = CreateRoleRequest::unique();
    let response = server
        .post_auth(
            &format!("/guilds/{}/roles", guild.id),
            &auth.access_token,
            &role_req,
        )
        .await
        .unwrap();
    let role: RoleResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert_eq!(role.name, role_req.name);
    assert_eq!(role.guild_id, guild.id);
}

#[tokio::test]
async fn test_get_guild_roles() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Get roles (should have @everyone at minimum)
    let response = server
        .get_auth(&format!("/guilds/{}/roles", guild.id), &auth.access_token)
        .await
        .unwrap();
    let roles: Vec<RoleResponse> = assert_json(response, StatusCode::OK).await.unwrap();

    assert!(!roles.is_empty());
}

// ============================================================================
// Member Tests
// ============================================================================

#[tokio::test]
async fn test_get_guild_members() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Get members (owner should be a member)
    let response = server
        .get_auth(
            &format!("/guilds/{}/members", guild.id),
            &auth.access_token,
        )
        .await
        .unwrap();
    let members: Vec<MemberResponse> = assert_json(response, StatusCode::OK).await.unwrap();

    assert!(!members.is_empty());
    assert!(members.iter().any(|m| m.user.id == auth.user.id));
}

// ============================================================================
// Invite Tests
// ============================================================================

#[tokio::test]
async fn test_create_invite() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let channel_req = CreateChannelRequest::text_channel();
    let response = server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();
    let channel: ChannelResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Create invite
    let invite_req = CreateInviteRequest::default();
    let response = server
        .post_auth(
            &format!("/channels/{}/invites", channel.id),
            &auth.access_token,
            &invite_req,
        )
        .await
        .unwrap();
    let invite: InviteResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    assert!(!invite.code.is_empty());
    assert_eq!(invite.guild_id, guild.id);
    assert_eq!(invite.channel_id, channel.id);
}

#[tokio::test]
async fn test_get_invite() {
    if !check_test_env().await {
        return;
    }

    let server = TestServer::start().await.expect("Failed to start server");

    // Setup
    let register_req = RegisterRequest::unique();
    let response = server.post("/auth/register", &register_req).await.unwrap();
    let auth: AuthResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let guild_req = CreateGuildRequest::unique();
    let response = server
        .post_auth("/guilds", &auth.access_token, &guild_req)
        .await
        .unwrap();
    let guild: GuildResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let channel_req = CreateChannelRequest::text_channel();
    let response = server
        .post_auth(
            &format!("/guilds/{}/channels", guild.id),
            &auth.access_token,
            &channel_req,
        )
        .await
        .unwrap();
    let channel: ChannelResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    let invite_req = CreateInviteRequest::default();
    let response = server
        .post_auth(
            &format!("/channels/{}/invites", channel.id),
            &auth.access_token,
            &invite_req,
        )
        .await
        .unwrap();
    let invite: InviteResponse = assert_json(response, StatusCode::CREATED).await.unwrap();

    // Get invite (public endpoint)
    let response = server
        .get(&format!("/invites/{}", invite.code))
        .await
        .unwrap();
    let fetched: InviteResponse = assert_json(response, StatusCode::OK).await.unwrap();

    assert_eq!(fetched.code, invite.code);
}
