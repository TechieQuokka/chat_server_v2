//! # chat-service
//!
//! Application layer containing business logic, services, and DTOs.
//!
//! This crate provides the service layer for the chat application,
//! implementing all business logic, validation, and orchestration.
//!
//! ## Services
//!
//! - [`AuthService`] - Authentication (register, login, logout, token refresh)
//! - [`PermissionService`] - Permission checking and role hierarchy
//! - [`UserService`] - User profile management
//! - [`GuildService`] - Guild (server) CRUD operations
//! - [`ChannelService`] - Channel management within guilds
//! - [`MessageService`] - Message creation, editing, deletion
//! - [`MemberService`] - Guild member and ban management
//! - [`RoleService`] - Role creation and assignment
//! - [`ReactionService`] - Message reactions
//! - [`InviteService`] - Guild invitations
//! - [`DmService`] - Direct message channels
//! - [`PresenceService`] - User online status
//!
//! ## DTOs
//!
//! Request and response types are in the [`dto`] module, with validation
//! via the `validator` crate and serialization via `serde`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use chat_service::services::{ServiceContext, ServiceContextBuilder, AuthService};
//! use chat_service::dto::{RegisterRequest, AuthResponse};
//!
//! // Build service context with all dependencies
//! let ctx = ServiceContextBuilder::new()
//!     .pool(pg_pool)
//!     .redis_pool(redis_pool)
//!     // ... add repositories
//!     .build()?;
//!
//! // Use services
//! let auth_service = AuthService::new(&ctx);
//! let response = auth_service.register(request).await?;
//! ```

pub mod dto;
pub mod services;

// Re-export DTOs
pub use dto::{
    // Request types
    AddReactionRequest, BulkDeleteMessagesRequest, CreateBanRequest, CreateChannelRequest,
    CreateDmRequest, CreateGuildRequest, CreateInviteRequest, CreateMessageRequest,
    CreateRoleRequest, LoginRequest, LogoutRequest, MessageReference, RefreshTokenRequest,
    RegisterRequest, RolePosition, TypingRequest, UpdateChannelRequest, UpdateGuildRequest,
    UpdateMemberRequest, UpdateMessageRequest, UpdatePresenceRequest, UpdateRoleRequest,
    UpdateRolePositionsRequest, UpdateUserRequest,
    // Response types
    ApiResponse, AttachmentResponse, AuthResponse, BanResponse, ChannelResponse,
    CurrentUserResponse, DmChannelResponse, GuildPreviewResponse, GuildResponse,
    GuildWithCountsResponse, HealthChecks, HealthResponse, InviteChannelResponse, InviteMinimalResponse,
    InviteResponse, MemberResponse, MessageReferenceResponse, MessageResponse, PaginatedResponse,
    PaginationMeta, PresenceResponse, PublicUserResponse, ReactionResponse, ReadinessResponse,
    RoleResponse, TypingResponse, UserResponse,
    // Helper types
    DmChannelWithRecipients, GuildWithCounts, InviteWithDetails, MemberWithUser, MessageWithDetails,
    ReactionWithMeta,
};

// Re-export services
pub use services::{
    AuthService, ChannelService, DmService, GuildService, InviteService, MemberService,
    MessageService, PermissionService, PresenceService, ReactionService, RoleService,
    ServiceContext, ServiceContextBuilder, ServiceError, ServiceResult, UserService,
};
