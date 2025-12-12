//! User presence storage in Redis.
//!
//! Tracks user online status and typing indicators.

use crate::pool::{RedisPool, RedisResult};
use chat_core::Snowflake;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

/// Key prefix for user presence
const PRESENCE_PREFIX: &str = "presence:";
/// Key prefix for typing indicators
const TYPING_PREFIX: &str = "typing:";
/// Key prefix for guild online members
const GUILD_ONLINE_PREFIX: &str = "guild_online:";

/// Typing indicator TTL (10 seconds)
const TYPING_TTL: u64 = 10;
/// Presence TTL (5 minutes - refreshed by heartbeat)
const PRESENCE_TTL: u64 = 300;

/// User online status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// User is online and active
    Online,
    /// User is idle (away from keyboard)
    Idle,
    /// Do not disturb
    Dnd,
    /// User is offline (or invisible)
    Offline,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Offline
    }
}

impl UserStatus {
    /// Check if this status should be visible to others
    #[must_use]
    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::Offline)
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "online"),
            Self::Idle => write!(f, "idle"),
            Self::Dnd => write!(f, "dnd"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "online" => Ok(Self::Online),
            "idle" => Ok(Self::Idle),
            "dnd" => Ok(Self::Dnd),
            "offline" => Ok(Self::Offline),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

/// User presence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceData {
    /// User ID
    pub user_id: Snowflake,
    /// Current status
    pub status: UserStatus,
    /// Custom status text (optional)
    pub custom_status: Option<String>,
    /// Last update timestamp
    pub updated_at: i64,
    /// List of active session IDs
    pub sessions: Vec<String>,
}

impl PresenceData {
    /// Create new presence data
    #[must_use]
    pub fn new(user_id: Snowflake, status: UserStatus) -> Self {
        Self {
            user_id,
            status,
            custom_status: None,
            updated_at: chrono::Utc::now().timestamp(),
            sessions: Vec::new(),
        }
    }

    /// Set custom status
    #[must_use]
    pub fn with_custom_status(mut self, status: impl Into<String>) -> Self {
        self.custom_status = Some(status.into());
        self
    }

    /// Add a session
    pub fn add_session(&mut self, session_id: String) {
        if !self.sessions.contains(&session_id) {
            self.sessions.push(session_id);
        }
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.retain(|s| s != session_id);
    }

    /// Check if user has any active sessions
    #[must_use]
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Typing indicator data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingData {
    /// User ID who is typing
    pub user_id: Snowflake,
    /// Channel ID where typing
    pub channel_id: Snowflake,
    /// Guild ID (None for DMs)
    pub guild_id: Option<Snowflake>,
    /// Typing start timestamp
    pub timestamp: i64,
}

impl TypingData {
    /// Create new typing indicator
    #[must_use]
    pub fn new(user_id: Snowflake, channel_id: Snowflake, guild_id: Option<Snowflake>) -> Self {
        Self {
            user_id,
            channel_id,
            guild_id,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// User presence store
#[derive(Clone)]
pub struct PresenceStore {
    pool: RedisPool,
}

impl PresenceStore {
    /// Create a new presence store
    #[must_use]
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Generate Redis key for user presence
    fn presence_key(user_id: Snowflake) -> String {
        format!("{PRESENCE_PREFIX}{user_id}")
    }

    /// Generate Redis key for typing indicator
    fn typing_key(channel_id: Snowflake, user_id: Snowflake) -> String {
        format!("{TYPING_PREFIX}{channel_id}:{user_id}")
    }

    /// Generate Redis key for guild online members
    fn guild_online_key(guild_id: Snowflake) -> String {
        format!("{GUILD_ONLINE_PREFIX}{guild_id}")
    }

    /// Set user presence
    pub async fn set_presence(&self, presence: &PresenceData) -> RedisResult<()> {
        let key = Self::presence_key(presence.user_id);
        self.pool.set(&key, presence, Some(PRESENCE_TTL)).await?;

        tracing::debug!(
            user_id = %presence.user_id,
            status = %presence.status,
            "Set user presence"
        );

        Ok(())
    }

    /// Get user presence
    pub async fn get_presence(&self, user_id: Snowflake) -> RedisResult<Option<PresenceData>> {
        let key = Self::presence_key(user_id);
        self.pool.get_value(&key).await
    }

    /// Update user status
    pub async fn update_status(
        &self,
        user_id: Snowflake,
        status: UserStatus,
    ) -> RedisResult<Option<PresenceData>> {
        if let Some(mut presence) = self.get_presence(user_id).await? {
            presence.status = status;
            presence.touch();
            self.set_presence(&presence).await?;
            Ok(Some(presence))
        } else {
            // Create new presence
            let presence = PresenceData::new(user_id, status);
            self.set_presence(&presence).await?;
            Ok(Some(presence))
        }
    }

    /// Remove user presence (set offline)
    pub async fn remove_presence(&self, user_id: Snowflake) -> RedisResult<bool> {
        let key = Self::presence_key(user_id);
        self.pool.delete(&key).await
    }

    /// Refresh presence TTL (called on heartbeat)
    pub async fn refresh_presence(&self, user_id: Snowflake) -> RedisResult<bool> {
        if let Some(mut presence) = self.get_presence(user_id).await? {
            presence.touch();
            self.set_presence(&presence).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get presence for multiple users
    pub async fn get_presences(&self, user_ids: &[Snowflake]) -> RedisResult<Vec<PresenceData>> {
        let mut presences = Vec::new();
        for user_id in user_ids {
            if let Some(presence) = self.get_presence(*user_id).await? {
                presences.push(presence);
            }
        }
        Ok(presences)
    }

    /// Add user to guild's online set
    pub async fn add_to_guild_online(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> RedisResult<()> {
        let key = Self::guild_online_key(guild_id);
        let mut conn = self.pool.get().await?;
        conn.sadd::<_, _, ()>(&key, user_id.to_string()).await?;
        Ok(())
    }

    /// Remove user from guild's online set
    pub async fn remove_from_guild_online(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
    ) -> RedisResult<()> {
        let key = Self::guild_online_key(guild_id);
        let mut conn = self.pool.get().await?;
        conn.srem::<_, _, ()>(&key, user_id.to_string()).await?;
        Ok(())
    }

    /// Get all online users in a guild
    pub async fn get_guild_online_users(&self, guild_id: Snowflake) -> RedisResult<Vec<Snowflake>> {
        let key = Self::guild_online_key(guild_id);
        let mut conn = self.pool.get().await?;
        let user_ids: Vec<String> = conn.smembers(&key).await?;

        let mut result = Vec::new();
        for id_str in user_ids {
            if let Ok(id) = id_str.parse::<i64>() {
                result.push(Snowflake::from(id));
            }
        }
        Ok(result)
    }

    /// Get count of online users in a guild
    pub async fn get_guild_online_count(&self, guild_id: Snowflake) -> RedisResult<u64> {
        let key = Self::guild_online_key(guild_id);
        let mut conn = self.pool.get().await?;
        let count: u64 = conn.scard(&key).await?;
        Ok(count)
    }

    /// Set typing indicator
    pub async fn set_typing(&self, typing: &TypingData) -> RedisResult<()> {
        let key = Self::typing_key(typing.channel_id, typing.user_id);
        self.pool.set(&key, typing, Some(TYPING_TTL)).await?;

        tracing::trace!(
            user_id = %typing.user_id,
            channel_id = %typing.channel_id,
            "Set typing indicator"
        );

        Ok(())
    }

    /// Get typing indicator
    pub async fn get_typing(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> RedisResult<Option<TypingData>> {
        let key = Self::typing_key(channel_id, user_id);
        self.pool.get_value(&key).await
    }

    /// Remove typing indicator
    pub async fn remove_typing(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> RedisResult<bool> {
        let key = Self::typing_key(channel_id, user_id);
        self.pool.delete(&key).await
    }

    /// Get all users typing in a channel
    pub async fn get_channel_typing(&self, channel_id: Snowflake) -> RedisResult<Vec<TypingData>> {
        let pattern = format!("{TYPING_PREFIX}{channel_id}:*");
        let keys = self.pool.scan_keys(&pattern, 100).await?;

        let mut typing = Vec::new();
        for key in keys {
            if let Some(data) = self.pool.get_value::<TypingData>(&key).await? {
                typing.push(data);
            }
        }

        Ok(typing)
    }

    /// Check if user is typing in channel
    pub async fn is_typing(
        &self,
        channel_id: Snowflake,
        user_id: Snowflake,
    ) -> RedisResult<bool> {
        let key = Self::typing_key(channel_id, user_id);
        self.pool.exists(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status_display() {
        assert_eq!(UserStatus::Online.to_string(), "online");
        assert_eq!(UserStatus::Idle.to_string(), "idle");
        assert_eq!(UserStatus::Dnd.to_string(), "dnd");
        assert_eq!(UserStatus::Offline.to_string(), "offline");
    }

    #[test]
    fn test_user_status_parse() {
        assert_eq!("online".parse::<UserStatus>().unwrap(), UserStatus::Online);
        assert_eq!("IDLE".parse::<UserStatus>().unwrap(), UserStatus::Idle);
        assert_eq!("DnD".parse::<UserStatus>().unwrap(), UserStatus::Dnd);
        assert!("invalid".parse::<UserStatus>().is_err());
    }

    #[test]
    fn test_user_status_visibility() {
        assert!(UserStatus::Online.is_visible());
        assert!(UserStatus::Idle.is_visible());
        assert!(UserStatus::Dnd.is_visible());
        assert!(!UserStatus::Offline.is_visible());
    }

    #[test]
    fn test_presence_data_creation() {
        let user_id = Snowflake::from(12345i64);
        let mut presence = PresenceData::new(user_id, UserStatus::Online)
            .with_custom_status("Playing a game");

        assert_eq!(presence.user_id, user_id);
        assert_eq!(presence.status, UserStatus::Online);
        assert_eq!(presence.custom_status, Some("Playing a game".to_string()));
        assert!(!presence.has_sessions());

        presence.add_session("session1".to_string());
        assert!(presence.has_sessions());
        assert_eq!(presence.sessions.len(), 1);

        presence.remove_session("session1");
        assert!(!presence.has_sessions());
    }

    #[test]
    fn test_typing_data_creation() {
        let user_id = Snowflake::from(12345i64);
        let channel_id = Snowflake::from(67890i64);
        let guild_id = Snowflake::from(11111i64);

        let typing = TypingData::new(user_id, channel_id, Some(guild_id));

        assert_eq!(typing.user_id, user_id);
        assert_eq!(typing.channel_id, channel_id);
        assert_eq!(typing.guild_id, Some(guild_id));
    }

    #[test]
    fn test_key_generation() {
        let user_id = Snowflake::from(12345i64);
        let channel_id = Snowflake::from(67890i64);
        let guild_id = Snowflake::from(11111i64);

        assert_eq!(
            PresenceStore::presence_key(user_id),
            format!("presence:{user_id}")
        );
        assert_eq!(
            PresenceStore::typing_key(channel_id, user_id),
            format!("typing:{channel_id}:{user_id}")
        );
        assert_eq!(
            PresenceStore::guild_online_key(guild_id),
            format!("guild_online:{guild_id}")
        );
    }
}
