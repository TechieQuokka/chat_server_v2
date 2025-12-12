# Discord-like Chat Server Specification

> **Version:** 1.0.0-MVP
> **Last Updated:** 2024-01-15
> **Status:** Requirements Complete - Ready for Implementation

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Tech Stack](#2-tech-stack)
3. [Data Model](#3-data-model)
4. [Database Schema](#4-database-schema)
5. [API Design](#5-api-design)
6. [WebSocket Protocol](#6-websocket-protocol)
7. [Permission System](#7-permission-system)
8. [Authentication](#8-authentication)
9. [File Storage](#9-file-storage)
10. [Redis Usage](#10-redis-usage)
11. [Project Structure](#11-project-structure)
12. [Implementation Phases](#12-implementation-phases)

---

## 1. Project Overview

### 1.1 Vision
A Discord-like real-time chat server built in Rust, providing a clean REST API and WebSocket interface for any client implementation.

### 1.2 Scope
- **Server-side only** - Client (Python) built separately
- **Target scale:** 1K-10K concurrent users initially
- **Deployment:** Single server first, designed for horizontal scaling

### 1.3 MVP Features
| Feature | Status |
|---------|--------|
| Text channels | MVP |
| Direct Messages (DMs) | MVP |
| Group DMs | Later |
| Message threads/replies | Later |
| Image uploads (10MB max) | MVP |
| Emoji reactions | MVP |
| Basic search | MVP |
| Presence (online/offline/idle) | MVP |
| Typing indicators | MVP |
| Read receipts | Later |
| Voice/Video | Later |

---

## 2. Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Framework | Axum | Latest |
| Async Runtime | Tokio | Latest |
| WebSocket | axum (built-in ws) + tokio-tungstenite | Latest |
| Database | PostgreSQL | 15+ |
| Cache/PubSub | Redis | 7+ |
| Serialization | serde + serde_json | Latest |
| Password Hashing | argon2 | Latest |
| JWT | jsonwebtoken | Latest |
| Validation | validator | Latest |
| Logging | tracing + tracing-subscriber | Latest |
| OpenAPI | utoipa + utoipa-swagger-ui | Latest |

### 2.1 Key Crates
```toml
[dependencies]
axum = { version = "0.8", features = ["ws", "macros"] }
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.24"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
argon2 = "0.5"
validator = { version = "0.18", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
chrono = { version = "0.4", features = ["serde"] }
futures-util = "0.3"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
thiserror = "2"
anyhow = "1"
```

---

## 3. Data Model

### 3.1 Entity Relationship Diagram
```
┌──────────┐       ┌──────────────┐       ┌──────────┐
│  Users   │◄──────│ GuildMembers │──────►│  Guilds  │
└──────────┘       └──────────────┘       └──────────┘
     │                    │                     │
     │                    │                     │
     ▼                    ▼                     ▼
┌──────────┐       ┌──────────────┐       ┌──────────┐
│ Messages │       │  MemberRoles │       │ Channels │
└──────────┘       └──────────────┘       └──────────┘
     │                    │                     │
     │                    ▼                     │
     │             ┌──────────────┐             │
     │             │    Roles     │             │
     │             └──────────────┘             │
     ▼                                         │
┌──────────┐       ┌──────────────┐            │
│Reactions │       │  DMChannels  │◄───────────┘
└──────────┘       └──────────────┘
```

### 3.2 Core Entities

| Entity | Description |
|--------|-------------|
| User | Platform user account |
| Guild | Server/community (Discord's "Server") |
| Channel | Text channel within a guild |
| Category | Channel grouping (optional parent) |
| Message | Text message in a channel |
| Role | Permission group within a guild |
| GuildMember | User's membership in a guild |
| Reaction | Emoji reaction on a message |
| DMChannel | Direct message channel between users |
| Invite | Guild invitation link |
| AuditLog | Action audit trail |

### 3.3 ID Strategy: Snowflake IDs

64-bit integer IDs with embedded timestamp:
```
 63         22 21    12 11       0
+------------+---------+----------+
|  Timestamp | Worker  | Sequence |
|  (41 bits) | (10 bit)| (12 bits)|
+------------+---------+----------+
```

- **Timestamp:** Milliseconds since custom epoch (2024-01-01)
- **Worker ID:** Server/process identifier (0-1023)
- **Sequence:** Counter for same-millisecond IDs (0-4095)

**Benefits:**
- Chronologically sortable
- No coordination needed for distributed generation
- 64-bit (efficient indexing)
- Discord-compatible

---

## 4. Database Schema

See `schema.sql` for complete PostgreSQL schema.

### 4.1 Key Design Decisions
- **Soft deletes:** `deleted_at` timestamp (nullable)
- **Audit trail:** `created_at`, `updated_at` on all tables
- **Message editing:** `edited_at` timestamp
- **Adjacency list:** Categories as optional `parent_id` on channels
- **Snowflake IDs:** `BIGINT` primary keys
- **JSON in JSON:** Strings stored as-is, IDs as strings in API responses

---

## 5. API Design

### 5.1 Base URL & Versioning
```
Base URL: /api/v1
Versioning: URL path
```

### 5.2 Request/Response Conventions

| Convention | Format |
|------------|--------|
| Timestamps | ISO 8601 (`"2024-01-15T10:30:00Z"`) |
| Snowflake IDs | String (`"123456789012345678"`) |
| Nullable fields | Explicit `null` |
| Empty arrays | Include `[]` |
| Pagination | Cursor-based (`before`/`after` + `limit`) |
| Default page size | 50 |
| Max page size | 100 |

### 5.3 Error Response Format
```json
{
  "code": 50035,
  "message": "Invalid Form Body",
  "errors": {
    "name": {
      "_errors": [
        {"code": "BASE_TYPE_REQUIRED", "message": "This field is required"}
      ]
    }
  }
}
```

### 5.4 HTTP Status Codes
| Code | Usage |
|------|-------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful delete) |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized |
| 403 | Forbidden (permission denied) |
| 404 | Not Found |
| 429 | Rate Limited |
| 500 | Internal Server Error |

### 5.5 Endpoints

#### Authentication
```
POST   /api/v1/auth/register     - Create account
POST   /api/v1/auth/login        - Login (returns JWT)
POST   /api/v1/auth/refresh      - Refresh access token
POST   /api/v1/auth/logout       - Invalidate refresh token
```

#### Users
```
GET    /api/v1/users/@me         - Get current user
PATCH  /api/v1/users/@me         - Update current user
GET    /api/v1/users/{user_id}   - Get user by ID
```

#### Guilds
```
POST   /api/v1/guilds                      - Create guild
GET    /api/v1/guilds/{guild_id}           - Get guild
PATCH  /api/v1/guilds/{guild_id}           - Update guild
DELETE /api/v1/guilds/{guild_id}           - Delete guild
GET    /api/v1/users/@me/guilds            - List user's guilds
```

#### Channels
```
GET    /api/v1/guilds/{guild_id}/channels  - List guild channels
POST   /api/v1/guilds/{guild_id}/channels  - Create channel
GET    /api/v1/channels/{channel_id}       - Get channel
PATCH  /api/v1/channels/{channel_id}       - Update channel
DELETE /api/v1/channels/{channel_id}       - Delete channel
```

#### Messages
```
GET    /api/v1/channels/{channel_id}/messages                    - List messages
POST   /api/v1/channels/{channel_id}/messages                    - Send message
GET    /api/v1/channels/{channel_id}/messages/{message_id}       - Get message
PATCH  /api/v1/channels/{channel_id}/messages/{message_id}       - Edit message
DELETE /api/v1/channels/{channel_id}/messages/{message_id}       - Delete message
```

#### Members
```
GET    /api/v1/guilds/{guild_id}/members              - List members
GET    /api/v1/guilds/{guild_id}/members/{user_id}    - Get member
PATCH  /api/v1/guilds/{guild_id}/members/{user_id}    - Update member (nickname, roles)
DELETE /api/v1/guilds/{guild_id}/members/{user_id}    - Kick member
```

#### Roles
```
GET    /api/v1/guilds/{guild_id}/roles              - List roles
POST   /api/v1/guilds/{guild_id}/roles              - Create role
PATCH  /api/v1/guilds/{guild_id}/roles/{role_id}    - Update role
DELETE /api/v1/guilds/{guild_id}/roles/{role_id}    - Delete role
```

#### Reactions
```
PUT    /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me    - Add reaction
DELETE /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}/@me    - Remove reaction
GET    /api/v1/channels/{channel_id}/messages/{message_id}/reactions/{emoji}        - Get reactors
```

#### DMs
```
POST   /api/v1/users/@me/channels    - Create/get DM channel
GET    /api/v1/users/@me/channels    - List DM channels
```

#### Invites
```
POST   /api/v1/guilds/{guild_id}/invites    - Create invite
GET    /api/v1/invites/{code}               - Get invite info
POST   /api/v1/invites/{code}               - Accept invite (join guild)
DELETE /api/v1/invites/{code}               - Delete invite
```

#### Bans
```
GET    /api/v1/guilds/{guild_id}/bans              - List bans
PUT    /api/v1/guilds/{guild_id}/bans/{user_id}    - Ban user
DELETE /api/v1/guilds/{guild_id}/bans/{user_id}    - Unban user
```

#### Health
```
GET    /health         - Basic health check
GET    /health/ready   - Readiness (DB + Redis connected)
```

### 5.6 Rate Limit Headers (Design for future)
```
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 49
X-RateLimit-Reset: 1705315800
X-RateLimit-Bucket: channel_messages
```

---

## 6. WebSocket Protocol

### 6.1 Connection URL
```
ws://server/gateway
wss://server/gateway (production)
```

### 6.2 Message Format
```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 42,
  "d": { ... }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `op` | integer | Operation code |
| `t` | string | Event name (only for op=0) |
| `s` | integer | Sequence number (only for op=0) |
| `d` | object | Event data payload |

### 6.3 Op Codes

| Op | Name | Direction | Description |
|----|------|-----------|-------------|
| 0 | Dispatch | S→C | Event dispatch |
| 1 | Heartbeat | Both | Keep-alive ping |
| 2 | Identify | C→S | Initial authentication |
| 3 | Presence Update | C→S | Update online status |
| 4 | Resume | C→S | Resume dropped connection |
| 5 | Reconnect | S→C | Server requests reconnection |
| 7 | Invalid Session | S→C | Session invalid, re-identify |
| 10 | Hello | S→C | Sent on connect with heartbeat interval |
| 11 | Heartbeat ACK | S→C | Heartbeat acknowledged |

### 6.4 Connection Lifecycle

```
Client                                    Server
   |                                         |
   |-------- TCP/WebSocket Connect --------->|
   |                                         |
   |<------------- Hello (op=10) ------------|
   |         {heartbeat_interval: 45000}     |
   |                                         |
   |------------ Identify (op=2) ----------->|
   |         {token: "..."}                  |
   |                                         |
   |<------------- Ready (op=0) -------------|
   |         {user, guilds, ...}             |
   |                                         |
   |<----------- Guild Create (op=0) --------|
   |         (for each guild)                |
   |                                         |
   |----------- Heartbeat (op=1) ----------->|
   |<-------- Heartbeat ACK (op=11) ---------|
   |                                         |
   |<------- Message Create (op=0) ----------|
   |         {channel_id, content, ...}      |
   |                                         |
```

### 6.5 Resume Flow
```
Client                                    Server
   |                                         |
   |-------- Connection Lost ----------------|
   |                                         |
   |-------- Reconnect ---------------------->|
   |                                         |
   |<------------- Hello (op=10) ------------|
   |                                         |
   |------------ Resume (op=4) ------------->|
   |    {token, session_id, seq}             |
   |                                         |
   |<--------- Missed Events (op=0) ---------|
   |<--------- Missed Events (op=0) ---------|
   |<----------- Resumed (op=0) -------------|
```

**Session resumable window:** 2 minutes

### 6.6 Gateway Events

#### Connection Events
| Event | Description |
|-------|-------------|
| `READY` | Initial state after identify |
| `RESUMED` | Successfully resumed session |

#### Guild Events
| Event | Description |
|-------|-------------|
| `GUILD_CREATE` | Guild available (on connect or join) |
| `GUILD_UPDATE` | Guild settings changed |
| `GUILD_DELETE` | Guild unavailable (left, kicked, deleted) |

#### Channel Events
| Event | Description |
|-------|-------------|
| `CHANNEL_CREATE` | New channel created |
| `CHANNEL_UPDATE` | Channel settings changed |
| `CHANNEL_DELETE` | Channel deleted |

#### Member Events
| Event | Description |
|-------|-------------|
| `GUILD_MEMBER_ADD` | User joined guild |
| `GUILD_MEMBER_UPDATE` | Member updated (roles, nickname) |
| `GUILD_MEMBER_REMOVE` | User left/kicked/banned |

#### Message Events
| Event | Description |
|-------|-------------|
| `MESSAGE_CREATE` | New message |
| `MESSAGE_UPDATE` | Message edited |
| `MESSAGE_DELETE` | Message deleted |
| `MESSAGE_REACTION_ADD` | Reaction added |
| `MESSAGE_REACTION_REMOVE` | Reaction removed |

#### Presence Events
| Event | Description |
|-------|-------------|
| `PRESENCE_UPDATE` | User status changed |
| `TYPING_START` | User started typing |

#### User Events
| Event | Description |
|-------|-------------|
| `USER_UPDATE` | Current user updated |

### 6.7 Event Payloads

#### READY
```json
{
  "op": 0,
  "t": "READY",
  "s": 1,
  "d": {
    "v": 1,
    "user": {
      "id": "123456789",
      "username": "user",
      "discriminator": "0001",
      "avatar": "abc123"
    },
    "guilds": [
      {"id": "111", "unavailable": true}
    ],
    "session_id": "session_abc123",
    "resume_gateway_url": "wss://..."
  }
}
```

#### MESSAGE_CREATE
```json
{
  "op": 0,
  "t": "MESSAGE_CREATE",
  "s": 42,
  "d": {
    "id": "987654321",
    "channel_id": "123456789",
    "guild_id": "111222333",
    "author": {
      "id": "444555666",
      "username": "sender",
      "avatar": null
    },
    "content": "Hello, world!",
    "timestamp": "2024-01-15T10:30:00Z",
    "edited_timestamp": null,
    "attachments": [],
    "reactions": []
  }
}
```

#### TYPING_START
```json
{
  "op": 0,
  "t": "TYPING_START",
  "s": 43,
  "d": {
    "channel_id": "123456789",
    "guild_id": "111222333",
    "user_id": "444555666",
    "timestamp": 1705315800
  }
}
```

#### PRESENCE_UPDATE
```json
{
  "op": 0,
  "t": "PRESENCE_UPDATE",
  "s": 44,
  "d": {
    "user": {"id": "444555666"},
    "guild_id": "111222333",
    "status": "online"
  }
}
```

---

## 7. Permission System

### 7.1 Permission Bits
```rust
bitflags! {
    pub struct Permissions: u64 {
        const VIEW_CHANNEL     = 1 << 0;   // See channel exists
        const SEND_MESSAGES    = 1 << 1;   // Send text messages
        const MANAGE_MESSAGES  = 1 << 2;   // Delete others' messages
        const MANAGE_CHANNELS  = 1 << 3;   // Edit/delete channels
        const MANAGE_ROLES     = 1 << 4;   // Create/edit/assign roles
        const MANAGE_GUILD     = 1 << 5;   // Edit guild settings
        const KICK_MEMBERS     = 1 << 6;   // Kick members
        const BAN_MEMBERS      = 1 << 7;   // Ban/unban members
        const ADMINISTRATOR    = 1 << 8;   // Bypass all permission checks
        const ATTACH_FILES     = 1 << 9;   // Upload files/images
        const ADD_REACTIONS    = 1 << 10;  // Add emoji reactions
    }
}
```

### 7.2 Default Permission Values
```rust
// @everyone role default
const EVERYONE_DEFAULT: u64 =
    VIEW_CHANNEL | SEND_MESSAGES | ADD_REACTIONS | ATTACH_FILES;

// Owner (cannot be deleted, always has all permissions)
const OWNER_PERMISSIONS: u64 = u64::MAX;
```

### 7.3 Permission Resolution (MVP - Simplified)
```
1. If user is guild owner → GRANT ALL
2. If user has role with ADMINISTRATOR → GRANT ALL
3. Combine all user's role permissions (bitwise OR)
4. Check if required permission bit is set
```

**Note:** Channel permission overwrites deferred to later phase.

### 7.4 Role Hierarchy
- Roles have a `position` field (higher = more authority)
- Users can only assign roles lower than their highest role
- Guild owner bypasses hierarchy checks

---

## 8. Authentication

### 8.1 User Authentication
- **Method:** Username/Password with JWT
- **Access Token:** Short-lived (15 minutes)
- **Refresh Token:** Long-lived (7 days), stored in Redis

### 8.2 JWT Structure
```json
{
  "sub": "user_id",
  "exp": 1705316700,
  "iat": 1705315800,
  "type": "access"
}
```

### 8.3 Bot Authentication
- **Method:** Long-lived token (Discord-style)
- **Format:** `Bot {token}`
- **Token:** Base64-encoded, includes bot user ID

### 8.4 WebSocket Authentication
1. Connect to gateway
2. Receive `Hello` (op=10)
3. Send `Identify` (op=2) with access token
4. Receive `Ready` (op=0) on success

---

## 9. File Storage

### 9.1 MVP Implementation
- **Storage:** Local filesystem
- **Path pattern:** `uploads/{channel_id}/{message_id}/{filename}`
- **Max size:** 10MB
- **Allowed types:** PNG, JPG, JPEG, GIF, WEBP

### 9.2 Abstraction for S3 Migration
```rust
#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> Result<String>;
    async fn download(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    fn get_url(&self, key: &str) -> String;
}
```

---

## 10. Redis Usage

### 10.1 MVP Features

| Feature | Key Pattern | TTL |
|---------|-------------|-----|
| Refresh Tokens | `refresh_token:{token_hash}` | 7 days |
| Presence State | `presence:{user_id}` | 5 minutes |
| Typing Indicators | `typing:{channel_id}:{user_id}` | 10 seconds |
| WebSocket Sessions | `ws_session:{session_id}` | 2 minutes |
| Event Sequence | `seq:{session_id}` | 2 minutes |

### 10.2 Pub/Sub Channels
```
guild:{guild_id}          - Guild-wide events
channel:{channel_id}      - Channel-specific events
user:{user_id}            - User-specific events (DMs, friend requests)
```

### 10.3 Presence State
```json
{
  "status": "online",
  "last_seen": 1705315800,
  "connected_sessions": ["session1", "session2"]
}
```

---

## 11. Project Structure

```
chat_server_v2/
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── README.md
├── SPEC.md
├── schema.sql
│
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library exports
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs         # Configuration management
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs             # Database connection pool
│   │   └── migrations/         # SQL migrations
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── guild.rs
│   │   ├── channel.rs
│   │   ├── message.rs
│   │   ├── role.rs
│   │   ├── member.rs
│   │   ├── reaction.rs
│   │   ├── invite.rs
│   │   └── audit_log.rs
│   │
│   ├── snowflake/
│   │   ├── mod.rs
│   │   └── generator.rs        # Snowflake ID generation
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── router.rs           # Route definitions
│   │   ├── error.rs            # API error handling
│   │   ├── extractors/         # Custom Axum extractors
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   └── pagination.rs
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── auth.rs
│   │       ├── users.rs
│   │       ├── guilds.rs
│   │       ├── channels.rs
│   │       ├── messages.rs
│   │       ├── members.rs
│   │       ├── roles.rs
│   │       ├── reactions.rs
│   │       ├── invites.rs
│   │       └── health.rs
│   │
│   ├── gateway/
│   │   ├── mod.rs
│   │   ├── server.rs           # WebSocket upgrade handler
│   │   ├── session.rs          # Connection session management
│   │   ├── handler.rs          # Message handling
│   │   ├── events.rs           # Event definitions
│   │   ├── opcodes.rs          # Op code definitions
│   │   └── presence.rs         # Presence management
│   │
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── user_service.rs
│   │   ├── guild_service.rs
│   │   ├── channel_service.rs
│   │   ├── message_service.rs
│   │   ├── permission_service.rs
│   │   ├── file_service.rs
│   │   └── audit_service.rs
│   │
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── redis.rs            # Redis connection
│   │   ├── presence.rs         # Presence caching
│   │   └── session.rs          # Session caching
│   │
│   └── utils/
│       ├── mod.rs
│       ├── jwt.rs
│       ├── password.rs
│       └── validation.rs
│
├── tests/
│   ├── common/
│   │   └── mod.rs
│   ├── api/
│   │   ├── auth_test.rs
│   │   ├── guild_test.rs
│   │   └── message_test.rs
│   └── gateway/
│       └── connection_test.rs
│
└── docs/
    └── openapi.yaml            # Generated OpenAPI spec
```

---

## 12. Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Project setup (Cargo.toml, dependencies)
- [ ] Configuration management
- [ ] Database connection pool
- [ ] Snowflake ID generator
- [ ] Basic error handling
- [ ] Health endpoints

### Phase 2: Authentication (Week 2-3)
- [ ] User model and migrations
- [ ] Password hashing
- [ ] JWT generation/validation
- [ ] Register/Login/Logout endpoints
- [ ] Refresh token flow
- [ ] Auth middleware

### Phase 3: Core Entities (Week 3-4)
- [ ] Guild CRUD
- [ ] Channel CRUD
- [ ] Role CRUD
- [ ] Member management
- [ ] Permission checking

### Phase 4: Messaging (Week 4-5)
- [ ] Message CRUD
- [ ] Reactions
- [ ] File uploads (local)
- [ ] Message pagination

### Phase 5: WebSocket Gateway (Week 5-7)
- [ ] WebSocket upgrade handler
- [ ] Session management
- [ ] Heartbeat system
- [ ] Identify flow
- [ ] Event dispatching
- [ ] Resume capability

### Phase 6: Real-time Features (Week 7-8)
- [ ] Redis Pub/Sub integration
- [ ] Message broadcasting
- [ ] Presence system
- [ ] Typing indicators

### Phase 7: Polish (Week 8-9)
- [ ] DM channels
- [ ] Invites
- [ ] Audit logs
- [ ] OpenAPI documentation
- [ ] Integration tests

### Phase 8: Future Enhancements
- [ ] Rate limiting
- [ ] Channel permission overwrites
- [ ] Group DMs
- [ ] Message threads
- [ ] Bot applications
- [ ] OAuth providers
- [ ] S3 file storage
- [ ] Horizontal scaling

---

## Appendix A: Error Codes

| Code | Message |
|------|---------|
| 10001 | Unknown Account |
| 10002 | Unknown Guild |
| 10003 | Unknown Channel |
| 10004 | Unknown Member |
| 10008 | Unknown Message |
| 10011 | Unknown Role |
| 10013 | Unknown Invite |
| 30001 | Maximum guilds reached |
| 30002 | Maximum roles reached |
| 30005 | Maximum channels reached |
| 40001 | Unauthorized |
| 40002 | Invalid account type |
| 50001 | Missing access |
| 50013 | Missing permissions |
| 50035 | Invalid form body |

---

## Appendix B: Gateway Close Codes

| Code | Description | Reconnect? |
|------|-------------|------------|
| 4000 | Unknown error | Yes |
| 4001 | Unknown opcode | Yes |
| 4002 | Decode error | Yes |
| 4003 | Not authenticated | No |
| 4004 | Authentication failed | No |
| 4005 | Already authenticated | Yes |
| 4007 | Invalid sequence | Yes |
| 4008 | Rate limited | Yes |
| 4009 | Session timeout | Yes |
| 4010 | Invalid shard | No |
| 4011 | Sharding required | No |
| 4012 | Invalid API version | No |

---

*This specification serves as the authoritative reference for implementing the Discord-like chat server. All implementation decisions should align with this document.*
