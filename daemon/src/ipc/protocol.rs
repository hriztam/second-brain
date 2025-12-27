//! IPC message protocol definitions
//!
//! All messages are JSON-encoded, prefixed with a 4-byte little-endian length.

use serde::{Deserialize, Serialize};

/// Current operating mode of the daemon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// No active mode, waiting for hotkey
    Idle,
    /// Dictation mode: low-latency transcription
    Dictation,
    /// Intelligent mode: LLM response generation
    Intelligent,
    /// Agent mode: multi-step task execution
    Agent,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Idle
    }
}

/// Requests from UI to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Request current daemon status
    GetStatus,
    
    /// Set the active mode
    SetMode { mode: Mode },
    
    /// Ping to check connectivity
    Ping,
}

/// Responses from daemon to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Current daemon status
    Status(DaemonStatus),
    
    /// Mode change notification
    ModeChange { mode: Mode, active: bool },
    
    /// Pong response to ping
    Pong,
    
    /// Error response
    Error { code: String, message: String },
}

/// Full daemon status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Daemon version
    pub version: String,
    
    /// Current mode
    pub mode: Mode,
    
    /// Whether hotkey is registered
    pub hotkey_registered: bool,
    
    /// Uptime in seconds
    pub uptime_secs: u64,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            mode: Mode::default(),
            hotkey_registered: false,
            uptime_secs: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request::SetMode { mode: Mode::Dictation };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("set_mode"));
        assert!(json.contains("dictation"));
    }

    #[test]
    fn test_response_serialization() {
        let resp = Response::Status(DaemonStatus::default());
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("status"));
    }
}
