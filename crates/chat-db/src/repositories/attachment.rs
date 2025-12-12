//! PostgreSQL implementation of AttachmentRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::Attachment;
use chat_core::traits::{AttachmentRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::AttachmentModel;

use super::error::map_db_error;

/// PostgreSQL implementation of AttachmentRepository
#[derive(Clone)]
pub struct PgAttachmentRepository {
    pool: PgPool,
}

impl PgAttachmentRepository {
    /// Create a new PgAttachmentRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttachmentRepository for PgAttachmentRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Attachment>> {
        let result = sqlx::query_as::<_, AttachmentModel>(
            r#"
            SELECT id, message_id, filename, content_type, size, url, proxy_url, width, height, created_at
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Attachment::from))
    }

    #[instrument(skip(self))]
    async fn find_by_message(&self, message_id: Snowflake) -> RepoResult<Vec<Attachment>> {
        let results = sqlx::query_as::<_, AttachmentModel>(
            r#"
            SELECT id, message_id, filename, content_type, size, url, proxy_url, width, height, created_at
            FROM attachments
            WHERE message_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(message_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Attachment::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, attachment: &Attachment) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO attachments (id, message_id, filename, content_type, size, url, proxy_url, width, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(attachment.id.into_inner())
        .bind(attachment.message_id.into_inner())
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size)
        .bind(&attachment.url)
        .bind(&attachment.proxy_url)
        .bind(attachment.width)
        .bind(attachment.height)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_by_message(&self, message_id: Snowflake) -> RepoResult<()> {
        sqlx::query(
            r#"
            DELETE FROM attachments WHERE message_id = $1
            "#,
        )
        .bind(message_id.into_inner())
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
        assert_send_sync::<PgAttachmentRepository>();
    }
}
