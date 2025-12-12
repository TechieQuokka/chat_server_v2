# Discord-like Chat Server

A real-time chat server MVP built with Rust, featuring a REST API and WebSocket gateway for Discord-like functionality.

## Features

- **User Management**: Registration, authentication (JWT), profile management
- **Guilds (Servers)**: Create, manage, and join guilds
- **Channels**: Text channels with categories
- **Messaging**: Real-time messages with replies
- **Roles & Permissions**: Hierarchical role system with granular permissions
- **Invites**: Shareable invite links with expiration
- **Presence**: Online status and typing indicators
- **Real-time Events**: WebSocket gateway for live updates

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
│                    (Web, Mobile, Desktop)                        │
└───────────────────┬─────────────────────┬───────────────────────┘
                    │                     │
                    ▼                     ▼
┌───────────────────────────┐ ┌───────────────────────────────────┐
│      REST API (8080)      │ │   WebSocket Gateway (8081)        │
│        chat-api           │ │        chat-gateway               │
│                           │ │                                   │
│  • Auth endpoints         │ │  • Real-time events               │
│  • CRUD operations        │ │  • Presence updates               │
│  • File uploads           │ │  • Typing indicators              │
└───────────┬───────────────┘ └───────────────┬───────────────────┘
            │                                 │
            └────────────┬────────────────────┘
                         │
                         ▼
┌────────────────────────────────────────────────────────────────┐
│                     Service Layer                               │
│                     chat-service                                │
│  • AuthService      • GuildService     • ChannelService        │
│  • MessageService   • MemberService    • RoleService           │
│  • InviteService    • PresenceService  • PermissionService     │
└───────────────────────┬────────────────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
┌─────────────────┐ ┌────────────┐ ┌─────────────────┐
│   chat-db       │ │ chat-cache │ │   chat-core     │
│ PostgreSQL      │ │   Redis    │ │ Domain Models   │
│ Repositories    │ │ Sessions   │ │ Value Objects   │
│                 │ │ Presence   │ │ Repository      │
│                 │ │ Pub/Sub    │ │ Traits          │
└─────────────────┘ └────────────┘ └─────────────────┘
```

## Tech Stack

- **Language**: Rust 1.82+
- **Web Framework**: Axum
- **Database**: PostgreSQL 16
- **Cache**: Redis 7
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: Argon2
- **Async Runtime**: Tokio

## Project Structure

```
chat_server_v2/
├── crates/
│   ├── chat-core/       # Domain layer (entities, value objects, traits)
│   ├── chat-common/     # Shared utilities (config, errors, JWT, password)
│   ├── chat-db/         # Database layer (PostgreSQL repositories)
│   ├── chat-cache/      # Cache layer (Redis sessions, presence, pub/sub)
│   ├── chat-service/    # Business logic layer
│   ├── chat-api/        # REST API server
│   └── chat-gateway/    # WebSocket gateway server
├── tests/
│   └── integration/     # Integration tests
├── docs/
│   ├── schema.sql       # Database schema
│   ├── api.yaml         # OpenAPI specification
│   └── ...              # Architecture documentation
├── Dockerfile           # Multi-stage Docker build
├── docker-compose.yml   # Full stack deployment
└── .env.example         # Environment configuration template
```

## Quick Start

### Prerequisites

- Rust 1.82+
- PostgreSQL 16+
- Redis 7+
- Docker & Docker Compose (optional)

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd chat_server_v2

# Start all services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f api gateway
```

The services will be available at:
- REST API: http://localhost:8080
- WebSocket Gateway: ws://localhost:8081/gateway
- Health Check: http://localhost:8080/health

### Manual Setup

1. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Set up PostgreSQL**:
   ```bash
   # Create database
   createdb chat_db

   # Run schema
   psql -d chat_db -f docs/schema.sql
   ```

3. **Start Redis**:
   ```bash
   redis-server
   ```

4. **Build and run**:
   ```bash
   # Build all crates
   cargo build --release

   # Run API server
   cargo run --release --bin chat-api

   # Run Gateway server (in another terminal)
   cargo run --release --bin chat-gateway
   ```

## API Documentation

### Authentication

```bash
# Register a new user
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "Password123!"}'

# Login
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "Password123!"}'

# Response includes access_token and refresh_token
```

### Guilds

```bash
# Create a guild (requires auth)
curl -X POST http://localhost:8080/guilds \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "My Guild", "description": "A test guild"}'

# Get guild
curl http://localhost:8080/guilds/<guild_id> \
  -H "Authorization: Bearer <access_token>"
```

### Channels

```bash
# Create a channel
curl -X POST http://localhost:8080/guilds/<guild_id>/channels \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "general", "type": 0}'

# Get channel messages
curl http://localhost:8080/channels/<channel_id>/messages \
  -H "Authorization: Bearer <access_token>"
```

### WebSocket Gateway

Connect to `ws://localhost:8081/gateway` and follow the protocol:

1. **Receive HELLO** (op: 10):
   ```json
   {"op": 10, "d": {"heartbeat_interval": 45000}}
   ```

2. **Send IDENTIFY** (op: 2):
   ```json
   {"op": 2, "d": {"token": "<access_token>"}}
   ```

3. **Receive READY** (op: 0):
   ```json
   {"op": 0, "t": "READY", "d": {"user": {...}, "guilds": [...]}}
   ```

4. **Send HEARTBEAT** (op: 1) periodically:
   ```json
   {"op": 1, "d": null}
   ```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `APP_ENV` | Environment (development/staging/production) | development |
| `API_HOST` | API server host | 127.0.0.1 |
| `API_PORT` | API server port | 8080 |
| `GATEWAY_HOST` | Gateway server host | 127.0.0.1 |
| `GATEWAY_PORT` | Gateway server port | 8081 |
| `DATABASE_URL` | PostgreSQL connection URL | - |
| `DATABASE_MAX_CONNECTIONS` | Max DB pool connections | 20 |
| `REDIS_URL` | Redis connection URL | redis://localhost:6379 |
| `JWT_SECRET` | JWT signing secret (min 32 chars) | - |
| `JWT_ACCESS_TOKEN_EXPIRY` | Access token TTL (seconds) | 900 |
| `JWT_REFRESH_TOKEN_EXPIRY` | Refresh token TTL (seconds) | 604800 |
| `WORKER_ID` | Snowflake ID worker ID (0-31) | 0 |

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run integration tests (requires running PostgreSQL and Redis)
cargo test -p integration-tests

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace -- -D warnings

# Check for security vulnerabilities
cargo audit
```

### Building for Production

```bash
# Build optimized release binaries
cargo build --release

# Binaries will be in target/release/
# - chat-api
# - chat-gateway
```

## Database Schema

The database consists of 14 tables:

- `users` - User accounts
- `guilds` - Guild/server entities
- `channels` - Text channels and categories
- `messages` - Chat messages
- `roles` - Permission roles
- `guild_members` - Guild membership
- `member_roles` - Role assignments
- `reactions` - Message reactions
- `invites` - Guild invite links
- `bans` - User bans
- `attachments` - File attachments
- `dm_recipients` - DM channel participants
- `audit_logs` - Audit trail
- `refresh_tokens` - Session tokens

See `docs/schema.sql` for the complete schema with indexes and triggers.

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request
