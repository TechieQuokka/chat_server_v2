//! Client payload definitions
//!
//! Defines the payload structures for client-to-server messages.

use serde::{Deserialize, Serialize};

/// Payload for op 10 (Hello)
///
/// Sent by the server immediately after connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval: u64,
}

impl HelloPayload {
    /// Default heartbeat interval (45 seconds)
    pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 45_000;

    /// Create a new Hello payload with default interval
    #[must_use]
    pub fn new() -> Self {
        Self {
            heartbeat_interval: Self::DEFAULT_HEARTBEAT_INTERVAL,
        }
    }

    /// Create a Hello payload with custom interval
    #[must_use]
    pub fn with_interval(heartbeat_interval: u64) -> Self {
        Self { heartbeat_interval }
    }
}

impl Default for HelloPayload {
    fn default() -> Self {
        Self::new()
    }
}

/// Payload for op 2 (Identify)
///
/// Sent by the client to authenticate the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyPayload {
    /// Authentication token (Bearer token)
    pub token: String,

    /// Optional client properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<IdentifyProperties>,
}

/// Client connection properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyProperties {
    /// Operating system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,

    /// Browser or client name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,

    /// Device type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

impl IdentifyProperties {
    /// Create empty properties
    #[must_use]
    pub fn new() -> Self {
        Self {
            os: None,
            browser: None,
            device: None,
        }
    }

    /// Set operating system
    #[must_use]
    pub fn with_os(mut self, os: impl Into<String>) -> Self {
        self.os = Some(os.into());
        self
    }

    /// Set browser
    #[must_use]
    pub fn with_browser(mut self, browser: impl Into<String>) -> Self {
        self.browser = Some(browser.into());
        self
    }

    /// Set device type
    #[must_use]
    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }
}

impl Default for IdentifyProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl From<IdentifyProperties> for chat_cache::ClientProperties {
    fn from(props: IdentifyProperties) -> Self {
        Self {
            os: props.os,
            browser: props.browser,
            device: props.device,
        }
    }
}

/// Payload for op 3 (Presence Update)
///
/// Sent by the client to update their online status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdatePayload {
    /// New status (online, idle, dnd, offline)
    pub status: String,
}

impl PresenceUpdatePayload {
    /// Valid status values
    pub const VALID_STATUSES: &'static [&'static str] = &["online", "idle", "dnd", "offline"];

    /// Check if the status is valid
    #[must_use]
    pub fn is_valid_status(&self) -> bool {
        Self::VALID_STATUSES.contains(&self.status.as_str())
    }
}

/// Payload for op 4 (Resume)
///
/// Sent by the client to resume a disconnected session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumePayload {
    /// Authentication token
    pub token: String,

    /// Session ID to resume
    pub session_id: String,

    /// Last received sequence number
    pub seq: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_payload() {
        let hello = HelloPayload::new();
        assert_eq!(hello.heartbeat_interval, 45_000);

        let custom = HelloPayload::with_interval(30_000);
        assert_eq!(custom.heartbeat_interval, 30_000);
    }

    #[test]
    fn test_identify_properties() {
        let props = IdentifyProperties::new()
            .with_os("windows")
            .with_browser("rust-client")
            .with_device("desktop");

        assert_eq!(props.os, Some("windows".to_string()));
        assert_eq!(props.browser, Some("rust-client".to_string()));
        assert_eq!(props.device, Some("desktop".to_string()));
    }

    #[test]
    fn test_presence_update_validation() {
        let valid = PresenceUpdatePayload { status: "online".to_string() };
        assert!(valid.is_valid_status());

        let invalid = PresenceUpdatePayload { status: "busy".to_string() };
        assert!(!invalid.is_valid_status());
    }

    #[test]
    fn test_identify_payload_serialization() {
        let payload = IdentifyPayload {
            token: "Bearer token123".to_string(),
            properties: Some(IdentifyProperties::new().with_os("linux")),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("token123"));
        assert!(json.contains("linux"));
    }

    #[test]
    fn test_resume_payload_serialization() {
        let payload = ResumePayload {
            token: "Bearer token123".to_string(),
            session_id: "session456".to_string(),
            seq: 42,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("session456"));
        assert!(json.contains("42"));
    }
}
