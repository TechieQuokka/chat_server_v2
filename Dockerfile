# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.82-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty project
WORKDIR /app

# Copy workspace manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/chat-core/Cargo.toml crates/chat-core/
COPY crates/chat-common/Cargo.toml crates/chat-common/
COPY crates/chat-db/Cargo.toml crates/chat-db/
COPY crates/chat-cache/Cargo.toml crates/chat-cache/
COPY crates/chat-service/Cargo.toml crates/chat-service/
COPY crates/chat-api/Cargo.toml crates/chat-api/
COPY crates/chat-gateway/Cargo.toml crates/chat-gateway/
COPY tests/integration/Cargo.toml tests/integration/

# Create dummy source files for dependency caching
RUN mkdir -p crates/chat-core/src && echo "pub fn dummy() {}" > crates/chat-core/src/lib.rs
RUN mkdir -p crates/chat-common/src && echo "pub fn dummy() {}" > crates/chat-common/src/lib.rs
RUN mkdir -p crates/chat-db/src && echo "pub fn dummy() {}" > crates/chat-db/src/lib.rs
RUN mkdir -p crates/chat-cache/src && echo "pub fn dummy() {}" > crates/chat-cache/src/lib.rs
RUN mkdir -p crates/chat-service/src && echo "pub fn dummy() {}" > crates/chat-service/src/lib.rs
RUN mkdir -p crates/chat-api/src && echo "fn main() {}" > crates/chat-api/src/main.rs && echo "pub fn dummy() {}" > crates/chat-api/src/lib.rs
RUN mkdir -p crates/chat-gateway/src && echo "fn main() {}" > crates/chat-gateway/src/main.rs && echo "pub fn dummy() {}" > crates/chat-gateway/src/lib.rs
RUN mkdir -p tests/integration/src && echo "pub fn dummy() {}" > tests/integration/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release --workspace 2>/dev/null || true

# Remove dummy source files
RUN rm -rf crates tests

# Copy the actual source code
COPY crates crates
COPY tests tests

# Touch source files to invalidate cache
RUN find crates tests -name "*.rs" -exec touch {} +

# Build the actual applications
RUN cargo build --release --bin chat-api --bin chat-gateway

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -u 1000 appuser

# Create app directory
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/chat-api /app/chat-api
COPY --from=builder /app/target/release/chat-gateway /app/chat-gateway

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose ports
EXPOSE 8080 8081

# Default command (can be overridden)
CMD ["./chat-api"]

# =============================================================================
# Stage 3: API server image
# =============================================================================
FROM runtime AS api
CMD ["./chat-api"]

# =============================================================================
# Stage 4: Gateway server image
# =============================================================================
FROM runtime AS gateway
CMD ["./chat-gateway"]
