//! PostgreSQL implementation of ReactionRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Reaction;
use chat_core::traits::{ReactionRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::{ReactionCountModel, ReactionModel};

use super::error::map_db_error;

/// PostgreSQL implementation of ReactionRepository
#[derive(Clone)]
pub struct PgReactionRepository {
    pool: PgPool,
}

impl PgReactionRepository {
    /// Create a new PgReactionRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReactionRepository for PgReactionRepository {
    #[instrument(skip(self))]
    async fn find(
        &self,
        message_id: Snowflake,
        user_id: Snowflake,
        emoji: &str,
    ) -> RepoResult<Option<Reaction>> {
        let result = sqlx::query_as::<_, ReactionModel>(
            r#"
            SELECT message_id, user_id, emoji, created_at
            FROM reactions
            WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id.into_inner())
        .bind(user_id.into_inner())
        .bind(emoji)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Reaction::from))
    }

    #[instrument(skip(self))]
    async fn find_by_message(&self, message_id: Snowflake) -> RepoResult<Vec<Reaction>> {
        let results = sqlx::query_as::<_, ReactionModel>(
            r#"
            SELECT message_id, user_id, emoji, created_at
            FROM reactions
            WHERE message_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(message_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Reaction::from).collect())
    }

    #[instrument(skip(self))]
    async fn find_users_by_emoji(
        &self,
        message_id: Snowflake,
        emoji: &str,
        limit: i64,
    ) -> RepoResult<Vec<Snowflake>> {
        let limit = limit.clamp(1, 100);

        let results = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT user_id
            FROM reactions
            WHERE message_id = $1 AND emoji = $2
            ORDER BY created_at
            LIMIT $3
            "#,
        )
        .bind(message_id.into_inner())
        .bind(emoji)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Snowflake::new).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, reaction: &Reaction) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO reactions (message_id, user_id, emoji, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id, user_id, emoji) DO NOTHING
            "#,
        )
        .bind(reaction.message_id.into_inner())
        .bind(reaction.user_id.into_inner())
        .bind(&reaction.emoji)
        .bind(reaction.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, message_id: Snowflake, user_id: Snowflake, emoji: &str) -> RepoResult<()> {
        sqlx::query(
            r#"
            DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3
            "#,
        )
        .bind(message_id.into_inner())
        .bind(user_id.into_inner())
        .bind(emoji)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_all(&self, message_id: Snowflake) -> RepoResult<()> {
        sqlx::query(
            r#"
            DELETE FROM reactions WHERE message_id = $1
            "#,
        )
        .bind(message_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_by_emoji(&self, message_id: Snowflake, emoji: &str) -> RepoResult<()> {
        sqlx::query(
            r#"
            DELETE FROM reactions WHERE message_id = $1 AND emoji = $2
            "#,
        )
        .bind(message_id.into_inner())
        .bind(emoji)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn count_by_emoji(&self, message_id: Snowflake) -> RepoResult<Vec<(String, i64)>> {
        let results = sqlx::query_as::<_, ReactionCountModel>(
            r#"
            SELECT emoji, COUNT(*) as count
            FROM reactions
            WHERE message_id = $1
            GROUP BY emoji
            ORDER BY count DESC
            "#,
        )
        .bind(message_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(|r| (r.emoji, r.count)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgReactionRepository>();
    }
}
