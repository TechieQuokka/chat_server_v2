//! Message and Attachment entity <-> model mapper

use chat_core::entities::{Attachment, Message};
use chat_core::value_objects::Snowflake;

use crate::models::{AttachmentModel, MessageModel};

/// Convert MessageModel to Message entity
impl From<MessageModel> for Message {
    fn from(model: MessageModel) -> Self {
        Message {
            id: Snowflake::new(model.id),
            channel_id: Snowflake::new(model.channel_id),
            author_id: Snowflake::new(model.author_id),
            content: model.content,
            created_at: model.created_at,
            edited_at: model.edited_at,
            reference_id: model.reference_id.map(Snowflake::new),
        }
    }
}

/// Convert AttachmentModel to Attachment entity
impl From<AttachmentModel> for Attachment {
    fn from(model: AttachmentModel) -> Self {
        Attachment {
            id: Snowflake::new(model.id),
            message_id: Snowflake::new(model.message_id),
            filename: model.filename,
            content_type: model.content_type,
            size: model.size,
            url: model.url,
            proxy_url: model.proxy_url,
            width: model.width,
            height: model.height,
        }
    }
}

/// Convert Message entity reference to values for database insertion
pub struct MessageInsert<'a> {
    pub id: i64,
    pub channel_id: i64,
    pub author_id: i64,
    pub content: &'a str,
    pub reference_id: Option<i64>,
}

impl<'a> MessageInsert<'a> {
    pub fn new(message: &'a Message) -> Self {
        Self {
            id: message.id.into_inner(),
            channel_id: message.channel_id.into_inner(),
            author_id: message.author_id.into_inner(),
            content: &message.content,
            reference_id: message.reference_id.map(chat_core::Snowflake::into_inner),
        }
    }
}

/// Convert Attachment entity reference to values for database insertion
pub struct AttachmentInsert<'a> {
    pub id: i64,
    pub message_id: i64,
    pub filename: &'a str,
    pub content_type: &'a str,
    pub size: i32,
    pub url: &'a str,
    pub proxy_url: Option<&'a str>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl<'a> AttachmentInsert<'a> {
    pub fn new(attachment: &'a Attachment) -> Self {
        Self {
            id: attachment.id.into_inner(),
            message_id: attachment.message_id.into_inner(),
            filename: &attachment.filename,
            content_type: &attachment.content_type,
            size: attachment.size,
            url: &attachment.url,
            proxy_url: attachment.proxy_url.as_deref(),
            width: attachment.width,
            height: attachment.height,
        }
    }
}
