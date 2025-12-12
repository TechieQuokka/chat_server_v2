# Core Data Models Design

## Discord-like Chat Server - Rust Struct Definitions

This document defines all Rust data structures for the chat server, including domain entities, database models, API DTOs, and WebSocket payloads.

---

## Table of Contents

1. [Design Principles](#design-principles)
2. [Value Objects](#value-objects)
3. [Domain Entities](#domain-entities)
4. [Database Models](#database-models)
5. [API DTOs](#api-dtos)
6. [WebSocket Payloads](#websocket-payloads)
7. [Mapping Strategies](#mapping-strategies)

---

## Design Principles

### Separation of Concerns

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Layer                                 │
│  Request DTOs ──► Domain Entities ──► Response DTOs             │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Domain Layer                                │
│  Entities (User, Guild, Channel, Message, Role, etc.)           │
│  Value Objects (Snowflake, Permissions, Email, etc.)            │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                           │
│  Database Models (UserModel, GuildModel, etc.)                  │
│  Cache Models (PresenceState, SessionData, etc.)                │
└─────────────────────────────────────────────────────────────────┘
```

### Key Guidelines

1. **Domain entities** are framework-agnostic (no Axum/SQLx dependencies)
2. **Database models** use `#[derive(sqlx::FromRow)]`
3. **API DTOs** use `#[derive(serde::Serialize, serde::Deserialize)]`
4. **Snowflake IDs** are `i64` internally, serialized as `String` in JSON
5. **Timestamps** use `chrono::DateTime<Utc>`, serialized as ISO 8601

---

## Value Objects

### Snowflake ID

```rust
// crates/chat-core/src/value_objects/snowflake.rs

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Discord-compatible Snowflake ID (64-bit)
///
/// Structure:
/// - Bits 63-22: Timestamp (milliseconds since epoch)
/// - Bits 21-12: Worker ID (0-1023)
/// - Bits 11-0:  Sequence number (0-4095)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Snowflake(i64);

impl Snowflake {
    /// Custom epoch: 2024-01-01 00:00:00 UTC
    pub const EPOCH: i64 = 1704067200000;

    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    pub const fn into_inner(self) -> i64 {
        self.0
    }

    /// Extract timestamp from Snowflake ID
    pub fn timestamp(&self) -> i64 {
        (self.0 >> 22) + Self::EPOCH
    }

    /// Extract worker ID from Snowflake ID
    pub fn worker_id(&self) -> u16 {
        ((self.0 >> 12) & 0x3FF) as u16
    }

    /// Extract sequence number from Snowflake ID
    pub fn sequence(&self) -> u16 {
        (self.0 & 0xFFF) as u16
    }
}

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Serialize as string for JSON (JavaScript BigInt safety)
impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

// Deserialize from string or number
impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Implementation handles both string and number
        unimplemented!()
    }
}

// SQLx support
impl sqlx::Type<sqlx::Postgres> for Snowflake {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for Snowflake {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.0.encode_by_ref(buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for Snowflake {
    fn decode(
        value: sqlx::postgres::PgValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Snowflake(<i64 as sqlx::Decode<sqlx::Postgres>>::decode(value)?))
    }
}
```

### Snowflake Generator

```rust
// crates/chat-core/src/value_objects/snowflake_generator.rs

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SnowflakeGenerator {
    worker_id: u16,
    sequence: AtomicI64,
    last_timestamp: AtomicI64,
}

impl SnowflakeGenerator {
    pub fn new(worker_id: u16) -> Self {
        assert!(worker_id < 1024, "Worker ID must be < 1024");
        Self {
            worker_id,
            sequence: AtomicI64::new(0),
            last_timestamp: AtomicI64::new(0),
        }
    }

    pub fn generate(&self) -> Snowflake {
        let mut timestamp = self.current_timestamp();
        let last = self.last_timestamp.load(Ordering::SeqCst);

        let sequence = if timestamp == last {
            let seq = self.sequence.fetch_add(1, Ordering::SeqCst) & 0xFFF;
            if seq == 0 {
                // Sequence overflow, wait for next millisecond
                while timestamp <= last {
                    timestamp = self.current_timestamp();
                }
            }
            seq
        } else {
            self.sequence.store(0, Ordering::SeqCst);
            0
        };

        self.last_timestamp.store(timestamp, Ordering::SeqCst);

        let id = ((timestamp - Snowflake::EPOCH) << 22)
            | ((self.worker_id as i64) << 12)
            | sequence;

        Snowflake::new(id)
    }

    fn current_timestamp(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
}
```

### Permissions Bitfield

```rust
// crates/chat-core/src/value_objects/permissions.rs

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Permissions: u64 {
        /// View channel and read messages
        const VIEW_CHANNEL     = 1 << 0;
        /// Send messages in text channels
        const SEND_MESSAGES    = 1 << 1;
        /// Delete other users' messages
        const MANAGE_MESSAGES  = 1 << 2;
        /// Create, edit, delete channels
        const MANAGE_CHANNELS  = 1 << 3;
        /// Create, edit, delete, assign roles
        const MANAGE_ROLES     = 1 << 4;
        /// Edit guild settings
        const MANAGE_GUILD     = 1 << 5;
        /// Kick members from guild
        const KICK_MEMBERS     = 1 << 6;
        /// Ban members from guild
        const BAN_MEMBERS      = 1 << 7;
        /// Bypass all permission checks
        const ADMINISTRATOR    = 1 << 8;
        /// Upload files and images
        const ATTACH_FILES     = 1 << 9;
        /// Add emoji reactions
        const ADD_REACTIONS    = 1 << 10;

        /// Default permissions for @everyone
        const DEFAULT = Self::VIEW_CHANNEL.bits()
            | Self::SEND_MESSAGES.bits()
            | Self::ADD_REACTIONS.bits()
            | Self::ATTACH_FILES.bits();

        /// All permissions (for administrators)
        const ALL = u64::MAX;
    }
}

impl Permissions {
    /// Check if the permission set contains a required permission
    pub fn has(&self, permission: Permissions) -> bool {
        if self.contains(Permissions::ADMINISTRATOR) {
            return true;
        }
        self.contains(permission)
    }

    /// Combine permissions from multiple roles
    pub fn combine(roles: impl IntoIterator<Item = Permissions>) -> Self {
        roles.into_iter().fold(Permissions::empty(), |acc, p| acc | p)
    }
}

// SQLx support - stored as BIGINT
impl sqlx::Type<sqlx::Postgres> for Permissions {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}
```

### Validated Types

```rust
// crates/chat-core/src/value_objects/email.rs

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
#[serde(transparent)]
pub struct Email(#[validate(email)] String);

impl Email {
    pub fn new(email: impl Into<String>) -> Result<Self, &'static str> {
        let email = email.into();
        let instance = Self(email);
        instance.validate().map_err(|_| "Invalid email format")?;
        Ok(instance)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// crates/chat-core/src/value_objects/username.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Validate)]
#[serde(transparent)]
pub struct Username(
    #[validate(length(min = 2, max = 32))]
    #[validate(regex(path = "USERNAME_REGEX"))]
    String
);

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex =
        regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}

impl Username {
    pub fn new(username: impl Into<String>) -> Result<Self, &'static str> {
        let username = username.into();
        let instance = Self(username);
        instance.validate().map_err(|_| "Invalid username")?;
        Ok(instance)
    }
}
```

---

## Domain Entities

### User Entity

```rust
// crates/chat-core/src/entities/user.rs

use chrono::{DateTime, Utc};
use crate::value_objects::{Snowflake, Email, Username};

#[derive(Debug, Clone)]
pub struct User {
    pub id: Snowflake,
    pub username: Username,
    pub discriminator: String,
    pub email: Email,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Full tag: username#discriminator
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username.as_str(), self.discriminator)
    }

    /// Avatar URL or default
    pub fn avatar_url(&self) -> String {
        match &self.avatar {
            Some(hash) => format!("/avatars/{}/{}.png", self.id, hash),
            None => format!("/embed/avatars/{}.png", self.default_avatar_index()),
        }
    }

    fn default_avatar_index(&self) -> u8 {
        // Based on discriminator
        self.discriminator.parse::<u16>().unwrap_or(0) as u8 % 5
    }
}
```

### Guild Entity

```rust
// crates/chat-core/src/entities/guild.rs

use chrono::{DateTime, Utc};
use crate::value_objects::Snowflake;

#[derive(Debug, Clone)]
pub struct Guild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: Snowflake,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guild {
    pub fn is_owner(&self, user_id: Snowflake) -> bool {
        self.owner_id == user_id
    }

    pub fn icon_url(&self) -> Option<String> {
        self.icon.as_ref().map(|hash| {
            format!("/icons/{}/{}.png", self.id, hash)
        })
    }
}
```

### Channel Entity

```rust
// crates/chat-core/src/entities/channel.rs

use chrono::{DateTime, Utc};
use crate::value_objects::Snowflake;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChannelType {
    GuildText = 0,
    Dm = 1,
    GuildCategory = 4,
}

impl From<i16> for ChannelType {
    fn from(value: i16) -> Self {
        match value {
            0 => ChannelType::GuildText,
            1 => ChannelType::Dm,
            4 => ChannelType::GuildCategory,
            _ => ChannelType::GuildText,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub name: Option<String>,
    pub channel_type: ChannelType,
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<Snowflake>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Channel {
    pub fn is_text(&self) -> bool {
        matches!(self.channel_type, ChannelType::GuildText | ChannelType::Dm)
    }

    pub fn is_category(&self) -> bool {
        matches!(self.channel_type, ChannelType::GuildCategory)
    }

    pub fn is_dm(&self) -> bool {
        matches!(self.channel_type, ChannelType::Dm)
    }
}
```

### Message Entity

```rust
// crates/chat-core/src/entities/message.rs

use chrono::{DateTime, Utc};
use crate::value_objects::Snowflake;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub author_id: Snowflake,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub reference_id: Option<Snowflake>,
}

impl Message {
    pub fn is_edited(&self) -> bool {
        self.edited_at.is_some()
    }

    pub fn is_reply(&self) -> bool {
        self.reference_id.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: Snowflake,
    pub message_id: Snowflake,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    pub proxy_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
```

### Role Entity

```rust
// crates/chat-core/src/entities/role.rs

use crate::value_objects::{Snowflake, Permissions};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Role {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: Permissions,
    pub mentionable: bool,
    pub is_everyone: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    /// Check if this role grants a specific permission
    pub fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.has(permission)
    }

    /// Compare role positions for hierarchy
    pub fn is_higher_than(&self, other: &Role) -> bool {
        self.position > other.position
    }
}
```

### Member Entity

```rust
// crates/chat-core/src/entities/member.rs

use chrono::{DateTime, Utc};
use crate::value_objects::Snowflake;

#[derive(Debug, Clone)]
pub struct GuildMember {
    pub guild_id: Snowflake,
    pub user_id: Snowflake,
    pub nickname: Option<String>,
    pub role_ids: Vec<Snowflake>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GuildMember {
    pub fn display_name<'a>(&'a self, username: &'a str) -> &'a str {
        self.nickname.as_deref().unwrap_or(username)
    }

    pub fn has_role(&self, role_id: Snowflake) -> bool {
        self.role_ids.contains(&role_id)
    }
}
```

---

## Database Models

Database models use SQLx's `FromRow` derive for automatic mapping.

### User Model

```rust
// crates/chat-db/src/models/user_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserModel {
    pub id: i64,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    pub password_hash: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl UserModel {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
```

### Guild Model

```rust
// crates/chat-db/src/models/guild_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct GuildModel {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### Channel Model

```rust
// crates/chat-db/src/models/channel_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ChannelModel {
    pub id: i64,
    pub guild_id: Option<i64>,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub channel_type: String,  // PostgreSQL ENUM as string
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### Message Model

```rust
// crates/chat-db/src/models/message_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct MessageModel {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub reference_id: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AttachmentModel {
    pub id: i64,
    pub message_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    pub proxy_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}
```

### Role Model

```rust
// crates/chat-db/src/models/role_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct RoleModel {
    pub id: i64,
    pub guild_id: i64,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: i64,  // Bitfield stored as BIGINT
    pub mentionable: bool,
    pub is_everyone: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### Member Model

```rust
// crates/chat-db/src/models/member_model.rs

use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct GuildMemberModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub nickname: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MemberRoleModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub role_id: i64,
    pub assigned_at: DateTime<Utc>,
}

/// Denormalized view model with roles included
#[derive(Debug, Clone, FromRow)]
pub struct MemberWithRolesModel {
    pub guild_id: i64,
    pub user_id: i64,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub nickname: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub role_ids: Vec<i64>,  // From array_agg
}
```

---

## API DTOs

### Request DTOs

```rust
// crates/chat-service/src/dto/requests.rs

use serde::Deserialize;
use validator::Validate;

// === Auth ===

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 32))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 72))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// === User ===

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2, max = 32))]
    pub username: Option<String>,
    pub avatar: Option<String>,
}

// === Guild ===

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGuildRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGuildRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<String>,  // Transfer ownership
}

// === Channel ===

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: Option<i32>,  // 0 = text, 4 = category
    pub topic: Option<String>,
    pub parent_id: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<i32>,
    pub parent_id: Option<String>,
}

// === Message ===

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
    pub message_reference: Option<MessageReference>,
}

#[derive(Debug, Deserialize)]
pub struct MessageReference {
    pub message_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 2000))]
    pub content: String,
}

// === Role ===

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub color: Option<i32>,
    pub hoist: Option<bool>,
    pub permissions: Option<String>,  // Permissions bitfield as string
    pub mentionable: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub color: Option<i32>,
    pub hoist: Option<bool>,
    pub permissions: Option<String>,
    pub mentionable: Option<bool>,
    pub position: Option<i32>,
}

// === Member ===

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub nickname: Option<String>,
    pub roles: Option<Vec<String>>,  // Role IDs to set
}

// === Invite ===

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_age: Option<i32>,    // Seconds (0 = never)
    pub max_uses: Option<i32>,   // 0 = unlimited
    pub temporary: Option<bool>,
}
```

### Response DTOs

```rust
// crates/chat-service/src/dto/responses.rs

use chrono::{DateTime, Utc};
use serde::Serialize;

// === Common ===

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub before: Option<String>,
    pub after: Option<String>,
    pub has_more: bool,
    pub limit: i32,
}

// === Auth ===

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: UserResponse,
}

// === User ===

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub email: String,
    pub avatar: Option<String>,
    pub bot: bool,
    pub system: bool,
    pub created_at: DateTime<Utc>,
}

// === Guild ===

#[derive(Debug, Serialize)]
pub struct GuildResponse {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GuildWithCountsResponse {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub owner_id: String,
    pub member_count: i32,
    pub channel_count: i32,
    pub created_at: DateTime<Utc>,
}

// === Channel ===

#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: String,
    pub guild_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub topic: Option<String>,
    pub position: i32,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DmChannelResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub recipients: Vec<UserResponse>,
    pub last_message_id: Option<String>,
}

// === Message ===

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author: UserResponse,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub edited_timestamp: Option<DateTime<Utc>>,
    pub attachments: Vec<AttachmentResponse>,
    pub reactions: Vec<ReactionResponse>,
    pub message_reference: Option<MessageReferenceResponse>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    pub proxy_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub emoji: String,
    pub count: i32,
    pub me: bool,
}

#[derive(Debug, Serialize)]
pub struct MessageReferenceResponse {
    pub message_id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
}

// === Role ===

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub color: i32,
    pub hoist: bool,
    pub position: i32,
    pub permissions: String,  // Bitfield as string
    pub mentionable: bool,
}

// === Member ===

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub user: UserResponse,
    pub nickname: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: DateTime<Utc>,
}

// === Invite ===

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub code: String,
    pub guild: GuildResponse,
    pub channel: ChannelResponse,
    pub inviter: UserResponse,
    pub uses: i32,
    pub max_uses: Option<i32>,
    pub max_age: Option<i32>,
    pub temporary: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// === Presence ===

#[derive(Debug, Serialize)]
pub struct PresenceResponse {
    pub user_id: String,
    pub status: String,  // online, idle, dnd, offline
    pub last_seen: Option<DateTime<Utc>>,
}
```

### Error Response

```rust
// crates/chat-common/src/error/error_response.rs

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

// Error codes
pub mod error_codes {
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const CONFLICT: &str = "CONFLICT";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";

    // Specific errors
    pub const UNKNOWN_USER: &str = "UNKNOWN_USER";
    pub const UNKNOWN_GUILD: &str = "UNKNOWN_GUILD";
    pub const UNKNOWN_CHANNEL: &str = "UNKNOWN_CHANNEL";
    pub const UNKNOWN_MESSAGE: &str = "UNKNOWN_MESSAGE";
    pub const UNKNOWN_ROLE: &str = "UNKNOWN_ROLE";
    pub const UNKNOWN_INVITE: &str = "UNKNOWN_INVITE";
    pub const MISSING_PERMISSIONS: &str = "MISSING_PERMISSIONS";
    pub const INVALID_TOKEN: &str = "INVALID_TOKEN";
    pub const TOKEN_EXPIRED: &str = "TOKEN_EXPIRED";
}
```

---

## WebSocket Payloads

```rust
// crates/chat-gateway/src/events/payload.rs

use serde::{Deserialize, Serialize};

/// Gateway message format
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayMessage {
    pub op: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<serde_json::Value>,
}

/// Op codes
pub mod opcodes {
    pub const DISPATCH: u8 = 0;
    pub const HEARTBEAT: u8 = 1;
    pub const IDENTIFY: u8 = 2;
    pub const PRESENCE_UPDATE: u8 = 3;
    pub const RESUME: u8 = 4;
    pub const RECONNECT: u8 = 5;
    pub const INVALID_SESSION: u8 = 7;
    pub const HELLO: u8 = 10;
    pub const HEARTBEAT_ACK: u8 = 11;
}

// === Client -> Server Payloads ===

#[derive(Debug, Deserialize)]
pub struct IdentifyPayload {
    pub token: String,
    #[serde(default)]
    pub properties: Option<ConnectionProperties>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionProperties {
    pub os: Option<String>,
    pub browser: Option<String>,
    pub device: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResumePayload {
    pub token: String,
    pub session_id: String,
    pub seq: u64,
}

#[derive(Debug, Deserialize)]
pub struct PresenceUpdatePayload {
    pub status: String,  // online, idle, dnd, offline
}

// === Server -> Client Payloads ===

#[derive(Debug, Serialize)]
pub struct HelloPayload {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Serialize)]
pub struct ReadyPayload {
    pub v: u8,
    pub user: UserResponse,
    pub guilds: Vec<UnavailableGuild>,
    pub session_id: String,
    pub resume_gateway_url: String,
}

#[derive(Debug, Serialize)]
pub struct UnavailableGuild {
    pub id: String,
    pub unavailable: bool,
}

#[derive(Debug, Serialize)]
pub struct InvalidSessionPayload {
    pub resumable: bool,
}

// See docs/websocket.md for complete event payloads
```

---

## Mapping Strategies

### Entity <-> Model Mapping

```rust
// crates/chat-db/src/mappers/user_mapper.rs

use crate::models::UserModel;
use chat_core::{entities::User, value_objects::{Snowflake, Email, Username}};

impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        User {
            id: Snowflake::new(model.id),
            username: Username::new(model.username).unwrap(),
            discriminator: model.discriminator,
            email: Email::new(model.email).unwrap(),
            avatar: model.avatar,
            bot: model.bot,
            system: model.system,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
```

### Entity <-> DTO Mapping

```rust
// crates/chat-service/src/dto/mappers.rs

use crate::dto::responses::UserResponse;
use chat_core::entities::User;

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id.to_string(),
            username: user.username.to_string(),
            discriminator: user.discriminator,
            avatar: user.avatar,
            bot: user.bot,
            system: user.system,
            created_at: user.created_at,
        }
    }
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        UserResponse {
            id: user.id.to_string(),
            username: user.username.to_string(),
            discriminator: user.discriminator.clone(),
            avatar: user.avatar.clone(),
            bot: user.bot,
            system: user.system,
            created_at: user.created_at,
        }
    }
}
```

---

## Summary

| Layer | Location | Derive Macros |
|-------|----------|---------------|
| Value Objects | `chat-core` | `Serialize`, `Deserialize`, custom `sqlx::Type` |
| Domain Entities | `chat-core` | None (plain structs) |
| Database Models | `chat-db` | `sqlx::FromRow` |
| Request DTOs | `chat-service` | `Deserialize`, `Validate` |
| Response DTOs | `chat-service` | `Serialize` |
| WebSocket Payloads | `chat-gateway` | `Serialize`, `Deserialize` |

### Key Conventions

1. **Snowflake IDs**: `i64` internally, `String` in JSON
2. **Timestamps**: `DateTime<Utc>`, ISO 8601 in JSON
3. **Permissions**: `u64` bitfield, `String` in JSON
4. **Enums**: `#[serde(rename_all = "snake_case")]`
5. **Optional fields**: Use `Option<T>`, serialize as `null`
6. **Validation**: On request DTOs only

---

*Document Version: 1.0.0*
*Last Updated: 2024*
