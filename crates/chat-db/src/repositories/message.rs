//! PostgreSQL implementation of MessageRepository

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use chat_core::entities::{Attachment, Message};
use chat_core::traits::{MessageQuery, MessageRepository, RepoResult};
use chat_core::value_objects::Snowflake;

use crate::models::{AttachmentModel, MessageModel};

use super::error::{map_db_error, message_not_found};

/// PostgreSQL implementation of MessageRepository
#[derive(Clone)]
pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
    /// Create a new PgMessageRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PgMessageRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Snowflake) -> RepoResult<Option<Message>> {
        let result = sqlx::query_as::<_, MessageModel>(
            r#"
            SELECT id, channel_id, author_id, content, created_at, edited_at, deleted_at, reference_id
            FROM messages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(Message::from))
    }

    #[instrument(skip(self))]
    async fn find_by_channel(
        &self,
        channel_id: Snowflake,
        query: MessageQuery,
    ) -> RepoResult<Vec<Message>> {
        let limit = query.limit.clamp(1, 100);

        let results = match (query.before, query.after) {
            (Some(before), None) => {
                // Fetch messages before cursor (scrolling up)
                sqlx::query_as::<_, MessageModel>(
                    r#"
                    SELECT id, channel_id, author_id, content, created_at, edited_at, deleted_at, reference_id
                    FROM messages
                    WHERE channel_id = $1 AND id < $2 AND deleted_at IS NULL
                    ORDER BY id DESC
                    LIMIT $3
                    "#,
                )
                .bind(channel_id.into_inner())
                .bind(before.into_inner())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(after)) => {
                // Fetch messages after cursor (scrolling down)
                sqlx::query_as::<_, MessageModel>(
                    r#"
                    SELECT id, channel_id, author_id, content, created_at, edited_at, deleted_at, reference_id
                    FROM messages
                    WHERE channel_id = $1 AND id > $2 AND deleted_at IS NULL
                    ORDER BY id ASC
                    LIMIT $3
                    "#,
                )
                .bind(channel_id.into_inner())
                .bind(after.into_inner())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
            _ => {
                // Fetch latest messages (no cursor)
                sqlx::query_as::<_, MessageModel>(
                    r#"
                    SELECT id, channel_id, author_id, content, created_at, edited_at, deleted_at, reference_id
                    FROM messages
                    WHERE channel_id = $1 AND deleted_at IS NULL
                    ORDER BY id DESC
                    LIMIT $2
                    "#,
                )
                .bind(channel_id.into_inner())
                .bind(limit)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(map_db_error)?;

        Ok(results.into_iter().map(Message::from).collect())
    }

    #[instrument(skip(self))]
    async fn create(&self, message: &Message) -> RepoResult<()> {
        sqlx::query(
            r#"
            INSERT INTO messages (id, channel_id, author_id, content, created_at, reference_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(message.id.into_inner())
        .bind(message.channel_id.into_inner())
        .bind(message.author_id.into_inner())
        .bind(&message.content)
        .bind(message.created_at)
        .bind(message.reference_id.map(|s| s.into_inner()))
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn update(&self, message: &Message) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET content = $2, edited_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(message.id.into_inner())
        .bind(&message.content)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(message_not_found(message.id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Snowflake) -> RepoResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id.into_inner())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            return Err(message_not_found(id));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn bulk_delete(&self, channel_id: Snowflake, message_ids: &[Snowflake]) -> RepoResult<u64> {
        if message_ids.is_empty() {
            return Ok(0);
        }

        let ids: Vec<i64> = message_ids.iter().map(|s| s.into_inner()).collect();

        let result = sqlx::query(
            r#"
            UPDATE messages
            SET deleted_at = NOW()
            WHERE channel_id = $1 AND id = ANY($2) AND deleted_at IS NULL
            "#,
        )
        .bind(channel_id.into_inner())
        .bind(&ids)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.rows_affected())
    }

    #[instrument(skip(self))]
    async fn find_with_attachments(&self, id: Snowflake) -> RepoResult<Option<(Message, Vec<Attachment>)>> {
        let message = self.find_by_id(id).await?;

        match message {
            Some(msg) => {
                let attachments = sqlx::query_as::<_, AttachmentModel>(
                    r#"
                    SELECT id, message_id, filename, content_type, size, url, proxy_url, width, height, created_at
                    FROM attachments
                    WHERE message_id = $1
                    "#,
                )
                .bind(id.into_inner())
                .fetch_all(&self.pool)
                .await
                .map_err(map_db_error)?;

                Ok(Some((msg, attachments.into_iter().map(Attachment::from).collect())))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PgMessageRepository>();
    }
}
