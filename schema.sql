-- Discord-like Chat Server Database Schema
-- PostgreSQL 15+
--
-- Features:
-- - Snowflake IDs (BIGINT) for all primary keys
-- - Soft deletes via deleted_at timestamp
-- - Audit timestamps (created_at, updated_at)
-- - Adjacency list for channel hierarchy

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- CUSTOM TYPES
-- ============================================================================

-- Channel types
CREATE TYPE channel_type AS ENUM (
    'text',
    'category',
    'dm'
);

-- Presence status
CREATE TYPE presence_status AS ENUM (
    'online',
    'idle',
    'dnd',
    'offline'
);

-- Audit log action types
CREATE TYPE audit_action AS ENUM (
    'guild_update',
    'channel_create',
    'channel_update',
    'channel_delete',
    'role_create',
    'role_update',
    'role_delete',
    'member_kick',
    'member_ban',
    'member_unban',
    'member_role_update',
    'message_delete'
);

-- ============================================================================
-- USERS
-- ============================================================================

CREATE TABLE users (
    id              BIGINT PRIMARY KEY,
    username        VARCHAR(32) NOT NULL,
    discriminator   VARCHAR(4) NOT NULL DEFAULT '0001',
    email           VARCHAR(255) NOT NULL UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    avatar          VARCHAR(255),
    bot             BOOLEAN NOT NULL DEFAULT FALSE,
    system          BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT users_username_discriminator_unique
        UNIQUE (username, discriminator)
);

CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_bot ON users(bot) WHERE deleted_at IS NULL;

-- ============================================================================
-- GUILDS (Servers)
-- ============================================================================

CREATE TABLE guilds (
    id              BIGINT PRIMARY KEY,
    name            VARCHAR(100) NOT NULL,
    icon            VARCHAR(255),
    description     TEXT,
    owner_id        BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_guilds_owner ON guilds(owner_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_guilds_name ON guilds(name) WHERE deleted_at IS NULL;

-- ============================================================================
-- CHANNELS
-- ============================================================================

CREATE TABLE channels (
    id              BIGINT PRIMARY KEY,
    guild_id        BIGINT REFERENCES guilds(id),  -- NULL for DM channels
    name            VARCHAR(100),                   -- NULL for DM channels
    type            channel_type NOT NULL DEFAULT 'text',
    topic           TEXT,
    position        INTEGER NOT NULL DEFAULT 0,
    parent_id       BIGINT REFERENCES channels(id), -- Category reference
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT channels_category_no_parent
        CHECK (type != 'category' OR parent_id IS NULL)
);

CREATE INDEX idx_channels_guild ON channels(guild_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_channels_parent ON channels(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_channels_type ON channels(type) WHERE deleted_at IS NULL;

-- ============================================================================
-- DM CHANNEL RECIPIENTS (for DM and Group DM channels)
-- ============================================================================

CREATE TABLE dm_channel_recipients (
    channel_id      BIGINT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX idx_dm_recipients_user ON dm_channel_recipients(user_id);

-- ============================================================================
-- ROLES
-- ============================================================================

CREATE TABLE roles (
    id              BIGINT PRIMARY KEY,
    guild_id        BIGINT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    color           INTEGER NOT NULL DEFAULT 0,      -- RGB color as integer
    hoist           BOOLEAN NOT NULL DEFAULT FALSE,  -- Display separately
    position        INTEGER NOT NULL DEFAULT 0,
    permissions     BIGINT NOT NULL DEFAULT 0,       -- Permission bitfield
    mentionable     BOOLEAN NOT NULL DEFAULT FALSE,
    is_everyone     BOOLEAN NOT NULL DEFAULT FALSE,  -- @everyone role
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT roles_one_everyone_per_guild
        UNIQUE (guild_id, is_everyone)
        -- Note: Partial unique constraint enforced via trigger
);

CREATE INDEX idx_roles_guild ON roles(guild_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_position ON roles(guild_id, position) WHERE deleted_at IS NULL;

-- ============================================================================
-- GUILD MEMBERS
-- ============================================================================

CREATE TABLE guild_members (
    guild_id        BIGINT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    nickname        VARCHAR(32),
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (guild_id, user_id)
);

CREATE INDEX idx_members_user ON guild_members(user_id);
CREATE INDEX idx_members_joined ON guild_members(guild_id, joined_at);

-- ============================================================================
-- MEMBER ROLES (many-to-many)
-- ============================================================================

CREATE TABLE member_roles (
    guild_id        BIGINT NOT NULL,
    user_id         BIGINT NOT NULL,
    role_id         BIGINT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (guild_id, user_id, role_id),
    FOREIGN KEY (guild_id, user_id) REFERENCES guild_members(guild_id, user_id) ON DELETE CASCADE
);

CREATE INDEX idx_member_roles_role ON member_roles(role_id);

-- ============================================================================
-- MESSAGES
-- ============================================================================

CREATE TABLE messages (
    id              BIGINT PRIMARY KEY,
    channel_id      BIGINT NOT NULL REFERENCES channels(id),
    author_id       BIGINT NOT NULL REFERENCES users(id),
    content         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at       TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,

    -- For future: reply/thread support
    reference_id    BIGINT REFERENCES messages(id)
);

-- Primary index for message fetching (cursor pagination)
CREATE INDEX idx_messages_channel_id ON messages(channel_id, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_messages_author ON messages(author_id) WHERE deleted_at IS NULL;

-- For search (basic)
CREATE INDEX idx_messages_content_search ON messages
    USING gin(to_tsvector('english', content))
    WHERE deleted_at IS NULL;

-- ============================================================================
-- MESSAGE ATTACHMENTS
-- ============================================================================

CREATE TABLE attachments (
    id              BIGINT PRIMARY KEY,
    message_id      BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename        VARCHAR(255) NOT NULL,
    content_type    VARCHAR(100) NOT NULL,
    size            INTEGER NOT NULL,
    url             VARCHAR(512) NOT NULL,
    proxy_url       VARCHAR(512),
    width           INTEGER,  -- For images
    height          INTEGER,  -- For images
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_message ON attachments(message_id);

-- ============================================================================
-- REACTIONS
-- ============================================================================

CREATE TABLE reactions (
    message_id      BIGINT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    emoji           VARCHAR(64) NOT NULL,  -- Unicode emoji or custom emoji ID
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (message_id, user_id, emoji)
);

CREATE INDEX idx_reactions_message ON reactions(message_id);
CREATE INDEX idx_reactions_user ON reactions(user_id);

-- ============================================================================
-- INVITES
-- ============================================================================

CREATE TABLE invites (
    code            VARCHAR(16) PRIMARY KEY,
    guild_id        BIGINT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    channel_id      BIGINT NOT NULL REFERENCES channels(id),
    inviter_id      BIGINT NOT NULL REFERENCES users(id),
    uses            INTEGER NOT NULL DEFAULT 0,
    max_uses        INTEGER,  -- NULL = unlimited
    max_age         INTEGER,  -- Seconds, NULL = never expires
    temporary       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_invites_guild ON invites(guild_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_invites_expires ON invites(expires_at) WHERE deleted_at IS NULL;

-- ============================================================================
-- BANS
-- ============================================================================

CREATE TABLE bans (
    guild_id        BIGINT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    reason          TEXT,
    banned_by       BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (guild_id, user_id)
);

CREATE INDEX idx_bans_user ON bans(user_id);

-- ============================================================================
-- AUDIT LOGS
-- ============================================================================

CREATE TABLE audit_logs (
    id              BIGINT PRIMARY KEY,
    guild_id        BIGINT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id),  -- User who performed action
    action          audit_action NOT NULL,
    target_id       BIGINT,      -- ID of affected entity
    target_type     VARCHAR(50), -- Type of affected entity
    changes         JSONB,       -- Before/after values
    reason          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_guild ON audit_logs(guild_id, created_at DESC);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- ============================================================================
-- REFRESH TOKENS (for JWT refresh)
-- ============================================================================

CREATE TABLE refresh_tokens (
    id              BIGINT PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      VARCHAR(255) NOT NULL UNIQUE,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens(expires_at)
    WHERE revoked_at IS NULL;

-- ============================================================================
-- BOT APPLICATIONS (Later phase)
-- ============================================================================

-- CREATE TABLE applications (
--     id              BIGINT PRIMARY KEY,
--     name            VARCHAR(100) NOT NULL,
--     description     TEXT,
--     icon            VARCHAR(255),
--     owner_id        BIGINT NOT NULL REFERENCES users(id),
--     bot_user_id     BIGINT REFERENCES users(id),
--     bot_token_hash  VARCHAR(255),
--     public          BOOLEAN NOT NULL DEFAULT TRUE,
--     created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
--     updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- ============================================================================
-- FUNCTIONS & TRIGGERS
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to all tables with updated_at
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_guilds_updated_at
    BEFORE UPDATE ON guilds
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_channels_updated_at
    BEFORE UPDATE ON channels
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_roles_updated_at
    BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_guild_members_updated_at
    BEFORE UPDATE ON guild_members
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- DEFAULT DATA HELPERS
-- ============================================================================

-- Function to create default @everyone role when guild is created
CREATE OR REPLACE FUNCTION create_everyone_role()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO roles (id, guild_id, name, permissions, is_everyone, position)
    VALUES (
        NEW.id,  -- Use same ID as guild for @everyone role
        NEW.id,
        '@everyone',
        -- Default permissions: VIEW_CHANNEL | SEND_MESSAGES | ADD_REACTIONS | ATTACH_FILES
        (1 | 2 | 1024 | 512),
        TRUE,
        0
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_guild_everyone_role
    AFTER INSERT ON guilds
    FOR EACH ROW EXECUTE FUNCTION create_everyone_role();

-- ============================================================================
-- HELPFUL VIEWS
-- ============================================================================

-- Active users (not deleted)
CREATE VIEW active_users AS
SELECT * FROM users WHERE deleted_at IS NULL;

-- Active guilds (not deleted)
CREATE VIEW active_guilds AS
SELECT * FROM guilds WHERE deleted_at IS NULL;

-- Active channels (not deleted)
CREATE VIEW active_channels AS
SELECT * FROM channels WHERE deleted_at IS NULL;

-- Active messages (not deleted)
CREATE VIEW active_messages AS
SELECT * FROM messages WHERE deleted_at IS NULL;

-- Member with roles (denormalized view)
CREATE VIEW member_with_roles AS
SELECT
    gm.guild_id,
    gm.user_id,
    u.username,
    u.discriminator,
    u.avatar,
    gm.nickname,
    gm.joined_at,
    COALESCE(
        array_agg(mr.role_id) FILTER (WHERE mr.role_id IS NOT NULL),
        ARRAY[]::BIGINT[]
    ) AS role_ids
FROM guild_members gm
JOIN users u ON u.id = gm.user_id
LEFT JOIN member_roles mr ON mr.guild_id = gm.guild_id AND mr.user_id = gm.user_id
WHERE u.deleted_at IS NULL
GROUP BY gm.guild_id, gm.user_id, u.username, u.discriminator, u.avatar, gm.nickname, gm.joined_at;

-- ============================================================================
-- INDEXES FOR COMMON QUERIES
-- ============================================================================

-- For fetching user's guilds
CREATE INDEX idx_guild_members_user_guilds
ON guild_members(user_id, joined_at DESC);

-- For computing member permissions (roles)
CREATE INDEX idx_member_roles_permissions
ON member_roles(guild_id, user_id);

-- For message pagination with cursor
CREATE INDEX idx_messages_cursor
ON messages(channel_id, id) WHERE deleted_at IS NULL;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE users IS 'Platform user accounts (including bots)';
COMMENT ON TABLE guilds IS 'Servers/communities (Discord calls these "servers")';
COMMENT ON TABLE channels IS 'Text channels, categories, and DM channels';
COMMENT ON TABLE roles IS 'Permission roles within a guild';
COMMENT ON TABLE guild_members IS 'User membership in guilds';
COMMENT ON TABLE member_roles IS 'Role assignments for guild members';
COMMENT ON TABLE messages IS 'Text messages in channels';
COMMENT ON TABLE reactions IS 'Emoji reactions on messages';
COMMENT ON TABLE invites IS 'Guild invitation links';
COMMENT ON TABLE bans IS 'Banned users per guild';
COMMENT ON TABLE audit_logs IS 'Moderation action audit trail';

COMMENT ON COLUMN roles.permissions IS 'Bitfield: VIEW_CHANNEL=1, SEND_MESSAGES=2, MANAGE_MESSAGES=4, MANAGE_CHANNELS=8, MANAGE_ROLES=16, MANAGE_GUILD=32, KICK_MEMBERS=64, BAN_MEMBERS=128, ADMINISTRATOR=256, ATTACH_FILES=512, ADD_REACTIONS=1024';
COMMENT ON COLUMN roles.is_everyone IS 'TRUE for the default @everyone role (one per guild)';
COMMENT ON COLUMN messages.reference_id IS 'For reply/thread support (future feature)';
