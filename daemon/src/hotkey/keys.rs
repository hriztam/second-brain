//! Modifier key definitions and state tracking
//!
//! Provides constants for macOS modifier key flags and a struct
//! for tracking the current state of modifier keys.

use core_graphics::event::CGEventFlags;

/// Modifier key flag masks from macOS CGEventFlags
pub mod flags {
    use core_graphics::event::CGEventFlags;

    /// Control key modifier flag
    pub const CONTROL: CGEventFlags = CGEventFlags::CGEventFlagControl;
    /// Option/Alt key modifier flag  
    pub const OPTION: CGEventFlags = CGEventFlags::CGEventFlagAlternate;
    /// Command key modifier flag
    pub const COMMAND: CGEventFlags = CGEventFlags::CGEventFlagCommand;
}

/// Tracks which modifier keys are currently pressed
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierState {
    /// Control key is held
    pub control: bool,
    /// Option/Alt key is held
    pub option: bool,
    /// Command key is held
    pub command: bool,
}

impl ModifierState {
    /// Create a new ModifierState from CGEventFlags
    pub fn from_flags(flags: CGEventFlags) -> Self {
        Self {
            control: flags.contains(flags::CONTROL),
            option: flags.contains(flags::OPTION),
            command: flags.contains(flags::COMMAND),
        }
    }

    /// Check if all modifiers are released
    pub fn is_empty(&self) -> bool {
        !self.control && !self.option && !self.command
    }

    /// Check if only Control is pressed (for Dictation mode)
    pub fn is_control_only(&self) -> bool {
        self.control && !self.option && !self.command
    }

    /// Check if Control + Option are pressed (for Intelligent mode)
    pub fn is_control_option(&self) -> bool {
        self.control && self.option && !self.command
    }

    /// Check if Control + Command are pressed (for Agent toggle)
    pub fn is_control_command(&self) -> bool {
        self.control && self.command && !self.option
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state() {
        let state = ModifierState::default();
        assert!(state.is_empty());
        assert!(!state.is_control_only());
    }

    #[test]
    fn test_control_only() {
        let state = ModifierState {
            control: true,
            option: false,
            command: false,
        };
        assert!(!state.is_empty());
        assert!(state.is_control_only());
        assert!(!state.is_control_option());
        assert!(!state.is_control_command());
    }

    #[test]
    fn test_control_option() {
        let state = ModifierState {
            control: true,
            option: true,
            command: false,
        };
        assert!(!state.is_control_only());
        assert!(state.is_control_option());
        assert!(!state.is_control_command());
    }

    #[test]
    fn test_control_command() {
        let state = ModifierState {
            control: true,
            option: false,
            command: true,
        };
        assert!(!state.is_control_only());
        assert!(!state.is_control_option());
        assert!(state.is_control_command());
    }
}
