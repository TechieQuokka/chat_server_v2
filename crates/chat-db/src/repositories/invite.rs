//! PostgreSQL implementation of InviteRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Invite;
use chat_core::error::DomainError;
use chat_core::traits::{InviteRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::InviteModel;

use super::error::{invite_not_found, map_db_error, map_unique_violation};

/// PostgreSQL implementation of InviteRepository
#[derive(Clone)]
pub struct PgInviteRepository {
    pool: PgPool,
}

impl PgInviteRepository {
    /// Create a new PgInviteRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InviteRepository for PgInviteRepository {
    #[instrument(skip(self))]
    async fn find_by_code(&self, code: &str) -> RepoResult<Option<Invite>> {
        let result = sqlx::query_as::<_, InviteModel>(
            r#"
            SELECT code, guild_id, channel_id, inviter_id, uses, max_uses, max_age,
                   temporary, created_at, expires_at, deleted_at
            FROM invites
            WHERE code = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Invite::from))
    }

    #[instrument(skip(self))]
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Invite>> {
        let results = sqlx::query_as::<_, InviteModel>(
            r#"
            SELECT code, guild_id, channel_id, inviter_id, uses, max_uses, max_age,
                   temporary, created_at, expires_at, deleted_at
            FROM invites
            WHERE guild_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(guild_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Invite::from).collect())
    }

    #[instrument(skip(self))]
    async fn find_by_channel(&self, channel_id: Snowflake) -> RepoResult<Vec<Invite>> {
        let results = sqlx::query_as::<_, InviteModel>(
            r#"
            SELECT code, guild_id, channel_id, inviter_id, uses, max_uses, max_age,
                   temporary, created_at, expires_at, deleted_at
            FROM invites
            WHERE channel_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(channel_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Invite::from).collect())
    }

    #[instrument(skip(self))]
    async fn find_by_inviter(&self, inviter_id: Snowflake) -> RepoResult<Vec<Invite>> {
        let results = sqlx::query_as::<_, InviteModel>(
            r#"
            SELECT code, guild_id, channel_id, inviter_id, uses, max_uses, max_age,
                   temporary, created_at, expires_at, deleted_at
            FROM invites
            WHERE inviter_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(inviter_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Invite::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, invite: &Invite) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO invites (code, guild_id, channel_id, inviter_id, max_uses, max_age,
                                temporary, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&invite.code)
        .bind(invite.guild_id.into_inner())
        .bind(invite.channel_id.into_inner())
        .bind(invite.inviter_id.into_inner())
        .bind(invite.max_uses)
        .bind(invite.max_age)
        .bind(invite.temporary)
        .bind(invite.created_at)
        .bind(invite.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_unique_violation(e, || DomainError::InviteCodeExists))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn increment_uses(&self, code: &str) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE invites
            SET uses = uses + 1
            WHERE code = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(invite_not_found(code));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, code: &str) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE invites
            SET deleted_at = NOW()
            WHERE code = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(invite_not_found(code));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_expired(&self, guild_id: Snowflake) -> RepoResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE invites
            SET deleted_at = NOW()
            WHERE guild_id = $1
              AND deleted_at IS NULL
              AND expires_at IS NOT NULL
              AND expires_at < NOW()
            "#,
        )
        .bind(guild_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgInviteRepository>();
    }
}
