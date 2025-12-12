//! PostgreSQL implementation of RoleRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Role;
use chat_core::error::DomainError;
use chat_core::traits::{RepoResult, RoleRepository};
use chat_core::value_objects::Snowflake;

use crate::models::RoleModel;

use super::error::{map_db_error, role_not_found};

/// PostgreSQL implementation of RoleRepository
#[derive(Clone)]
pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    /// Create a new PgRoleRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Role>> {
        let result = sqlx::query_as::<_, RoleModel>(
            r#"
            SELECT id, guild_id, name, color, hoist, position, permissions, mentionable,
                   is_everyone, created_at, updated_at, deleted_at
            FROM roles
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Role::from))
    }

    #[instrument(skip(self))]
    async fn find_by_guild(&self, guild_id: Snowflake) -> RepoResult<Vec<Role>> {
        let results = sqlx::query_as::<_, RoleModel>(
            r#"
            SELECT id, guild_id, name, color, hoist, position, permissions, mentionable,
                   is_everyone, created_at, updated_at, deleted_at
            FROM roles
            WHERE guild_id = $1 AND deleted_at IS NULL
            ORDER BY position DESC
            "#,
        )
        .bind(guild_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Role::from).collect())
    }

    #[instrument(skip(self))]
    async fn find_everyone(&self, guild_id: Snowflake) -> RepoResult<Option<Role>> {
        let result = sqlx::query_as::<_, RoleModel>(
            r#"
            SELECT id, guild_id, name, color, hoist, position, permissions, mentionable,
                   is_everyone, created_at, updated_at, deleted_at
            FROM roles
            WHERE guild_id = $1 AND is_everyone = TRUE AND deleted_at IS NULL
            "#,
        )
        .bind(guild_id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Role::from))
    }

    #[instrument(skip(self))]
    async fn create(&self, role: &Role) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO roles (id, guild_id, name, color, hoist, position, permissions, mentionable,
                              is_everyone, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(role.id.into_inner())
        .bind(role.guild_id.into_inner())
        .bind(&role.name)
        .bind(role.color)
        .bind(role.hoist)
        .bind(role.position)
        .bind(role.permissions.to_i64())
        .bind(role.mentionable)
        .bind(role.is_everyone)
        .bind(role.created_at)
        .bind(role.updated_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, role: &Role) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE roles
            SET name = $2, color = $3, hoist = $4, position = $5, permissions = $6,
                mentionable = $7, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(role.id.into_inner())
        .bind(&role.name)
        .bind(role.color)
        .bind(role.hoist)
        .bind(role.position)
        .bind(role.permissions.to_i64())
        .bind(role.mentionable)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(role_not_found(role.id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Snowflake) -> RepoResult<()> {
        // Check if it's the @everyone role
        let is_everyone = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT is_everyone FROM roles WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        if is_everyone == Some(true) {
            return Err(DomainError::CannotDeleteEveryoneRole);
        }

        let result = sqlx::query(
            r#"
            UPDATE roles
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(role_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update_positions(
        &self,
        guild_id: Snowflake,
        positions: &[(Snowflake, i32)],
    ) -> RepoResult<()> {
        // Use a transaction for bulk position update
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        for (role_id, position) in positions {
            sqlx::query(
                r#"
                UPDATE roles
                SET position = $3, updated_at = NOW()
                WHERE id = $1 AND guild_id = $2 AND deleted_at IS NULL AND is_everyone = FALSE
                "#,
            )
            .bind(role_id.into_inner())
            .bind(guild_id.into_inner())
            .bind(position)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        tx.commit().await.map_err(map_db_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgRoleRepository>();
    }
}
