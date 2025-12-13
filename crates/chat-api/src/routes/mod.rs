//! Route definitions
//!
//! All API routes organized by domain and mounted under /api/v1.

use axum::{routing::{delete, get, patch, post, put}, Router};

use crate::handlers::{auth, channels, guilds, health, invites, members, messages, reactions, roles, users};
use crate::state::AppState;

/// Create the main API router with all routes (excluding health for separate middleware handling)
pub fn create_router() -> Router<AppState> {
    Router::new()
        // API v1 endpoints
        .nest("/api/v1", api_v1_routes())
}

/// Health check routes (exported separately to bypass rate limiting)
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
}

/// API v1 routes
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        .merge(auth_routes())
        .merge(user_routes())
        .merge(guild_routes())
        .merge(channel_routes())
        .merge(invite_routes())
}

/// Authentication routes
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout))
}

/// User routes
fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users/@me", get(users::get_current_user))
        .route("/users/@me", patch(users::update_current_user))
        .route("/users/@me/guilds", get(users::get_current_user_guilds))
        .route("/users/@me/channels", get(users::get_dm_channels))
        .route("/users/@me/channels", post(users::create_dm_channel))
        .route("/users/:user_id", get(users::get_user))
}

/// Guild routes
fn guild_routes() -> Router<AppState> {
    Router::new()
        // Guild CRUD
        .route("/guilds", post(guilds::create_guild))
        .route("/guilds/:guild_id", get(guilds::get_guild))
        .route("/guilds/:guild_id", patch(guilds::update_guild))
        .route("/guilds/:guild_id", delete(guilds::delete_guild))
        // Guild channels
        .route("/guilds/:guild_id/channels", get(channels::get_guild_channels))
        .route("/guilds/:guild_id/channels", post(channels::create_channel))
        // Guild members
        .route("/guilds/:guild_id/members", get(members::get_guild_members))
        .route("/guilds/:guild_id/members/:user_id", get(members::get_guild_member))
        .route("/guilds/:guild_id/members/:user_id", patch(members::update_guild_member))
        .route("/guilds/:guild_id/members/:user_id", delete(members::remove_guild_member))
        .route("/guilds/:guild_id/members/@me", delete(members::leave_guild))
        // Guild roles
        .route("/guilds/:guild_id/roles", get(roles::get_guild_roles))
        .route("/guilds/:guild_id/roles", post(roles::create_role))
        .route("/guilds/:guild_id/roles/:role_id", get(roles::get_role))
        .route("/guilds/:guild_id/roles/:role_id", patch(roles::update_role))
        .route("/guilds/:guild_id/roles/:role_id", delete(roles::delete_role))
        // Guild invites
        .route("/guilds/:guild_id/invites", get(invites::get_guild_invites))
}

/// Channel routes
fn channel_routes() -> Router<AppState> {
    Router::new()
        // Channel CRUD
        .route("/channels/:channel_id", get(channels::get_channel))
        .route("/channels/:channel_id", patch(channels::update_channel))
        .route("/channels/:channel_id", delete(channels::delete_channel))
        // Channel messages
        .route("/channels/:channel_id/messages", get(messages::get_messages))
        .route("/channels/:channel_id/messages", post(messages::create_message))
        .route("/channels/:channel_id/messages/:message_id", get(messages::get_message))
        .route("/channels/:channel_id/messages/:message_id", patch(messages::update_message))
        .route("/channels/:channel_id/messages/:message_id", delete(messages::delete_message))
        // Message reactions
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji/@me",
            put(reactions::add_reaction),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji/@me",
            delete(reactions::remove_own_reaction),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji/:user_id",
            delete(reactions::remove_user_reaction),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji",
            get(reactions::get_reactions),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions/:emoji",
            delete(reactions::delete_all_reactions_for_emoji),
        )
        .route(
            "/channels/:channel_id/messages/:message_id/reactions",
            delete(reactions::delete_all_reactions),
        )
        // Typing indicator
        .route("/channels/:channel_id/typing", post(channels::typing_indicator))
        // Channel invites
        .route("/channels/:channel_id/invites", get(invites::get_channel_invites))
        .route("/channels/:channel_id/invites", post(invites::create_invite))
}

/// Invite routes
fn invite_routes() -> Router<AppState> {
    Router::new()
        .route("/invites/:invite_code", get(invites::get_invite))
        .route("/invites/:invite_code", post(invites::accept_invite))
        .route("/invites/:invite_code", delete(invites::delete_invite))
}
