//! WebSocket close codes
//!
//! Defines gateway-specific close codes for WebSocket connections.

use serde::{Deserialize, Serialize};

/// Gateway WebSocket close codes
///
/// These codes are sent when closing a WebSocket connection to indicate the reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum CloseCode {
    /// Unknown error occurred
    UnknownError = 4000,
    /// Invalid opcode sent
    UnknownOpcode = 4001,
    /// Invalid payload encoding (JSON decode error)
    DecodeError = 4002,
    /// Sent payload before Identify
    NotAuthenticated = 4003,
    /// Invalid token provided
    AuthenticationFailed = 4004,
    /// Sent Identify twice
    AlreadyAuthenticated = 4005,
    /// Invalid sequence number for Resume
    InvalidSequence = 4007,
    /// Too many requests (rate limited)
    RateLimited = 4008,
    /// Session has timed out
    SessionTimeout = 4009,
    /// Invalid shard configuration
    InvalidShard = 4010,
    /// Sharding is required
    ShardingRequired = 4011,
    /// Invalid/outdated API version
    InvalidApiVersion = 4012,
}

impl CloseCode {
    /// Create a `CloseCode` from a raw u16 value
    #[must_use]
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            4000 => Some(Self::UnknownError),
            4001 => Some(Self::UnknownOpcode),
            4002 => Some(Self::DecodeError),
            4003 => Some(Self::NotAuthenticated),
            4004 => Some(Self::AuthenticationFailed),
            4005 => Some(Self::AlreadyAuthenticated),
            4007 => Some(Self::InvalidSequence),
            4008 => Some(Self::RateLimited),
            4009 => Some(Self::SessionTimeout),
            4010 => Some(Self::InvalidShard),
            4011 => Some(Self::ShardingRequired),
            4012 => Some(Self::InvalidApiVersion),
            _ => None,
        }
    }

    /// Get the raw u16 value
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Check if the client should attempt to reconnect after this close code
    #[must_use]
    pub const fn should_reconnect(self) -> bool {
        matches!(
            self,
            Self::UnknownError
                | Self::UnknownOpcode
                | Self::DecodeError
                | Self::AlreadyAuthenticated
                | Self::InvalidSequence
                | Self::RateLimited
                | Self::SessionTimeout
        )
    }

    /// Get the description for this close code
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::UnknownError => "Unknown error occurred",
            Self::UnknownOpcode => "Invalid opcode sent",
            Self::DecodeError => "Invalid payload encoding",
            Self::NotAuthenticated => "Not authenticated",
            Self::AuthenticationFailed => "Authentication failed",
            Self::AlreadyAuthenticated => "Already authenticated",
            Self::InvalidSequence => "Invalid sequence number",
            Self::RateLimited => "Rate limited",
            Self::SessionTimeout => "Session timeout",
            Self::InvalidShard => "Invalid shard configuration",
            Self::ShardingRequired => "Sharding required",
            Self::InvalidApiVersion => "Invalid API version",
        }
    }

    /// Get the name of this close code
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnknownError => "UnknownError",
            Self::UnknownOpcode => "UnknownOpcode",
            Self::DecodeError => "DecodeError",
            Self::NotAuthenticated => "NotAuthenticated",
            Self::AuthenticationFailed => "AuthenticationFailed",
            Self::AlreadyAuthenticated => "AlreadyAuthenticated",
            Self::InvalidSequence => "InvalidSequence",
            Self::RateLimited => "RateLimited",
            Self::SessionTimeout => "SessionTimeout",
            Self::InvalidShard => "InvalidShard",
            Self::ShardingRequired => "ShardingRequired",
            Self::InvalidApiVersion => "InvalidApiVersion",
        }
    }
}

impl std::fmt::Display for CloseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {}", self.name(), self.as_u16(), self.description())
    }
}

impl From<CloseCode> for u16 {
    fn from(code: CloseCode) -> Self {
        code.as_u16()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_code_from_u16() {
        assert_eq!(CloseCode::from_u16(4000), Some(CloseCode::UnknownError));
        assert_eq!(CloseCode::from_u16(4004), Some(CloseCode::AuthenticationFailed));
        assert_eq!(CloseCode::from_u16(4012), Some(CloseCode::InvalidApiVersion));
        assert_eq!(CloseCode::from_u16(1000), None);
        assert_eq!(CloseCode::from_u16(4006), None); // 4006 is not defined
    }

    #[test]
    fn test_close_code_as_u16() {
        assert_eq!(CloseCode::UnknownError.as_u16(), 4000);
        assert_eq!(CloseCode::AuthenticationFailed.as_u16(), 4004);
        assert_eq!(CloseCode::InvalidApiVersion.as_u16(), 4012);
    }

    #[test]
    fn test_should_reconnect() {
        // Should reconnect
        assert!(CloseCode::UnknownError.should_reconnect());
        assert!(CloseCode::UnknownOpcode.should_reconnect());
        assert!(CloseCode::DecodeError.should_reconnect());
        assert!(CloseCode::AlreadyAuthenticated.should_reconnect());
        assert!(CloseCode::InvalidSequence.should_reconnect());
        assert!(CloseCode::RateLimited.should_reconnect());
        assert!(CloseCode::SessionTimeout.should_reconnect());

        // Should not reconnect
        assert!(!CloseCode::NotAuthenticated.should_reconnect());
        assert!(!CloseCode::AuthenticationFailed.should_reconnect());
        assert!(!CloseCode::InvalidShard.should_reconnect());
        assert!(!CloseCode::ShardingRequired.should_reconnect());
        assert!(!CloseCode::InvalidApiVersion.should_reconnect());
    }

    #[test]
    fn test_close_code_display() {
        let code = CloseCode::AuthenticationFailed;
        let display = format!("{}", code);
        assert!(display.contains("4004"));
        assert!(display.contains("Authentication"));
    }
}
