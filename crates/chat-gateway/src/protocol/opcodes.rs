//! Gateway operation codes
//!
//! Defines all WebSocket gateway op codes per the protocol specification.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Gateway operation codes
///
/// Op codes define the type of message being sent or received over the WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    /// Server dispatches an event to the client (server only)
    Dispatch = 0,
    /// Heartbeat - keep connection alive (client/server)
    Heartbeat = 1,
    /// Identify - authenticate session (client only)
    Identify = 2,
    /// Presence Update - update online status (client only)
    PresenceUpdate = 3,
    /// Resume - resume dropped connection (client only)
    Resume = 4,
    /// Reconnect - server requests client reconnect (server only)
    Reconnect = 5,
    /// Invalid Session - session is invalid (server only)
    InvalidSession = 7,
    /// Hello - sent on connect (server only)
    Hello = 10,
    /// Heartbeat ACK - heartbeat acknowledged (server only)
    HeartbeatAck = 11,
}

impl OpCode {
    /// Create an `OpCode` from a raw integer value
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Dispatch),
            1 => Some(Self::Heartbeat),
            2 => Some(Self::Identify),
            3 => Some(Self::PresenceUpdate),
            4 => Some(Self::Resume),
            5 => Some(Self::Reconnect),
            7 => Some(Self::InvalidSession),
            10 => Some(Self::Hello),
            11 => Some(Self::HeartbeatAck),
            _ => None,
        }
    }

    /// Get the raw integer value
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Check if this op code can be sent by the client
    #[must_use]
    pub const fn is_client_op(self) -> bool {
        matches!(
            self,
            Self::Heartbeat | Self::Identify | Self::PresenceUpdate | Self::Resume
        )
    }

    /// Check if this op code can be sent by the server
    #[must_use]
    pub const fn is_server_op(self) -> bool {
        matches!(
            self,
            Self::Dispatch
                | Self::Heartbeat
                | Self::Reconnect
                | Self::InvalidSession
                | Self::Hello
                | Self::HeartbeatAck
        )
    }

    /// Get the name of this op code
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dispatch => "Dispatch",
            Self::Heartbeat => "Heartbeat",
            Self::Identify => "Identify",
            Self::PresenceUpdate => "PresenceUpdate",
            Self::Resume => "Resume",
            Self::Reconnect => "Reconnect",
            Self::InvalidSession => "InvalidSession",
            Self::Hello => "Hello",
            Self::HeartbeatAck => "HeartbeatAck",
        }
    }
}

impl Serialize for OpCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for OpCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::from_u8(value).ok_or_else(|| serde::de::Error::custom(format!("invalid op code: {value}")))
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name(), self.as_u8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(OpCode::from_u8(0), Some(OpCode::Dispatch));
        assert_eq!(OpCode::from_u8(1), Some(OpCode::Heartbeat));
        assert_eq!(OpCode::from_u8(2), Some(OpCode::Identify));
        assert_eq!(OpCode::from_u8(3), Some(OpCode::PresenceUpdate));
        assert_eq!(OpCode::from_u8(4), Some(OpCode::Resume));
        assert_eq!(OpCode::from_u8(5), Some(OpCode::Reconnect));
        assert_eq!(OpCode::from_u8(7), Some(OpCode::InvalidSession));
        assert_eq!(OpCode::from_u8(10), Some(OpCode::Hello));
        assert_eq!(OpCode::from_u8(11), Some(OpCode::HeartbeatAck));
        assert_eq!(OpCode::from_u8(6), None);
        assert_eq!(OpCode::from_u8(255), None);
    }

    #[test]
    fn test_opcode_as_u8() {
        assert_eq!(OpCode::Dispatch.as_u8(), 0);
        assert_eq!(OpCode::Heartbeat.as_u8(), 1);
        assert_eq!(OpCode::Hello.as_u8(), 10);
    }

    #[test]
    fn test_client_ops() {
        assert!(OpCode::Heartbeat.is_client_op());
        assert!(OpCode::Identify.is_client_op());
        assert!(OpCode::PresenceUpdate.is_client_op());
        assert!(OpCode::Resume.is_client_op());
        assert!(!OpCode::Dispatch.is_client_op());
        assert!(!OpCode::Hello.is_client_op());
    }

    #[test]
    fn test_server_ops() {
        assert!(OpCode::Dispatch.is_server_op());
        assert!(OpCode::Heartbeat.is_server_op());
        assert!(OpCode::Reconnect.is_server_op());
        assert!(OpCode::InvalidSession.is_server_op());
        assert!(OpCode::Hello.is_server_op());
        assert!(OpCode::HeartbeatAck.is_server_op());
        assert!(!OpCode::Identify.is_server_op());
        assert!(!OpCode::Resume.is_server_op());
    }

    #[test]
    fn test_opcode_serialization() {
        let json = serde_json::to_string(&OpCode::Hello).unwrap();
        assert_eq!(json, "10");

        let op: OpCode = serde_json::from_str("2").unwrap();
        assert_eq!(op, OpCode::Identify);
    }

    #[test]
    fn test_opcode_display() {
        assert_eq!(format!("{}", OpCode::Hello), "Hello (10)");
        assert_eq!(format!("{}", OpCode::Dispatch), "Dispatch (0)");
    }
}
