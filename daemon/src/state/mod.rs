//! State machine module for mode management
//!
//! Provides an explicit state machine with four states:
//! - Idle: Default state, no audio capture
//! - DictationActive: Momentary, while Control is held
//! - IntelligentActive: Momentary, while Control+Option are held
//! - AgentActive: Toggle, persists until toggled off

mod machine;

pub use machine::{State, StateMachine};
