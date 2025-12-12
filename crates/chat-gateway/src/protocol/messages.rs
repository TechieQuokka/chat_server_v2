//! Gateway message format
//!
//! Defines the structure for all WebSocket messages.

use super::{CloseCode, HelloPayload, IdentifyPayload, OpCode, PresenceUpdatePayload, ResumePayload};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Gateway message format
///
/// All messages sent over the WebSocket connection follow this format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    /// Operation code
    pub op: OpCode,

    /// Event type (only for op=0 Dispatch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,

    /// Sequence number (only for op=0 Dispatch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,

    /// Event data payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<Value>,
}

impl GatewayMessage {
    // === Server Messages ===

    /// Create a Dispatch message (op=0)
    #[must_use]
    pub fn dispatch(event_type: impl Into<String>, sequence: u64, data: Value) -> Self {
        Self {
            op: OpCode::Dispatch,
            t: Some(event_type.into()),
            s: Some(sequence),
            d: Some(data),
        }
    }

    /// Create a Hello message (op=10)
    #[must_use]
    pub fn hello(payload: HelloPayload) -> Self {
        Self {
            op: OpCode::Hello,
            t: None,
            s: None,
            d: Some(serde_json::to_value(payload).unwrap_or_default()),
        }
    }

    /// Create a Hello message with default heartbeat interval
    #[must_use]
    pub fn hello_default() -> Self {
        Self::hello(HelloPayload::new())
    }

    /// Create a Heartbeat ACK message (op=11)
    #[must_use]
    pub fn heartbeat_ack() -> Self {
        Self {
            op: OpCode::HeartbeatAck,
            t: None,
            s: None,
            d: None,
        }
    }

    /// Create a Reconnect message (op=5)
    #[must_use]
    pub fn reconnect() -> Self {
        Self {
            op: OpCode::Reconnect,
            t: None,
            s: None,
            d: None,
        }
    }

    /// Create an Invalid Session message (op=7)
    ///
    /// `resumable` indicates if the session can be resumed.
    #[must_use]
    pub fn invalid_session(resumable: bool) -> Self {
        Self {
            op: OpCode::InvalidSession,
            t: None,
            s: None,
            d: Some(Value::Bool(resumable)),
        }
    }

    /// Create a Heartbeat message (op=1) from server
    #[must_use]
    pub fn heartbeat(last_sequence: Option<u64>) -> Self {
        Self {
            op: OpCode::Heartbeat,
            t: None,
            s: None,
            d: last_sequence.map(|s| Value::Number(s.into())),
        }
    }

    // === Parsing Client Messages ===

    /// Try to parse as an Identify payload (op=2)
    pub fn as_identify(&self) -> Option<IdentifyPayload> {
        if self.op != OpCode::Identify {
            return None;
        }
        self.d.as_ref().and_then(|d| serde_json::from_value(d.clone()).ok())
    }

    /// Try to parse as a Resume payload (op=4)
    pub fn as_resume(&self) -> Option<ResumePayload> {
        if self.op != OpCode::Resume {
            return None;
        }
        self.d.as_ref().and_then(|d| serde_json::from_value(d.clone()).ok())
    }

    /// Try to parse as a Presence Update payload (op=3)
    pub fn as_presence_update(&self) -> Option<PresenceUpdatePayload> {
        if self.op != OpCode::PresenceUpdate {
            return None;
        }
        self.d.as_ref().and_then(|d| serde_json::from_value(d.clone()).ok())
    }

    /// Try to parse the heartbeat sequence number (op=1)
    pub fn as_heartbeat_seq(&self) -> Option<Option<u64>> {
        if self.op != OpCode::Heartbeat {
            return None;
        }
        Some(self.d.as_ref().and_then(|d| d.as_u64()))
    }

    // === Utilities ===

    /// Check if this is a valid client message
    #[must_use]
    pub fn is_valid_client_message(&self) -> bool {
        self.op.is_client_op()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Create an error close frame
    #[must_use]
    pub fn close_frame(code: CloseCode) -> (u16, String) {
        (code.as_u16(), code.description().to_string())
    }
}

impl std::fmt::Display for GatewayMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(t) = &self.t {
            write!(f, "GatewayMessage(op={}, t={}", self.op, t)?;
            if let Some(s) = self.s {
                write!(f, ", s={s}")?;
            }
            write!(f, ")")
        } else {
            write!(f, "GatewayMessage(op={})", self.op)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_message() {
        let msg = GatewayMessage::dispatch(
            "MESSAGE_CREATE",
            42,
            serde_json::json!({"id": "12345", "content": "Hello"}),
        );

        assert_eq!(msg.op, OpCode::Dispatch);
        assert_eq!(msg.t, Some("MESSAGE_CREATE".to_string()));
        assert_eq!(msg.s, Some(42));
        assert!(msg.d.is_some());
    }

    #[test]
    fn test_hello_message() {
        let msg = GatewayMessage::hello_default();
        assert_eq!(msg.op, OpCode::Hello);

        let json = msg.to_json().unwrap();
        assert!(json.contains("45000"));
    }

    #[test]
    fn test_heartbeat_ack_message() {
        let msg = GatewayMessage::heartbeat_ack();
        assert_eq!(msg.op, OpCode::HeartbeatAck);
        assert!(msg.t.is_none());
        assert!(msg.s.is_none());
        assert!(msg.d.is_none());
    }

    #[test]
    fn test_invalid_session_message() {
        let resumable = GatewayMessage::invalid_session(true);
        assert_eq!(resumable.d, Some(Value::Bool(true)));

        let not_resumable = GatewayMessage::invalid_session(false);
        assert_eq!(not_resumable.d, Some(Value::Bool(false)));
    }

    #[test]
    fn test_parse_identify() {
        let msg = GatewayMessage {
            op: OpCode::Identify,
            t: None,
            s: None,
            d: Some(serde_json::json!({
                "token": "Bearer xyz",
                "properties": {"os": "linux"}
            })),
        };

        let identify = msg.as_identify().unwrap();
        assert_eq!(identify.token, "Bearer xyz");
        assert!(identify.properties.is_some());
    }

    #[test]
    fn test_parse_heartbeat() {
        let msg = GatewayMessage {
            op: OpCode::Heartbeat,
            t: None,
            s: None,
            d: Some(Value::Number(41.into())),
        };

        let seq = msg.as_heartbeat_seq().unwrap();
        assert_eq!(seq, Some(41));

        let msg_null = GatewayMessage {
            op: OpCode::Heartbeat,
            t: None,
            s: None,
            d: None,
        };
        let seq_null = msg_null.as_heartbeat_seq().unwrap();
        assert_eq!(seq_null, None);
    }

    #[test]
    fn test_message_roundtrip() {
        let msg = GatewayMessage::dispatch("READY", 1, serde_json::json!({"v": 1}));
        let json = msg.to_json().unwrap();
        let parsed = GatewayMessage::from_json(&json).unwrap();

        assert_eq!(parsed.op, msg.op);
        assert_eq!(parsed.t, msg.t);
        assert_eq!(parsed.s, msg.s);
    }

    #[test]
    fn test_close_frame() {
        let (code, desc) = GatewayMessage::close_frame(CloseCode::AuthenticationFailed);
        assert_eq!(code, 4004);
        assert!(desc.contains("Authentication"));
    }

    #[test]
    fn test_message_display() {
        let dispatch = GatewayMessage::dispatch("MESSAGE_CREATE", 5, serde_json::json!({}));
        let display = format!("{}", dispatch);
        assert!(display.contains("MESSAGE_CREATE"));
        assert!(display.contains("s=5"));

        let hello = GatewayMessage::hello_default();
        let display2 = format!("{}", hello);
        assert!(display2.contains("Hello"));
    }
}
