//! Gateway protocol definitions
//!
//! Defines the WebSocket protocol including op codes, message formats, and close codes.

mod close_codes;
mod messages;
mod opcodes;
mod payloads;

pub use close_codes::CloseCode;
pub use messages::GatewayMessage;
pub use opcodes::OpCode;
pub use payloads::{
    HelloPayload, IdentifyPayload, IdentifyProperties, PresenceUpdatePayload, ResumePayload,
};
