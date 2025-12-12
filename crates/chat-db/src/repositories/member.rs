//! PostgreSQL implementation of MemberRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::GuildMember;
use chat_core::error::DomainError;
use chat_core::traits::{MemberRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::mappers::member_with_roles;
use crate::models::GuildMemberModel;

use super::error::{map_db_error, map_unique_violation, member_not_found};

/// PostgreSQL implementation of MemberRepository
#[derive(Clone)]
pub struct PgMemberRepository {
    pool: PgPool,
}

impl PgMemberRepository {
    /// Create a new PgMemberRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load role IDs for a member
    async fn load_role_ids(&self, guild_id: i64, user_id: i64) -> Result<Vec<i64>, DomainError> {
        let role_ids = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT role_id FROM member_roles WHERE guild_id = $1 AND user_id = $2
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(role_ids)
    }
}

#[async_trait]
impl MemberRepository for PgMemberRepository {
    #[instrument(skip(self))]
    async fn find(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Option<GuildMember>> {
        let result = sqlx::query_as::<_, GuildMemberModel>(
            r#"
            SELECT guild_id, user_id, nickname, joined_at, updated_at
            FROM guild_members
            WHERE guild_id = $1 AND user_id = $2
            "#,
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        match result {
            Some(model) => {
                let role_ids = self.load_role_ids(model.guild_id, model.user_id).await?;
                Ok(Some(member_with_roles(model, role_ids)))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn find_by_guild(
        &self,
        guild_id: Snowflake,
        limit: i64,
        after: Option<Snowflake>,
    ) -> RepoResult<Vec<GuildMember>> {
        let limit = limit.clamp(1, 1000);

        let results = match after {
            Some(after_id) => {
                sqlx::query_as::<_, GuildMemberModel>(
                    r#"
                    SELECT guild_id, user_id, nickname, joined_at, updated_at
                    FROM guild_members
                    WHERE guild_id = $1 AND user_id > $2
                    ORDER BY user_id
                    LIMIT $3
                    "#,
                )
                .bind(guild_id.into_inner())
                .bind(after_id.into_inner())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, GuildMemberModel>(
                    r#"
                    SELECT guild_id, user_id, nickname, joined_at, updated_at
                    FROM guild_members
                    WHERE guild_id = $1
                    ORDER BY user_id
                    LIMIT $2
                    "#,
                )
                .bind(guild_id.into_inner())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(map_db_error)?;

        // Load role IDs for each member
        let mut members = Vec::with_capacity(results.len());
        for model in results {
            let role_ids = self.load_role_ids(model.guild_id, model.user_id).await?;
            members.push(member_with_roles(model, role_ids));
        }

        Ok(members)
    }

    #[instrument(skip(self))]
    async fn find_by_user(&self, user_id: Snowflake) -> RepoResult<Vec<GuildMember>> {
        let results = sqlx::query_as::<_, GuildMemberModel>(
            r#"
            SELECT guild_id, user_id, nickname, joined_at, updated_at
            FROM guild_members
            WHERE user_id = $1
            ORDER BY joined_at DESC
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let mut members = Vec::with_capacity(results.len());
        for model in results {
            let role_ids = self.load_role_ids(model.guild_id, model.user_id).await?;
            members.push(member_with_roles(model, role_ids));
        }

        Ok(members)
    }

    #[instrument(skip(self))]
    async fn is_member(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(SELECT 1 FROM guild_members WHERE guild_id = $1 AND user_id = $2)
            "#,
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn create(&self, member: &GuildMember) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO guild_members (guild_id, user_id, nickname, joined_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(member.guild_id.into_inner())
        .bind(member.user_id.into_inner())
        .bind(&member.nickname)
        .bind(member.joined_at)
        .bind(member.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_unique_violation(e, || DomainError::AlreadyMember))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, member: &GuildMember) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE guild_members
            SET nickname = $3, updated_at = NOW()
            WHERE guild_id = $1 AND user_id = $2
            "#,
        )
        .bind(member.guild_id.into_inner())
        .bind(member.user_id.into_inner())
        .bind(&member.nickname)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(member_not_found());
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM guild_members WHERE guild_id = $1 AND user_id = $2
            "#,
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(member_not_found());
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn add_role(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        role_id: Snowflake,
    ) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO member_roles (guild_id, user_id, role_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (guild_id, user_id, role_id) DO NOTHING
            "#,
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .bind(role_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn remove_role(
        &self,
        guild_id: Snowflake,
        user_id: Snowflake,
        role_id: Snowflake,
    ) -> RepoResult<()> {
        sqlx::query(
            r#"
            DELETE FROM member_roles WHERE guild_id = $1 AND user_id = $2 AND role_id = $3
            "#,
        )
        .bind(guild_id.into_inner())
        .bind(user_id.into_inner())
        .bind(role_id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_role_ids(&self, guild_id: Snowflake, user_id: Snowflake) -> RepoResult<Vec<Snowflake>> {
        let role_ids = self
            .load_role_ids(guild_id.into_inner(), user_id.into_inner())
            .await?;

        Ok(role_ids.into_iter().map(Snowflake::new).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgMemberRepository>();
    }
}
