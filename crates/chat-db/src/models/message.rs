//! Message database model

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Database model for messages table
#[derive(Debug, Clone, FromRow)]
pub struct MessageModel {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub reference_id: Option<i64>,
}

impl MessageModel {
    /// Check if message is soft deleted
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if message has been edited
    #[inline]
    pub fn is_edited(&self) -> bool {
        self.edited_at.is_some()
    }

    /// Check if message is a reply
    #[inline]
    pub fn is_reply(&self) -> bool {
        self.reference_id.is_some()
    }
}

/// Database model for attachments table
#[derive(Debug, Clone, FromRow)]
pub struct AttachmentModel {
    pub id: i64,
    pub message_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    pub proxy_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl AttachmentModel {
    /// Check if attachment is an image
    #[inline]
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if attachment is a video
    #[inline]
    pub fn is_video(&self) -> bool {
        self.content_type.starts_with("video/")
    }
}
