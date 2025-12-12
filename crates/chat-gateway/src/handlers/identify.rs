//! Identify handler (op 2)

use super::{HandlerError, HandlerResult};
use crate::connection::{Connection, Session};
use crate::events::{GatewayEventType, GuildCreateEvent, ReadyEvent, UnavailableGuild, UserPayload};
use crate::protocol::{CloseCode, GatewayMessage, IdentifyPayload};
use crate::server::GatewayState;
use chat_cache::ClientProperties;
use chat_core::Snowflake;
use std::sync::Arc;

/// Handles Identify messages
pub struct IdentifyHandler;

impl IdentifyHandler {
    /// Handle an Identify message
    pub async fn handle(
        state: &GatewayState,
        connection: &Arc<Connection>,
        payload: IdentifyPayload,
    ) -> HandlerResult<Option<CloseCode>> {
        // Check if already authenticated
        if connection.is_authenticated().await {
            tracing::warn!(
                session_id = %connection.session_id(),
                "Client sent Identify while already authenticated"
            );
            return Ok(Some(CloseCode::AlreadyAuthenticated));
        }

        // Extract token (remove "Bearer " prefix if present)
        let token = payload.token.strip_prefix("Bearer ").unwrap_or(&payload.token);

        // Validate the token
        let claims = state
            .service_context()
            .jwt_service()
            .validate_access_token(token)
            .map_err(|e| {
                tracing::debug!(error = %e, "Token validation failed");
                HandlerError::AuthenticationFailed(e.to_string())
            })?;

        let user_id = claims
            .user_id()
            .map_err(|e| HandlerError::AuthenticationFailed(e.to_string()))?;

        // Get user from database
        let user = state
            .service_context()
            .user_repo()
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| HandlerError::AuthenticationFailed("User not found".to_string()))?;

        // Get user's guilds
        let guilds = state
            .service_context()
            .guild_repo()
            .find_by_user(user_id)
            .await?;

        // Convert client properties
        let client_props = payload.properties.map(ClientProperties::from);

        // Create session in Redis
        let session_id = connection.session_id().to_string();
        let resume_url = state.config().gateway.address();

        Session::create(
            state.service_context().session_store(),
            &session_id,
            user_id,
            client_props,
            Some(format!("ws://{}/gateway", resume_url)),
        )
        .await
        .map_err(HandlerError::CacheError)?;

        // Authenticate the connection
        state
            .connection_manager()
            .authenticate_connection(&session_id, user_id)
            .await;

        // Subscribe to guilds
        let guild_ids: Vec<Snowflake> = guilds.iter().map(|g| g.id).collect();
        for guild_id in &guild_ids {
            state
                .connection_manager()
                .subscribe_to_guild(&session_id, *guild_id)
                .await;
            Session::subscribe_guild(state.service_context().session_store(), &session_id, *guild_id)
                .await
                .ok(); // Ignore errors for now
        }

        // Build READY event
        let ready = ReadyEvent {
            v: 1,
            user: UserPayload {
                id: user_id,
                username: user.username.clone(),
                discriminator: user.discriminator.clone(),
                avatar: user.avatar.clone(),
                bot: false,
            },
            guilds: guild_ids.iter().map(|id| UnavailableGuild::new(*id)).collect(),
            session_id: session_id.clone(),
            resume_gateway_url: Some(format!("ws://{}/gateway", resume_url)),
        };

        // Send READY event
        let ready_data = serde_json::to_value(&ready).unwrap_or_default();
        let seq = connection.next_sequence();

        connection
            .send(GatewayMessage::dispatch(
                GatewayEventType::Ready.as_str(),
                seq,
                ready_data.clone(),
            ))
            .await
            .map_err(|e| HandlerError::Internal(format!("Failed to send READY: {e}")))?;

        // Queue event for potential resume
        Session::queue_event(
            state.service_context().session_store(),
            &session_id,
            seq,
            GatewayEventType::Ready.as_str(),
            ready_data,
        )
        .await
        .ok();

        tracing::info!(
            session_id = %session_id,
            user_id = %user_id,
            username = %user.username,
            guilds = guild_ids.len(),
            "Client identified"
        );

        // Send GUILD_CREATE for each guild
        for guild in guilds {
            let guild_create = Self::build_guild_create(state, &guild).await?;
            let guild_data = serde_json::to_value(&guild_create).unwrap_or_default();
            let seq = connection.next_sequence();

            connection
                .send(GatewayMessage::dispatch(
                    GatewayEventType::GuildCreate.as_str(),
                    seq,
                    guild_data.clone(),
                ))
                .await
                .map_err(|e| HandlerError::Internal(format!("Failed to send GUILD_CREATE: {e}")))?;

            // Queue event for potential resume
            Session::queue_event(
                state.service_context().session_store(),
                &session_id,
                seq,
                GatewayEventType::GuildCreate.as_str(),
                guild_data,
            )
            .await
            .ok();
        }

        // Set user presence to online
        let presence_data = chat_cache::PresenceData::new(user_id, chat_cache::UserStatus::Online);
        state
            .service_context()
            .presence_store()
            .set_presence(&presence_data)
            .await
            .ok();

        Ok(None)
    }

    /// Build a GUILD_CREATE event for a guild
    async fn build_guild_create(
        state: &GatewayState,
        guild: &chat_core::Guild,
    ) -> HandlerResult<GuildCreateEvent> {
        use crate::events::{ChannelPayload, MemberPayload, RolePayload};

        // Get channels
        let channels = state
            .service_context()
            .channel_repo()
            .find_by_guild(guild.id)
            .await?;

        // Get roles
        let roles = state
            .service_context()
            .role_repo()
            .find_by_guild(guild.id)
            .await?;

        // Get member count
        let member_count = state
            .service_context()
            .guild_repo()
            .member_count(guild.id)
            .await?;

        // We won't load all members here - just get basic info
        // In a full implementation, you'd paginate or limit this
        let members: Vec<MemberPayload> = vec![];

        Ok(GuildCreateEvent {
            id: guild.id,
            name: guild.name.clone(),
            icon: guild.icon.clone(),
            description: guild.description.clone(),
            owner_id: guild.owner_id,
            channels: channels
                .into_iter()
                .map(|c| ChannelPayload {
                    id: c.id,
                    guild_id: c.guild_id,
                    name: c.name.unwrap_or_default(),
                    channel_type: c.channel_type as i32,
                    position: c.position,
                    topic: c.topic,
                    parent_id: c.parent_id,
                })
                .collect(),
            roles: roles
                .into_iter()
                .map(|r| RolePayload {
                    id: r.id,
                    name: r.name,
                    permissions: r.permissions.bits().to_string(),
                    position: r.position,
                    color: Some(r.color),
                })
                .collect(),
            members,
            member_count: member_count as i32,
            created_at: guild.created_at.to_rfc3339(),
        })
    }
}
