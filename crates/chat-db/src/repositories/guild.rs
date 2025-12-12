//! PostgreSQL implementation of GuildRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Guild;
use chat_core::traits::{GuildRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::GuildModel;

use super::error::{guild_not_found, map_db_error};

/// PostgreSQL implementation of GuildRepository
#[derive(Clone)]
pub struct PgGuildRepository {
    pool: PgPool,
}

impl PgGuildRepository {
    /// Create a new PgGuildRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildRepository for PgGuildRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Guild>> {
        let result = sqlx::query_as::<_, GuildModel>(
            r"
            SELECT id, name, icon, description, owner_id, created_at, updated_at, deleted_at
            FROM guilds
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Guild::from))
    }

    #[instrument(skip(self))]
    async fn find_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<Guild>> {
        let results = sqlx::query_as::<_, GuildModel>(
            r"
            SELECT g.id, g.name, g.icon, g.description, g.owner_id, g.created_at, g.updated_at, g.deleted_at
            FROM guilds g
            JOIN guild_members gm ON gm.guild_id = g.id
            WHERE gm.user_id = $1 AND g.deleted_at IS NULL
            ORDER BY gm.joined_at DESC
            ",
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Guild::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, guild: &Guild) -> RepoResult<()> {
        sqlx::query(
            r"
            INSERT INTO guilds (id, name, icon, description, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
        )
        .bind(guild.id.into_inner())
        .bind(&guild.name)
        .bind(&guild.icon)
        .bind(&guild.description)
        .bind(guild.owner_id.into_inner())
        .bind(guild.created_at)
        .bind(guild.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, guild: &Guild) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE guilds
            SET name = $2, icon = $3, description = $4, owner_id = $5, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(guild.id.into_inner())
        .bind(&guild.name)
        .bind(&guild.icon)
        .bind(&guild.description)
        .bind(guild.owner_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(guild_not_found(guild.id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE guilds
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(guild_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn member_count(&self, guild_id: Snowflake) -> RepoResult<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            r"
            SELECT COUNT(*) FROM guild_members WHERE guild_id = $1
            ",
        )
        .bind(guild_id.into_inner())
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgGuildRepository>();
    }
}
