//! Data transfer objects for API requests and responses
//!
//! This module provides:
//! - Request DTOs with validation for API inputs
//! - Response DTOs for serializing API outputs
//! - Mappers for converting domain entities to DTOs

pub mod mappers;
pub mod requests;
pub mod responses;

// Re-export commonly used request types
pub use requests::{
    AddReactionRequest, BulkDeleteMessagesRequest, CreateBanRequest, CreateChannelRequest,
    CreateDmRequest, CreateGuildRequest, CreateInviteRequest, CreateMessageRequest,
    CreateRoleRequest, LoginRequest, LogoutRequest, MessageReference, RefreshTokenRequest,
    RegisterRequest, RolePosition, TypingRequest, UpdateChannelRequest, UpdateGuildRequest,
    UpdateMemberRequest, UpdateMessageRequest, UpdatePresenceRequest, UpdateRoleRequest,
    UpdateRolePositionsRequest, UpdateUserRequest,
};

// Re-export commonly used response types
pub use responses::{
    ApiResponse, AttachmentResponse, AuthResponse, BanResponse, ChannelResponse,
    CurrentUserResponse, DmChannelResponse, GuildPreviewResponse, GuildResponse,
    GuildWithCountsResponse, HealthChecks, HealthResponse, InviteChannelResponse, InviteMinimalResponse,
    InviteResponse, MemberResponse, MessageReferenceResponse, MessageResponse, PaginatedResponse,
    PaginationMeta, PresenceResponse, PublicUserResponse, ReactionResponse, ReadinessResponse,
    RoleResponse, TypingResponse, UserResponse,
};

// Re-export mappers and helper structs
pub use mappers::{
    DmChannelWithRecipients, GuildWithCounts, InviteWithDetails, MemberWithUser,
    MessageReference as MessageReferenceData, MessageWithDetails, ReactionWithMeta,
};
