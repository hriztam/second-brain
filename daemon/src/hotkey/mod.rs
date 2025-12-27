//! Hotkey module for global keyboard event listening
//!
//! Uses macOS CGEventTap to monitor modifier key press/release events
//! for triggering mode transitions.

mod keys;
mod listener;

pub use keys::ModifierState;
pub use listener::{HotkeyEvent, HotkeyListener};

