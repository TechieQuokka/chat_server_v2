# Project Structure Documentation

## Discord-like Chat Server - Rust Workspace Architecture

This document outlines the recommended project structure for a high-performance, Discord-like chat server built with Rust. The architecture follows Clean Architecture principles with a multi-crate workspace design for modularity, testability, and compile-time optimization.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Workspace Layout](#workspace-layout)
3. [Crate Descriptions](#crate-descriptions)
4. [Dependency Graph](#dependency-graph)
5. [Directory Structure](#directory-structure)
6. [Technology Stack](#technology-stack)
7. [Configuration Management](#configuration-management)
8. [Build and Development](#build-and-development)

---

## Architecture Overview

```
+------------------------------------------------------------------+
|                         CLIENT LAYER                              |
|  (Web/Mobile/Desktop clients via REST API & WebSocket Gateway)    |
+------------------------------------------------------------------+
                               |
                               v
+------------------------------------------------------------------+
|                       GATEWAY LAYER                               |
|  chat-gateway (WebSocket) | chat-api (REST/HTTP)                 |
+------------------------------------------------------------------+
                               |
                               v
+------------------------------------------------------------------+
|                      APPLICATION LAYER                            |
|  chat-service (Business Logic & Use Cases)                       |
+------------------------------------------------------------------+
                               |
                               v
+------------------------------------------------------------------+
|                        DOMAIN LAYER                               |
|  chat-core (Entities, Value Objects, Domain Events)              |
+------------------------------------------------------------------+
                               |
                               v
+------------------------------------------------------------------+
|                    INFRASTRUCTURE LAYER                           |
|  chat-db (PostgreSQL) | chat-cache (Redis) | chat-queue (Events) |
+------------------------------------------------------------------+
```

### Design Principles

1. **Dependency Inversion**: Higher layers depend on abstractions, not concrete implementations
2. **Single Responsibility**: Each crate has a focused purpose
3. **Interface Segregation**: Traits are specific to use cases
4. **Open/Closed**: Extensible through traits, closed for modification
5. **Hexagonal Architecture**: Ports (traits) and Adapters (implementations)

---

## Workspace Layout

```
chat_server_v2/
├── Cargo.toml                    # Workspace root manifest
├── Cargo.lock                    # Locked dependencies
├── .cargo/
│   └── config.toml               # Cargo configuration (linker, target settings)
├── .env.example                  # Environment variable template
├── .gitignore
├── README.md
├── PROJECT_STRUCTURE.md          # This document
├── ARCHITECTURE.md               # Detailed architecture decisions
├── CHANGELOG.md                  # Version history
├── LICENSE
│
├── config/                       # Configuration files
│   ├── default.toml              # Default configuration
│   ├── development.toml          # Development overrides
│   ├── production.toml           # Production overrides
│   └── test.toml                 # Test environment config
│
├── migrations/                   # Database migrations (sqlx)
│   ├── 20240101000000_init.sql
│   ├── 20240101000001_users.sql
│   ├── 20240101000002_servers.sql
│   ├── 20240101000003_channels.sql
│   ├── 20240101000004_messages.sql
│   └── 20240101000005_reactions.sql
│
├── proto/                        # Protocol Buffer definitions (optional gRPC)
│   ├── chat.proto
│   ├── gateway.proto
│   └── events.proto
│
├── scripts/                      # Development and deployment scripts
│   ├── setup.sh                  # Initial setup script
│   ├── migrate.sh                # Database migration runner
│   ├── seed.sh                   # Database seeding
│   └── docker-build.sh           # Docker image builder
│
├── docker/                       # Docker configurations
│   ├── Dockerfile                # Multi-stage production build
│   ├── Dockerfile.dev            # Development with hot reload
│   └── docker-compose.yml        # Local development stack
│
├── k8s/                          # Kubernetes manifests (optional)
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   └── secrets.yaml
│
├── docs/                         # Additional documentation
│   ├── api/                      # API documentation
│   │   ├── rest-api.md
│   │   └── websocket-api.md
│   ├── deployment.md
│   └── development.md
│
├── tests/                        # Integration and E2E tests
│   ├── common/
│   │   └── mod.rs                # Shared test utilities
│   ├── api_tests.rs              # REST API integration tests
│   ├── gateway_tests.rs          # WebSocket integration tests
│   └── load_tests/               # Performance tests (k6, locust)
│       └── scenarios/
│
└── crates/                       # Workspace member crates
    ├── chat-core/                # Domain layer
    ├── chat-db/                  # Database infrastructure
    ├── chat-cache/               # Redis caching layer
    ├── chat-service/             # Application/business logic
    ├── chat-api/                 # REST API server
    ├── chat-gateway/             # WebSocket gateway
    ├── chat-common/              # Shared utilities
    └── chat-macros/              # Procedural macros (optional)
```

---

## Crate Descriptions

### 1. `chat-core` (Domain Layer)

The heart of the application containing pure domain logic with zero external dependencies.

```
crates/chat-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── entities/                 # Core domain entities
    │   ├── mod.rs
    │   ├── user.rs               # User entity
    │   ├── server.rs             # Server (guild) entity
    │   ├── channel.rs            # Channel entity (text, voice, category)
    │   ├── message.rs            # Message entity
    │   ├── reaction.rs           # Reaction entity
    │   ├── member.rs             # Server membership
    │   ├── role.rs               # Role entity
    │   └── presence.rs           # User presence state
    │
    ├── value_objects/            # Immutable value types
    │   ├── mod.rs
    │   ├── snowflake.rs          # Snowflake ID implementation
    │   ├── email.rs              # Validated email type
    │   ├── username.rs           # Validated username
    │   ├── password.rs           # Password hash wrapper
    │   ├── permissions.rs        # Permission bitfield
    │   └── color.rs              # Role/embed color
    │
    ├── events/                   # Domain events
    │   ├── mod.rs
    │   ├── message_events.rs     # MessageCreated, MessageUpdated, etc.
    │   ├── channel_events.rs     # ChannelCreated, ChannelDeleted, etc.
    │   ├── member_events.rs      # MemberJoined, MemberLeft, etc.
    │   ├── presence_events.rs    # PresenceUpdated, TypingStarted, etc.
    │   └── server_events.rs      # ServerCreated, ServerUpdated, etc.
    │
    ├── errors/                   # Domain-specific errors
    │   ├── mod.rs
    │   └── domain_error.rs
    │
    ├── traits/                   # Repository and service traits (ports)
    │   ├── mod.rs
    │   ├── user_repository.rs
    │   ├── server_repository.rs
    │   ├── channel_repository.rs
    │   ├── message_repository.rs
    │   └── event_publisher.rs
    │
    └── aggregates/               # Aggregate roots
        ├── mod.rs
        ├── server_aggregate.rs   # Server with channels, roles, members
        └── conversation.rs       # DM conversation aggregate
```

**Key Responsibilities:**
- Define all domain entities with validation logic
- Implement Snowflake ID generation (Twitter-style distributed IDs)
- Define repository traits (ports) for dependency inversion
- Contain domain events for event-driven architecture
- Zero dependencies on web frameworks or databases

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
bitflags = "2.4"
validator = { version = "0.18", features = ["derive"] }
```

---

### 2. `chat-common` (Shared Utilities)

Cross-cutting concerns and utilities shared across all crates.

```
crates/chat-common/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── config/                   # Configuration management
    │   ├── mod.rs
    │   ├── app_config.rs         # Application configuration struct
    │   ├── database_config.rs    # Database settings
    │   ├── redis_config.rs       # Redis settings
    │   └── jwt_config.rs         # JWT settings
    │
    ├── error/                    # Unified error handling
    │   ├── mod.rs
    │   ├── app_error.rs          # Application-wide error type
    │   └── error_response.rs     # HTTP error response format
    │
    ├── telemetry/                # Observability
    │   ├── mod.rs
    │   ├── tracing.rs            # Tracing setup
    │   ├── metrics.rs            # Prometheus metrics
    │   └── health.rs             # Health check utilities
    │
    ├── auth/                     # Authentication utilities
    │   ├── mod.rs
    │   ├── jwt.rs                # JWT encoding/decoding
    │   ├── claims.rs             # Token claims structure
    │   └── password.rs           # Password hashing (argon2)
    │
    └── utils/                    # General utilities
        ├── mod.rs
        ├── time.rs               # Time utilities
        ├── pagination.rs         # Pagination helpers
        └── rate_limit.rs         # Rate limiting primitives
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-common"
version = "0.1.0"
edition = "2021"

[dependencies]
chat-core = { path = "../chat-core" }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Security
jsonwebtoken = "9.3"
argon2 = "0.5"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
prometheus = "0.13"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Async
tokio = { version = "1.40", features = ["sync"] }
```

---

### 3. `chat-db` (Database Infrastructure)

PostgreSQL database layer implementing repository traits.

```
crates/chat-db/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── pool.rs                   # Connection pool management
    │
    ├── models/                   # Database models (sqlx FromRow)
    │   ├── mod.rs
    │   ├── user_model.rs
    │   ├── server_model.rs
    │   ├── channel_model.rs
    │   ├── message_model.rs
    │   ├── member_model.rs
    │   ├── role_model.rs
    │   └── reaction_model.rs
    │
    ├── repositories/             # Repository implementations
    │   ├── mod.rs
    │   ├── pg_user_repository.rs
    │   ├── pg_server_repository.rs
    │   ├── pg_channel_repository.rs
    │   ├── pg_message_repository.rs
    │   └── pg_member_repository.rs
    │
    ├── queries/                  # SQL query modules
    │   ├── mod.rs
    │   ├── user_queries.rs
    │   ├── server_queries.rs
    │   ├── channel_queries.rs
    │   └── message_queries.rs
    │
    └── mappers/                  # Model <-> Entity mappers
        ├── mod.rs
        ├── user_mapper.rs
        ├── server_mapper.rs
        └── message_mapper.rs
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-db"
version = "0.1.0"
edition = "2021"

[dependencies]
chat-core = { path = "../chat-core" }
chat-common = { path = "../chat-common" }

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "postgres",
    "uuid",
    "chrono",
    "json",
    "migrate"
] }

# Async
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Tracing
tracing = "0.1"
```

---

### 4. `chat-cache` (Redis Caching Layer)

Redis integration for caching, sessions, presence, and pub/sub.

```
crates/chat-cache/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── pool.rs                   # Redis connection pool
    │
    ├── session/                  # Session management
    │   ├── mod.rs
    │   └── session_store.rs      # User session storage
    │
    ├── presence/                 # Presence tracking
    │   ├── mod.rs
    │   ├── presence_store.rs     # Online/offline status
    │   └── typing_indicator.rs   # Typing status (TTL-based)
    │
    ├── cache/                    # General caching
    │   ├── mod.rs
    │   ├── user_cache.rs         # User data cache
    │   ├── server_cache.rs       # Server data cache
    │   └── permission_cache.rs   # Permission cache
    │
    ├── pubsub/                   # Pub/Sub for real-time events
    │   ├── mod.rs
    │   ├── publisher.rs          # Event publisher
    │   └── subscriber.rs         # Event subscriber
    │
    └── rate_limit/               # Rate limiting
        ├── mod.rs
        └── sliding_window.rs     # Sliding window rate limiter
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-cache"
version = "0.1.0"
edition = "2021"

[dependencies]
chat-core = { path = "../chat-core" }
chat-common = { path = "../chat-common" }

# Redis
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.18"

# Async
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Tracing
tracing = "0.1"
```

---

### 5. `chat-service` (Application Layer)

Business logic and use case implementations.

```
crates/chat-service/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── services/                 # Service implementations
    │   ├── mod.rs
    │   ├── auth_service.rs       # Authentication & registration
    │   ├── user_service.rs       # User management
    │   ├── server_service.rs     # Server (guild) management
    │   ├── channel_service.rs    # Channel CRUD operations
    │   ├── message_service.rs    # Message handling
    │   ├── member_service.rs     # Membership management
    │   ├── role_service.rs       # Role & permission management
    │   ├── reaction_service.rs   # Message reactions
    │   ├── presence_service.rs   # Online presence tracking
    │   └── dm_service.rs         # Direct messages
    │
    ├── commands/                 # Command handlers (CQRS write side)
    │   ├── mod.rs
    │   ├── create_message.rs
    │   ├── update_message.rs
    │   ├── delete_message.rs
    │   ├── create_server.rs
    │   ├── join_server.rs
    │   └── update_presence.rs
    │
    ├── queries/                  # Query handlers (CQRS read side)
    │   ├── mod.rs
    │   ├── get_messages.rs
    │   ├── get_server.rs
    │   ├── get_channels.rs
    │   └── get_members.rs
    │
    ├── dto/                      # Data Transfer Objects
    │   ├── mod.rs
    │   ├── user_dto.rs
    │   ├── server_dto.rs
    │   ├── channel_dto.rs
    │   ├── message_dto.rs
    │   └── presence_dto.rs
    │
    └── events/                   # Event handlers
        ├── mod.rs
        ├── message_event_handler.rs
        └── presence_event_handler.rs
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-service"
version = "0.1.0"
edition = "2021"

[dependencies]
chat-core = { path = "../chat-core" }
chat-common = { path = "../chat-common" }
chat-db = { path = "../chat-db" }
chat-cache = { path = "../chat-cache" }

# Async
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Tracing
tracing = "0.1"

# Error handling
thiserror = "2.0"
```

---

### 6. `chat-api` (REST API Server)

HTTP REST API using Axum framework.

```
crates/chat-api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs                   # Binary entry point
    │
    ├── server.rs                 # Axum server setup
    │
    ├── routes/                   # Route definitions
    │   ├── mod.rs
    │   ├── auth_routes.rs        # /auth/*
    │   ├── user_routes.rs        # /users/*
    │   ├── server_routes.rs      # /servers/*
    │   ├── channel_routes.rs     # /channels/*
    │   ├── message_routes.rs     # /channels/{id}/messages/*
    │   └── health_routes.rs      # /health, /ready
    │
    ├── handlers/                 # Request handlers
    │   ├── mod.rs
    │   ├── auth_handler.rs
    │   ├── user_handler.rs
    │   ├── server_handler.rs
    │   ├── channel_handler.rs
    │   └── message_handler.rs
    │
    ├── extractors/               # Custom Axum extractors
    │   ├── mod.rs
    │   ├── auth.rs               # JWT authentication extractor
    │   ├── pagination.rs         # Pagination query params
    │   └── validated_json.rs     # JSON with validation
    │
    ├── middleware/               # Axum middleware
    │   ├── mod.rs
    │   ├── auth_middleware.rs    # JWT verification
    │   ├── rate_limit.rs         # Rate limiting
    │   ├── request_id.rs         # Request ID injection
    │   └── logging.rs            # Request/response logging
    │
    ├── response/                 # Response types
    │   ├── mod.rs
    │   ├── api_response.rs       # Unified API response
    │   └── error_response.rs     # Error response format
    │
    └── state.rs                  # Application state (shared services)
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-api"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "chat-api"
path = "src/main.rs"

[dependencies]
chat-core = { path = "../chat-core" }
chat-common = { path = "../chat-common" }
chat-service = { path = "../chat-service" }
chat-db = { path = "../chat-db" }
chat-cache = { path = "../chat-cache" }

# Web framework
axum = { version = "0.7", features = ["macros"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = { version = "0.5", features = ["timeout", "limit"] }
tower-http = { version = "0.6", features = [
    "cors",
    "trace",
    "request-id",
    "compression-gzip",
    "timeout"
] }

# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Tracing
tracing = "0.1"

# UUID
uuid = { version = "1.10", features = ["v4", "serde"] }
```

---

### 7. `chat-gateway` (WebSocket Gateway)

Real-time WebSocket server for live events.

```
crates/chat-gateway/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs                   # Binary entry point
    │
    ├── server.rs                 # WebSocket server setup
    │
    ├── connection/               # Connection management
    │   ├── mod.rs
    │   ├── manager.rs            # Connection manager (all active connections)
    │   ├── session.rs            # Individual session state
    │   └── heartbeat.rs          # Heartbeat/ping-pong handling
    │
    ├── handlers/                 # Message handlers
    │   ├── mod.rs
    │   ├── identify.rs           # Client identification/auth
    │   ├── resume.rs             # Session resume after disconnect
    │   ├── heartbeat.rs          # Heartbeat acknowledgment
    │   ├── subscribe.rs          # Channel subscription
    │   └── typing.rs             # Typing indicator
    │
    ├── events/                   # Event dispatching
    │   ├── mod.rs
    │   ├── dispatcher.rs         # Event dispatcher to clients
    │   ├── gateway_event.rs      # Gateway event types
    │   └── payload.rs            # Event payload structures
    │
    ├── protocol/                 # Gateway protocol
    │   ├── mod.rs
    │   ├── opcodes.rs            # Operation codes
    │   ├── close_codes.rs        # WebSocket close codes
    │   └── message.rs            # Protocol message format
    │
    └── broadcast/                # Broadcasting
        ├── mod.rs
        ├── channel_broadcast.rs  # Broadcast to channel subscribers
        ├── server_broadcast.rs   # Broadcast to server members
        └── user_broadcast.rs     # Broadcast to specific user
```

**Cargo.toml Dependencies:**
```toml
[package]
name = "chat-gateway"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "chat-gateway"
path = "src/main.rs"

[dependencies]
chat-core = { path = "../chat-core" }
chat-common = { path = "../chat-common" }
chat-service = { path = "../chat-service" }
chat-cache = { path = "../chat-cache" }

# Web framework with WebSocket
axum = { version = "0.7", features = ["ws"] }
tokio-tungstenite = "0.24"

# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Concurrency
dashmap = "6.1"
parking_lot = "0.12"

# Tracing
tracing = "0.1"

# Futures
futures = "0.3"
futures-util = "0.3"
```

---

### 8. `chat-macros` (Procedural Macros - Optional)

Custom derive macros for reducing boilerplate.

```
crates/chat-macros/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── snowflake.rs              # #[derive(Snowflake)] for ID types
    ├── event.rs                  # #[derive(DomainEvent)] for events
    └── dto.rs                    # #[derive(IntoDto)] for entity -> DTO
```

**Cargo.toml:**
```toml
[package]
name = "chat-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

---

## Dependency Graph

```
                                 chat-macros (optional)
                                        |
                                        v
+------------------+            +------------------+
|   chat-common    |<-----------|   chat-core      |
+------------------+            +------------------+
         ^                              ^
         |                              |
         |    +-------------------------+-------------------------+
         |    |                         |                         |
         v    v                         v                         v
+------------------+            +------------------+      +------------------+
|    chat-db       |            |   chat-cache     |      |   (future crates)|
+------------------+            +------------------+      +------------------+
         ^                              ^
         |                              |
         +---------------+--------------+
                         |
                         v
                +------------------+
                |  chat-service    |
                +------------------+
                         ^
                         |
         +---------------+---------------+
         |                               |
         v                               v
+------------------+            +------------------+
|    chat-api      |            |  chat-gateway    |
+------------------+            +------------------+
```

**Dependency Rules:**
1. `chat-core` has NO internal workspace dependencies (pure domain)
2. `chat-common` depends only on `chat-core`
3. Infrastructure crates (`chat-db`, `chat-cache`) depend on `chat-core` and `chat-common`
4. `chat-service` depends on all infrastructure crates
5. Presentation crates (`chat-api`, `chat-gateway`) depend on `chat-service`

---

## Directory Structure

### Complete File Tree

```
chat_server_v2/
├── Cargo.toml
├── Cargo.lock
├── .cargo/
│   └── config.toml
├── .env.example
├── .gitignore
├── README.md
├── PROJECT_STRUCTURE.md
├── ARCHITECTURE.md
├── CHANGELOG.md
├── LICENSE
├── rustfmt.toml
├── clippy.toml
├── deny.toml                     # cargo-deny configuration
│
├── config/
│   ├── default.toml
│   ├── development.toml
│   ├── production.toml
│   └── test.toml
│
├── migrations/
│   ├── 20240101000000_init.sql
│   ├── 20240101000001_users.sql
│   ├── 20240101000002_servers.sql
│   ├── 20240101000003_channels.sql
│   ├── 20240101000004_messages.sql
│   ├── 20240101000005_reactions.sql
│   ├── 20240101000006_members.sql
│   ├── 20240101000007_roles.sql
│   └── 20240101000008_dm_channels.sql
│
├── proto/
│   ├── chat.proto
│   ├── gateway.proto
│   └── events.proto
│
├── scripts/
│   ├── setup.sh
│   ├── setup.ps1                 # Windows PowerShell
│   ├── migrate.sh
│   ├── seed.sh
│   └── docker-build.sh
│
├── docker/
│   ├── Dockerfile
│   ├── Dockerfile.dev
│   └── docker-compose.yml
│
├── k8s/
│   ├── namespace.yaml
│   ├── deployment-api.yaml
│   ├── deployment-gateway.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   └── hpa.yaml
│
├── docs/
│   ├── api/
│   │   ├── rest-api.md
│   │   ├── websocket-api.md
│   │   └── openapi.yaml
│   ├── architecture/
│   │   ├── decisions/            # Architecture Decision Records
│   │   │   ├── 001-workspace-structure.md
│   │   │   ├── 002-snowflake-ids.md
│   │   │   └── 003-event-driven.md
│   │   └── diagrams/
│   ├── deployment.md
│   └── development.md
│
├── tests/
│   ├── common/
│   │   ├── mod.rs
│   │   ├── fixtures.rs
│   │   └── test_db.rs
│   ├── api_tests.rs
│   ├── gateway_tests.rs
│   └── load_tests/
│       ├── k6/
│       │   └── scenarios/
│       └── results/
│
├── benches/                      # Benchmarks
│   ├── snowflake_bench.rs
│   └── message_throughput.rs
│
└── crates/
    ├── chat-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── entities/
    │       ├── value_objects/
    │       ├── events/
    │       ├── errors/
    │       ├── traits/
    │       └── aggregates/
    │
    ├── chat-common/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── config/
    │       ├── error/
    │       ├── telemetry/
    │       ├── auth/
    │       └── utils/
    │
    ├── chat-db/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── pool.rs
    │       ├── models/
    │       ├── repositories/
    │       ├── queries/
    │       └── mappers/
    │
    ├── chat-cache/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── pool.rs
    │       ├── session/
    │       ├── presence/
    │       ├── cache/
    │       ├── pubsub/
    │       └── rate_limit/
    │
    ├── chat-service/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── services/
    │       ├── commands/
    │       ├── queries/
    │       ├── dto/
    │       └── events/
    │
    ├── chat-api/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── main.rs
    │       ├── server.rs
    │       ├── routes/
    │       ├── handlers/
    │       ├── extractors/
    │       ├── middleware/
    │       ├── response/
    │       └── state.rs
    │
    ├── chat-gateway/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── main.rs
    │       ├── server.rs
    │       ├── connection/
    │       ├── handlers/
    │       ├── events/
    │       ├── protocol/
    │       └── broadcast/
    │
    └── chat-macros/              # Optional
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── snowflake.rs
            ├── event.rs
            └── dto.rs
```

---

## Technology Stack

### Core Technologies

| Category | Technology | Version | Purpose |
|----------|------------|---------|---------|
| Language | Rust | 1.82+ | Primary language |
| Runtime | Tokio | 1.40+ | Async runtime |
| Web Framework | Axum | 0.7+ | HTTP/WebSocket server |
| Database | PostgreSQL | 16+ | Primary data store |
| Cache | Redis | 7+ | Caching, sessions, pub/sub |
| ORM/Query | SQLx | 0.8+ | Async SQL toolkit |

### Supporting Libraries

| Category | Crate | Purpose |
|----------|-------|---------|
| Serialization | serde, serde_json | JSON serialization |
| Validation | validator | Input validation |
| Error Handling | thiserror, anyhow | Error types |
| Authentication | jsonwebtoken, argon2 | JWT & password hashing |
| Observability | tracing, prometheus | Logging & metrics |
| Testing | tokio-test, testcontainers | Async testing |
| HTTP Client | reqwest | External API calls |
| WebSocket | tokio-tungstenite | WebSocket protocol |
| Concurrency | dashmap, parking_lot | Concurrent data structures |

### Development Tools

| Tool | Purpose |
|------|---------|
| cargo-watch | Hot reload during development |
| cargo-deny | Dependency auditing |
| cargo-nextest | Faster test runner |
| sqlx-cli | Database migrations |
| docker-compose | Local development environment |

---

## Configuration Management

### Root Cargo.toml (Workspace)

```toml
[workspace]
resolver = "2"
members = [
    "crates/chat-core",
    "crates/chat-common",
    "crates/chat-db",
    "crates/chat-cache",
    "crates/chat-service",
    "crates/chat-api",
    "crates/chat-gateway",
    # "crates/chat-macros",  # Uncomment when needed
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"
repository = "https://github.com/yourusername/chat_server_v2"

[workspace.dependencies]
# Internal crates
chat-core = { path = "crates/chat-core" }
chat-common = { path = "crates/chat-common" }
chat-db = { path = "crates/chat-db" }
chat-cache = { path = "crates/chat-cache" }
chat-service = { path = "crates/chat-service" }

# Async runtime
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Web framework
axum = { version = "0.7", features = ["macros", "ws"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = { version = "0.5", features = ["timeout", "limit"] }
tower-http = { version = "0.6", features = ["cors", "trace", "request-id", "compression-gzip", "timeout"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "migrate"] }

# Redis
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
deadpool-redis = "0.18"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Security
jsonwebtoken = "9.3"
argon2 = "0.5"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
prometheus = "0.13"

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.10", features = ["v4", "serde"] }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Concurrency
dashmap = "6.1"
parking_lot = "0.12"

# Utilities
bitflags = "2.4"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
panic = "abort"

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

### Environment Configuration (.env.example)

```env
# Application
APP_NAME=chat_server
APP_ENV=development
APP_HOST=0.0.0.0
API_PORT=8080
GATEWAY_PORT=8081
RUST_LOG=info,chat_api=debug,chat_gateway=debug,sqlx=warn

# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/chat_db
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# Redis
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=10

# JWT
JWT_SECRET=your-super-secret-key-change-in-production
JWT_EXPIRATION_HOURS=24
JWT_REFRESH_EXPIRATION_DAYS=7

# Snowflake ID
SNOWFLAKE_MACHINE_ID=1
SNOWFLAKE_DATACENTER_ID=1

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_SECOND=10
RATE_LIMIT_BURST=50

# Metrics
METRICS_ENABLED=true
METRICS_PORT=9090
```

---

## Build and Development

### Development Commands

```bash
# Initial setup
cargo build --workspace

# Run API server
cargo run -p chat-api

# Run Gateway server
cargo run -p chat-gateway

# Run both (use separate terminals or tmux)
cargo run -p chat-api &
cargo run -p chat-gateway &

# Run tests
cargo test --workspace

# Run tests with nextest (faster)
cargo nextest run --workspace

# Run specific crate tests
cargo test -p chat-core
cargo test -p chat-service

# Check all targets
cargo check --workspace --all-targets

# Clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Format code
cargo fmt --all

# Database migrations
sqlx migrate run

# Watch mode (development)
cargo watch -x 'run -p chat-api'
```

### Docker Commands

```bash
# Build production image
docker build -f docker/Dockerfile -t chat-server:latest .

# Run local development stack
docker-compose -f docker/docker-compose.yml up -d

# View logs
docker-compose -f docker/docker-compose.yml logs -f

# Stop stack
docker-compose -f docker/docker-compose.yml down
```

### Database Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

---

## References

### Repositories Analyzed

1. **TechieQuokka/chat-alpha** - Rust/Axum/WebSocket/PostgreSQL chat with JWT auth
2. **kumarUjjawal/whisper** - Real-time chat backend with Axum and PostgreSQL
3. **sushant-at-nitor/warp-microservice** - Clean Architecture Rust monorepo
4. **Spooled-Cloud/spooled-backend** - Production webhook queue with Axum/PostgreSQL/Redis
5. **serenity-rs/poise** - Discord bot framework with workspace structure

### Best Practices Applied

- **Workspace organization** from production Rust projects
- **Clean Architecture** separation of concerns
- **Repository pattern** for database abstraction
- **Event-driven design** for real-time features
- **Comprehensive observability** with tracing and metrics

---

## Next Steps

1. **Phase 1**: Implement `chat-core` with entities and Snowflake IDs
2. **Phase 2**: Set up `chat-common` with configuration and auth utilities
3. **Phase 3**: Implement `chat-db` with PostgreSQL repositories
4. **Phase 4**: Implement `chat-cache` with Redis integration
5. **Phase 5**: Build `chat-service` with business logic
6. **Phase 6**: Create `chat-api` REST endpoints
7. **Phase 7**: Develop `chat-gateway` WebSocket server
8. **Phase 8**: Integration testing and load testing
9. **Phase 9**: Docker and Kubernetes deployment

---

*Document Version: 1.0.0*
*Last Updated: 2024*
