//! PostgreSQL implementation of BanRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::traits::{Ban, BanRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::BanModel;

use super::error::{ban_not_found, map_db_error};

/// PostgreSQL implementation of BanRepository
#[derive(Clone)]
pub struct PgBanRepository {
    pool: PgPool,
}

impl PgBanRepository {
    /// Create a new PgBanRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl From<BanModel> for Ban {
    fn from(model: BanModel) -> Self {
        Ban {
            guild_id: Snowflake::new(model.guild_id),
            user_id: Snowflake::new(model.user_id),
            reason: model.reason,
        }
    }
}

#[async_trait]
impl BanRepository for PgBanRepository {
    #[instrument(skip(self))]
    async fn is_banned(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS(SELECT 1 FROM bans WHERE guild_id = $1 AND user_id = $2)
            ",
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn find(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Option<Ban>> {
        let result = sqlx::query_as::<_, BanModel>(
            r"
            SELECT guild_id, user_id, reason, banned_by, created_at
            FROM bans
            WHERE guild_id = $1 AND user_id = $2
            ",
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Ban::from))
    }

    #[instrument(skip(self))]
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Ban>> {
        let results = sqlx::query_as::<_, BanModel>(
            r"
            SELECT guild_id, user_id, reason, banned_by, created_at
            FROM bans
            WHERE guild_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(guild_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Ban::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, ban: &Ban) -> RepoResult<()> {
        // Note: banned_by is not in the Ban struct from traits, so we use the user_id
        // In practice, the service layer should pass the moderator ID
        sqlx::query(
            r"
            INSERT INTO bans (guild_id, user_id, reason, banned_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id, user_id) DO UPDATE SET reason = $3
            ",
        )
        .bind(ban.guild_id.into_inner())
        .bind(ban.user_id.into_inner())
        .bind(&ban.reason)
        .bind(ban.user_id.into_inner()) // This should be moderator ID in practice
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            DELETE FROM bans WHERE guild_id = $1 AND user_id = $2
            ",
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(ban_not_found());
        }

        Ok(())
    }
}

/// Extended ban operations that include moderator info
impl PgBanRepository {
    /// Create a ban with explicit moderator ID
    #[instrument(skip(self))]
    pub async fn create_with_moderator(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        moderator_id: Snowflake,
        reason: Option<&str>,
    ) -> RepoResult<()> {
        sqlx::query(
            r"
            INSERT INTO bans (guild_id, user_id, reason, banned_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id, user_id) DO UPDATE SET reason = $3, banned_by = $4
            ",
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .bind(reason)
        .bind(moderator_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgBanRepository>();
    }
}
