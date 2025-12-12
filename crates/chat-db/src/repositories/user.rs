//! PostgreSQL implementation of UserRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::User;
use chat_core::error::DomainError;
use chat_core::traits::{RepoResult, UserRepository};
use chat_core::value_objects::Snowflake;

use crate::models::UserModel;

use super::error::{map_db_error, map_unique_violation, user_not_found};

/// PostgreSQL implementation of UserRepository
#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Create a new PgUserRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<User>> {
        let result = sqlx::query_as::<_, UserModel>(
            r"
            SELECT id, username, discriminator, email, password_hash, avatar, bot, system,
                   created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(User::from))
    }

    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>> {
        let result = sqlx::query_as::<_, UserModel>(
            r"
            SELECT id, username, discriminator, email, password_hash, avatar, bot, system,
                   created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            ",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(User::from))
    }

    #[instrument(skip(self))]
    async fn find_by_tag(&self, username: &str, discriminator: &str) -> RepoResult<Option<User>> {
        let result = sqlx::query_as::<_, UserModel>(
            r"
            SELECT id, username, discriminator, email, password_hash, avatar, bot, system,
                   created_at, updated_at, deleted_at
            FROM users
            WHERE username = $1 AND discriminator = $2 AND deleted_at IS NULL
            ",
        )
        .bind(username)
        .bind(discriminator)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(User::from))
    }

    #[instrument(skip(self))]
    async fn email_exists(&self, email: &str) -> RepoResult<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND deleted_at IS NULL)
            ",
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    #[instrument(skip(self, password_hash))]
    async fn create(&self, user: &User, password_hash: &str) -> RepoResult<()> {
        sqlx::query(
            r"
            INSERT INTO users (id, username, discriminator, email, password_hash, avatar, bot, system, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ",
        )
        .bind(user.id.into_inner())
        .bind(&user.username)
        .bind(&user.discriminator)
        .bind(&user.email)
        .bind(password_hash)
        .bind(&user.avatar)
        .bind(user.bot)
        .bind(user.system)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_unique_violation(e, || DomainError::EmailAlreadyExists))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, user: &User) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE users
            SET username = $2, avatar = $3, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(user.id.into_inner())
        .bind(&user.username)
        .bind(&user.avatar)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(user_not_found(user.id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE users
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(user_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_password_hash(&self, id: Snowflake) -> RepoResult<Option<String>> {
        let result = sqlx::query_scalar::<_, String>(
            r"
            SELECT password_hash FROM users WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result)
    }

    #[instrument(skip(self, password_hash))]
    async fn update_password(&self, id: Snowflake, password_hash: &str) -> RepoResult<()> {
        let result = sqlx::query(
            r"
            UPDATE users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(id.into_inner())
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(user_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn next_discriminator(&self, username: &str) -> RepoResult<String> {
        // Find next available discriminator for the username
        let result = sqlx::query_scalar::<_, i32>(
            r"
            SELECT COALESCE(
                (
                    SELECT t.discriminator::INT + 1
                    FROM (
                        SELECT discriminator FROM users
                        WHERE username = $1 AND deleted_at IS NULL
                        ORDER BY discriminator::INT
                    ) t
                    WHERE NOT EXISTS (
                        SELECT 1 FROM users u2
                        WHERE u2.username = $1
                        AND u2.discriminator = LPAD((t.discriminator::INT + 1)::TEXT, 4, '0')
                        AND u2.deleted_at IS NULL
                    )
                    LIMIT 1
                ),
                1
            )
            ",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result > 9999 {
            return Err(DomainError::ValidationError(
                "No available discriminators for this username".to_string(),
            ));
        }

        Ok(format!("{result:04}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgUserRepository>();
    }
}
