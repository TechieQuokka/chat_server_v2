//! Service context - dependency container for services
//!
//! Holds all repositories, cache stores, and other dependencies needed by services.

use std::sync::Arc;

use chat_cache::{PresenceStore, Publisher, RefreshTokenStore, SharedRedisPool, WebSocketSessionStore};
use chat_common::auth::JwtService;
use chat_core::traits::{
    AttachmentRepository, BanRepository, ChannelRepository, GuildRepository, InviteRepository,
    MemberRepository, MessageRepository, ReactionRepository, RoleRepository, UserRepository,
};
use chat_core::SnowflakeGenerator;
use chat_db::PgPool;

/// Service context containing all dependencies
///
/// This is the main dependency container that gets passed to all services.
/// It provides access to:
/// - Database repositories
/// - Redis cache stores
/// - JWT service for authentication
/// - Snowflake generator for ID generation
/// - Redis pub/sub for events
#[derive(Clone)]
pub struct ServiceContext {
    // Database pool
    pool: PgPool,

    // Redis pool
    redis_pool: SharedRedisPool,

    // Repositories
    user_repo: Arc<dyn UserRepository>,
    guild_repo: Arc<dyn GuildRepository>,
    channel_repo: Arc<dyn ChannelRepository>,
    message_repo: Arc<dyn MessageRepository>,
    role_repo: Arc<dyn RoleRepository>,
    member_repo: Arc<dyn MemberRepository>,
    reaction_repo: Arc<dyn ReactionRepository>,
    invite_repo: Arc<dyn InviteRepository>,
    ban_repo: Arc<dyn BanRepository>,
    attachment_repo: Arc<dyn AttachmentRepository>,

    // Cache stores
    refresh_token_store: RefreshTokenStore,
    session_store: WebSocketSessionStore,
    presence_store: PresenceStore,

    // Pub/Sub
    publisher: Publisher,

    // Services
    jwt_service: Arc<JwtService>,
    snowflake_generator: Arc<SnowflakeGenerator>,
}

impl ServiceContext {
    /// Create a new service context with all dependencies
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: PgPool,
        redis_pool: SharedRedisPool,
        user_repo: Arc<dyn UserRepository>,
        guild_repo: Arc<dyn GuildRepository>,
        channel_repo: Arc<dyn ChannelRepository>,
        message_repo: Arc<dyn MessageRepository>,
        role_repo: Arc<dyn RoleRepository>,
        member_repo: Arc<dyn MemberRepository>,
        reaction_repo: Arc<dyn ReactionRepository>,
        invite_repo: Arc<dyn InviteRepository>,
        ban_repo: Arc<dyn BanRepository>,
        attachment_repo: Arc<dyn AttachmentRepository>,
        jwt_service: Arc<JwtService>,
        snowflake_generator: Arc<SnowflakeGenerator>,
    ) -> Self {
        // Clone the inner RedisPool from the Arc
        let inner_pool = (*redis_pool).clone();
        let refresh_token_store = RefreshTokenStore::new(inner_pool.clone());
        let session_store = WebSocketSessionStore::new(inner_pool.clone());
        let presence_store = PresenceStore::new(inner_pool.clone());
        let publisher = Publisher::new(inner_pool);

        Self {
            pool,
            redis_pool,
            user_repo,
            guild_repo,
            channel_repo,
            message_repo,
            role_repo,
            member_repo,
            reaction_repo,
            invite_repo,
            ban_repo,
            attachment_repo,
            refresh_token_store,
            session_store,
            presence_store,
            publisher,
            jwt_service,
            snowflake_generator,
        }
    }

    // === Database Pool ===

    /// Get the PostgreSQL connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the Redis connection pool
    pub fn redis_pool(&self) -> &SharedRedisPool {
        &self.redis_pool
    }

    // === Repositories ===

    /// Get the user repository
    pub fn user_repo(&self) -> &dyn UserRepository {
        self.user_repo.as_ref()
    }

    /// Get the guild repository
    pub fn guild_repo(&self) -> &dyn GuildRepository {
        self.guild_repo.as_ref()
    }

    /// Get the channel repository
    pub fn channel_repo(&self) -> &dyn ChannelRepository {
        self.channel_repo.as_ref()
    }

    /// Get the message repository
    pub fn message_repo(&self) -> &dyn MessageRepository {
        self.message_repo.as_ref()
    }

    /// Get the role repository
    pub fn role_repo(&self) -> &dyn RoleRepository {
        self.role_repo.as_ref()
    }

    /// Get the member repository
    pub fn member_repo(&self) -> &dyn MemberRepository {
        self.member_repo.as_ref()
    }

    /// Get the reaction repository
    pub fn reaction_repo(&self) -> &dyn ReactionRepository {
        self.reaction_repo.as_ref()
    }

    /// Get the invite repository
    pub fn invite_repo(&self) -> &dyn InviteRepository {
        self.invite_repo.as_ref()
    }

    /// Get the ban repository
    pub fn ban_repo(&self) -> &dyn BanRepository {
        self.ban_repo.as_ref()
    }

    /// Get the attachment repository
    pub fn attachment_repo(&self) -> &dyn AttachmentRepository {
        self.attachment_repo.as_ref()
    }

    // === Cache Stores ===

    /// Get the refresh token store
    pub fn refresh_token_store(&self) -> &RefreshTokenStore {
        &self.refresh_token_store
    }

    /// Get the WebSocket session store
    pub fn session_store(&self) -> &WebSocketSessionStore {
        &self.session_store
    }

    /// Get the presence store
    pub fn presence_store(&self) -> &PresenceStore {
        &self.presence_store
    }

    // === Pub/Sub ===

    /// Get the Redis pub/sub publisher
    pub fn publisher(&self) -> &Publisher {
        &self.publisher
    }

    // === Services ===

    /// Get the JWT service
    pub fn jwt_service(&self) -> &JwtService {
        self.jwt_service.as_ref()
    }

    /// Get the snowflake ID generator
    pub fn snowflake_generator(&self) -> &SnowflakeGenerator {
        self.snowflake_generator.as_ref()
    }

    /// Generate a new Snowflake ID
    pub fn generate_id(&self) -> chat_core::Snowflake {
        self.snowflake_generator.generate()
    }
}

impl std::fmt::Debug for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceContext")
            .field("pool", &"PgPool")
            .field("redis_pool", &"SharedRedisPool")
            .field("repositories", &"...")
            .field("cache_stores", &"...")
            .finish()
    }
}

/// Builder for creating ServiceContext with custom configuration
pub struct ServiceContextBuilder {
    pool: Option<PgPool>,
    redis_pool: Option<SharedRedisPool>,
    user_repo: Option<Arc<dyn UserRepository>>,
    guild_repo: Option<Arc<dyn GuildRepository>>,
    channel_repo: Option<Arc<dyn ChannelRepository>>,
    message_repo: Option<Arc<dyn MessageRepository>>,
    role_repo: Option<Arc<dyn RoleRepository>>,
    member_repo: Option<Arc<dyn MemberRepository>>,
    reaction_repo: Option<Arc<dyn ReactionRepository>>,
    invite_repo: Option<Arc<dyn InviteRepository>>,
    ban_repo: Option<Arc<dyn BanRepository>>,
    attachment_repo: Option<Arc<dyn AttachmentRepository>>,
    jwt_service: Option<Arc<JwtService>>,
    snowflake_generator: Option<Arc<SnowflakeGenerator>>,
}

impl ServiceContextBuilder {
    pub fn new() -> Self {
        Self {
            pool: None,
            redis_pool: None,
            user_repo: None,
            guild_repo: None,
            channel_repo: None,
            message_repo: None,
            role_repo: None,
            member_repo: None,
            reaction_repo: None,
            invite_repo: None,
            ban_repo: None,
            attachment_repo: None,
            jwt_service: None,
            snowflake_generator: None,
        }
    }

    pub fn pool(mut self, pool: PgPool) -> Self {
        self.pool = Some(pool);
        self
    }

    pub fn redis_pool(mut self, redis_pool: SharedRedisPool) -> Self {
        self.redis_pool = Some(redis_pool);
        self
    }

    pub fn user_repo(mut self, repo: Arc<dyn UserRepository>) -> Self {
        self.user_repo = Some(repo);
        self
    }

    pub fn guild_repo(mut self, repo: Arc<dyn GuildRepository>) -> Self {
        self.guild_repo = Some(repo);
        self
    }

    pub fn channel_repo(mut self, repo: Arc<dyn ChannelRepository>) -> Self {
        self.channel_repo = Some(repo);
        self
    }

    pub fn message_repo(mut self, repo: Arc<dyn MessageRepository>) -> Self {
        self.message_repo = Some(repo);
        self
    }

    pub fn role_repo(mut self, repo: Arc<dyn RoleRepository>) -> Self {
        self.role_repo = Some(repo);
        self
    }

    pub fn member_repo(mut self, repo: Arc<dyn MemberRepository>) -> Self {
        self.member_repo = Some(repo);
        self
    }

    pub fn reaction_repo(mut self, repo: Arc<dyn ReactionRepository>) -> Self {
        self.reaction_repo = Some(repo);
        self
    }

    pub fn invite_repo(mut self, repo: Arc<dyn InviteRepository>) -> Self {
        self.invite_repo = Some(repo);
        self
    }

    pub fn ban_repo(mut self, repo: Arc<dyn BanRepository>) -> Self {
        self.ban_repo = Some(repo);
        self
    }

    pub fn attachment_repo(mut self, repo: Arc<dyn AttachmentRepository>) -> Self {
        self.attachment_repo = Some(repo);
        self
    }

    pub fn jwt_service(mut self, service: Arc<JwtService>) -> Self {
        self.jwt_service = Some(service);
        self
    }

    pub fn snowflake_generator(mut self, generator: Arc<SnowflakeGenerator>) -> Self {
        self.snowflake_generator = Some(generator);
        self
    }

    /// Build the ServiceContext
    ///
    /// # Errors
    /// Returns `ServiceError::Validation` if any required dependency is missing
    pub fn build(self) -> super::error::ServiceResult<ServiceContext> {
        Ok(ServiceContext::new(
            self.pool.ok_or_else(|| super::error::ServiceError::validation("pool is required"))?,
            self.redis_pool.ok_or_else(|| super::error::ServiceError::validation("redis_pool is required"))?,
            self.user_repo.ok_or_else(|| super::error::ServiceError::validation("user_repo is required"))?,
            self.guild_repo.ok_or_else(|| super::error::ServiceError::validation("guild_repo is required"))?,
            self.channel_repo.ok_or_else(|| super::error::ServiceError::validation("channel_repo is required"))?,
            self.message_repo.ok_or_else(|| super::error::ServiceError::validation("message_repo is required"))?,
            self.role_repo.ok_or_else(|| super::error::ServiceError::validation("role_repo is required"))?,
            self.member_repo.ok_or_else(|| super::error::ServiceError::validation("member_repo is required"))?,
            self.reaction_repo.ok_or_else(|| super::error::ServiceError::validation("reaction_repo is required"))?,
            self.invite_repo.ok_or_else(|| super::error::ServiceError::validation("invite_repo is required"))?,
            self.ban_repo.ok_or_else(|| super::error::ServiceError::validation("ban_repo is required"))?,
            self.attachment_repo.ok_or_else(|| super::error::ServiceError::validation("attachment_repo is required"))?,
            self.jwt_service.ok_or_else(|| super::error::ServiceError::validation("jwt_service is required"))?,
            self.snowflake_generator.ok_or_else(|| super::error::ServiceError::validation("snowflake_generator is required"))?,
        ))
    }
}

impl Default for ServiceContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
