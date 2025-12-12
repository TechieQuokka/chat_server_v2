//! PostgreSQL implementation of ChannelRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Channel;
use chat_core::traits::{ChannelRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::mappers::channel_type_to_str;
use crate::models::ChannelModel;

use super::error::{channel_not_found, map_db_error};

/// PostgreSQL implementation of ChannelRepository
#[derive(Clone)]
pub struct PgChannelRepository {
    pool: PgPool,
}

impl PgChannelRepository {
    /// Create a new PgChannelRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChannelRepository for PgChannelRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Channel>> {
        let result = sqlx::query_as::<_, ChannelModel>(
            r"
            SELECT id, guild_id, name, type::TEXT as type, topic, position, parent_id,
                   created_at, updated_at, deleted_at
            FROM channels
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Channel::from))
    }

    #[instrument(skip(self))]
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Channel>> {
        let results = sqlx::query_as::<_, ChannelModel>(
            r"
            SELECT id, guild_id, name, type::TEXT as type, topic, position, parent_id,
                   created_at, updated_at, deleted_at
            FROM channels
            WHERE guild_id = $1 AND deleted_at IS NULL
            ORDER BY COALESCE(parent_id, id), type = 'category' DESC, position
            ",
        )
        .bind(guild_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Channel::from).collect())
    }

    #[instrument(skip(self))]
    async fn find_dm(&self, user1_id: Snowflake, user2_id: Snowflake) -> RepoResult<Option<Channel>> {
        let result = sqlx::query_as::<_, ChannelModel>(
            r"
            SELECT c.id, c.guild_id, c.name, c.type::TEXT as type, c.topic, c.position, c.parent_id,
                   c.created_at, c.updated_at, c.deleted_at
            FROM channels c
            JOIN dm_channel_recipients r1 ON r1.channel_id = c.id AND r1.user_id = $1
            JOIN dm_channel_recipients r2 ON r2.channel_id = c.id AND r2.user_id = $2
            WHERE c.type = 'dm'
              AND c.deleted_at IS NULL
              AND (SELECT COUNT(*) FROM dm_channel_recipients WHERE channel_id = c.id) = 2
            ",
        )
        .bind(user1_id.into_inner())
        .bind(user2_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Channel::from))
    }

    #[instrument(skip(self))]
    async fn find_dms_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<Channel>> {
        let results = sqlx::query_as::<_, ChannelModel>(
            r"
            SELECT c.id, c.guild_id, c.name, c.type::TEXT as type, c.topic, c.position, c.parent_id,
                   c.created_at, c.updated_at, c.deleted_at
            FROM channels c
            JOIN dm_channel_recipients r ON r.channel_id = c.id
            WHERE r.user_id = $1
              AND c.type = 'dm'
              AND c.deleted_at IS NULL
            ORDER BY c.created_at DESC
            ",
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Channel::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, channel: &Channel) -> RepoResult<()> {
        sqlx::query(
            r"
            INSERT INTO channels (id, guild_id, name, type, topic, position, parent_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4::channel_type, $5, $6, $7, $8, $9)
            ",
        )
        .bind(channel.id.into_inner())
        .bind(channel.guild_id.map(chat_core::Snowflake::into_inner))
        .bind(&channel.name)
        .bind(channel_type_to_str(channel.channel_type))
        .bind(&channel.topic)
        .bind(channel.position)
        .bind(channel.parent_id.map(chat_core::Snowflake::into_inner))
        .bind(channel.created_at)
        .bind(channel.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, channel: &Channel) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE channels
            SET name = $2, topic = $3, position = $4, parent_id = $5, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(channel.id.into_inner())
        .bind(&channel.name)
        .bind(&channel.topic)
        .bind(channel.position)
        .bind(channel.parent_id.map(chat_core::Snowflake::into_inner))
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(channel_not_found(channel.id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE channels
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(channel_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn add_dm_recipient(&self, channel_id: Snowflake, user_id: Snowflake) -> RepoResult<()> {
        sqlx::query(
            r"
            INSERT INTO dm_channel_recipients (channel_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (channel_id, user_id) DO NOTHING
            ",
        )
        .bind(channel_id.into_inner())
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_dm_recipients(&self, channel_id: Snowflake) -> RepoResult<Vec<Snowflake>> {
        let results = sqlx::query_scalar::<_, i64>(
            r"
            SELECT user_id FROM dm_channel_recipients WHERE channel_id = $1
            ",
        )
        .bind(channel_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Snowflake::new).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgChannelRepository>();
    }
}
