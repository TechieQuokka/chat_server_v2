//! Message entity - represents a chat message

use chrono::{DateTime, Utc};

use crate::value_objects::Snowflake;

/// Message entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub author_id: Snowflake,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub reference_id: Option<Snowflake>,
}

impl Message {
    /// Create a new Message
    pub fn new(
        id: Snowflake,
        channel_id: Snowflake,
        author_id: Snowflake,
        content: String,
    ) -> Self {
        Self {
            id,
            channel_id,
            author_id,
            content,
            created_at: Utc::now(),
            edited_at: None,
            reference_id: None,
        }
    }

    /// Create a reply message
    pub fn new_reply(
        id: Snowflake,
        channel_id: Snowflake,
        author_id: Snowflake,
        content: String,
        reference_id: Snowflake,
    ) -> Self {
        Self {
            id,
            channel_id,
            author_id,
            content,
            created_at: Utc::now(),
            edited_at: None,
            reference_id: Some(reference_id),
        }
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

    /// Edit the message content
    pub fn edit(&mut self, content: String) {
        self.content = content;
        self.edited_at = Some(Utc::now());
    }

    /// Get a truncated preview of the message (for notifications)
    pub fn preview(&self, max_len: usize) -> &str {
        if self.content.len() <= max_len {
            &self.content
        } else {
            let mut end = max_len;
            while !self.content.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            &self.content[..end]
        }
    }

    /// Check if message content is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}

/// Attachment entity (separate from Message for flexibility)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub id: Snowflake,
    pub message_id: Snowflake,
    pub filename: String,
    pub content_type: String,
    pub size: i32,
    pub url: String,
    pub proxy_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl Attachment {
    /// Create a new Attachment
    pub fn new(
        id: Snowflake,
        message_id: Snowflake,
        filename: String,
        content_type: String,
        size: i32,
        url: String,
    ) -> Self {
        Self {
            id,
            message_id,
            filename,
            content_type,
            size,
            url,
            proxy_url: None,
            width: None,
            height: None,
        }
    }

    /// Check if attachment is an image
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if attachment is a video
    pub fn is_video(&self) -> bool {
        self.content_type.starts_with("video/")
    }

    /// Check if attachment has dimensions (is an image/video)
    pub fn has_dimensions(&self) -> bool {
        self.width.is_some() && self.height.is_some()
    }

    /// Set image/video dimensions
    pub fn set_dimensions(&mut self, width: i32, height: i32) {
        self.width = Some(width);
        self.height = Some(height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            Snowflake::new(1),
            Snowflake::new(100),
            Snowflake::new(200),
            "Hello, world!".to_string(),
        );
        assert!(!msg.is_edited());
        assert!(!msg.is_reply());
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_message_reply() {
        let msg = Message::new_reply(
            Snowflake::new(2),
            Snowflake::new(100),
            Snowflake::new(200),
            "This is a reply".to_string(),
            Snowflake::new(1),
        );
        assert!(msg.is_reply());
        assert_eq!(msg.reference_id, Some(Snowflake::new(1)));
    }

    #[test]
    fn test_message_edit() {
        let mut msg = Message::new(
            Snowflake::new(1),
            Snowflake::new(100),
            Snowflake::new(200),
            "Original".to_string(),
        );
        assert!(!msg.is_edited());

        msg.edit("Edited content".to_string());
        assert!(msg.is_edited());
        assert_eq!(msg.content, "Edited content");
    }

    #[test]
    fn test_message_preview() {
        let msg = Message::new(
            Snowflake::new(1),
            Snowflake::new(100),
            Snowflake::new(200),
            "Hello, world!".to_string(),
        );
        assert_eq!(msg.preview(5), "Hello");
        assert_eq!(msg.preview(100), "Hello, world!");
    }

    #[test]
    fn test_attachment_is_image() {
        let attachment = Attachment::new(
            Snowflake::new(1),
            Snowflake::new(1),
            "image.png".to_string(),
            "image/png".to_string(),
            1024,
            "https://example.com/image.png".to_string(),
        );
        assert!(attachment.is_image());
        assert!(!attachment.is_video());
    }
}
