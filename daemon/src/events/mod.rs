//! Events module for state machine transitions
//!
//! Provides structured event types for mode entry, exit, and
//! audio capture state changes (stubs for now).

use serde::{Deserialize, Serialize};

/// Events emitted by the state machine during transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StateEvent {
    /// Entered dictation mode (Control held)
    DictationStarted,
    
    /// Finished dictation (Control released)
    DictationComplete {
        /// Duration in milliseconds that dictation was active
        duration_ms: u64,
    },
    
    /// Entered intelligent mode (Control+Option held)
    IntelligentStarted,
    
    /// Finished intelligent request (keys released)
    IntelligentRequestComplete {
        /// Duration in milliseconds that intelligent mode was active
        duration_ms: u64,
    },
    
    /// Agent mode toggled on (Control+Command)
    AgentModeEntered,
    
    /// Agent mode toggled off (Control+Command again)
    AgentModeExited {
        /// Duration in milliseconds that agent mode was active
        duration_ms: u64,
    },
    
    /// Audio capture started (stub - not implemented in Phase 0)
    AudioCaptureStarted,
    
    /// Audio capture stopped (stub - not implemented in Phase 0)
    AudioCaptureStopped,
}

impl std::fmt::Display for StateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateEvent::DictationStarted => write!(f, "DICTATION_STARTED"),
            StateEvent::DictationComplete { duration_ms } => {
                write!(f, "DICTATION_COMPLETE ({}ms)", duration_ms)
            }
            StateEvent::IntelligentStarted => write!(f, "INTELLIGENT_STARTED"),
            StateEvent::IntelligentRequestComplete { duration_ms } => {
                write!(f, "INTELLIGENT_REQUEST_COMPLETE ({}ms)", duration_ms)
            }
            StateEvent::AgentModeEntered => write!(f, "AGENT_MODE_ENTERED"),
            StateEvent::AgentModeExited { duration_ms } => {
                write!(f, "AGENT_MODE_EXITED ({}ms)", duration_ms)
            }
            StateEvent::AudioCaptureStarted => write!(f, "AUDIO_CAPTURE_STARTED"),
            StateEvent::AudioCaptureStopped => write!(f, "AUDIO_CAPTURE_STOPPED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = StateEvent::DictationComplete { duration_ms: 1500 };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("dictation_complete"));
        assert!(json.contains("1500"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"type":"agent_mode_entered"}"#;
        let event: StateEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, StateEvent::AgentModeEntered));
    }
}
