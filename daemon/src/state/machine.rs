//! Core state machine implementation
//!
//! Handles transitions between Idle, DictationActive, IntelligentActive,
//! and AgentActive states based on modifier key events.

use std::time::Instant;

use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::events::StateEvent;
use crate::hotkey::{HotkeyEvent, ModifierState};

/// The four possible states of the daemon
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// No active mode, waiting for hotkey
    Idle,
    /// Dictation mode: Control is held
    DictationActive,
    /// Intelligent mode: Control+Option are held
    IntelligentActive,
    /// Agent mode: Toggled on via Control+Command
    AgentActive,
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Idle => write!(f, "Idle"),
            State::DictationActive => write!(f, "DictationActive"),
            State::IntelligentActive => write!(f, "IntelligentActive"),
            State::AgentActive => write!(f, "AgentActive"),
        }
    }
}

/// The state machine that manages mode transitions
pub struct StateMachine {
    /// Current state
    state: State,
    /// Previous modifier state (for edge detection)
    prev_modifiers: ModifierState,
    /// Time when current non-Idle state was entered
    state_entered_at: Option<Instant>,
    /// Channel for emitting state events
    event_tx: broadcast::Sender<StateEvent>,
}

impl StateMachine {
    /// Create a new state machine
    pub fn new(event_tx: broadcast::Sender<StateEvent>) -> Self {
        Self {
            state: State::Idle,
            prev_modifiers: ModifierState::default(),
            state_entered_at: None,
            event_tx,
        }
    }

    /// Get the current state
    pub fn state(&self) -> State {
        self.state
    }

    /// Run the state machine, processing hotkey events
    pub async fn run(&mut self, mut hotkey_rx: mpsc::Receiver<HotkeyEvent>) {
        info!("state machine started in Idle state");

        while let Some(event) = hotkey_rx.recv().await {
            match event {
                HotkeyEvent::ModifierChanged(modifiers) => {
                    self.handle_modifier_change(modifiers);
                }
                HotkeyEvent::TapDisabled => {
                    warn!("hotkey tap disabled, events may be missed");
                }
            }
        }

        info!("state machine stopped");
    }

    /// Handle a modifier state change
    fn handle_modifier_change(&mut self, modifiers: ModifierState) {
        let old_state = self.state;
        let new_state = self.compute_next_state(&modifiers);

        if new_state != old_state {
            self.transition_to(new_state);
        }

        self.prev_modifiers = modifiers;
    }

    /// Compute the next state based on current state and modifier keys
    fn compute_next_state(&self, modifiers: &ModifierState) -> State {
        match self.state {
            State::Idle => self.compute_from_idle(modifiers),
            State::DictationActive => self.compute_from_dictation(modifiers),
            State::IntelligentActive => self.compute_from_intelligent(modifiers),
            State::AgentActive => self.compute_from_agent(modifiers),
        }
    }

    /// Compute next state when currently Idle
    fn compute_from_idle(&self, modifiers: &ModifierState) -> State {
        // Priority order: Agent > Intelligent > Dictation
        if self.is_rising_edge_control_command(modifiers) {
            State::AgentActive
        } else if modifiers.is_control_option() {
            State::IntelligentActive
        } else if modifiers.is_control_only() {
            State::DictationActive
        } else {
            State::Idle
        }
    }

    /// Compute next state when in DictationActive
    fn compute_from_dictation(&self, modifiers: &ModifierState) -> State {
        // If Option is added, upgrade to Intelligent
        if modifiers.is_control_option() {
            State::IntelligentActive
        }
        // If Control is released, go back to Idle
        else if !modifiers.control {
            State::Idle
        }
        // Stay in Dictation
        else {
            State::DictationActive
        }
    }

    /// Compute next state when in IntelligentActive
    fn compute_from_intelligent(&self, modifiers: &ModifierState) -> State {
        // If either Control or Option is released, go to Idle
        if !modifiers.control || !modifiers.option {
            State::Idle
        } else {
            State::IntelligentActive
        }
    }

    /// Compute next state when in AgentActive
    fn compute_from_agent(&self, modifiers: &ModifierState) -> State {
        // Only Control+Command toggle can exit Agent mode
        if self.is_rising_edge_control_command(modifiers) {
            State::Idle
        } else {
            // All other key combinations are ignored
            State::AgentActive
        }
    }

    /// Detect rising edge of Control+Command (just pressed together)
    fn is_rising_edge_control_command(&self, modifiers: &ModifierState) -> bool {
        // Both Control and Command are now pressed
        modifiers.is_control_command()
            // And at least one of them was not pressed before
            && (!self.prev_modifiers.control || !self.prev_modifiers.command)
    }

    /// Perform a state transition
    fn transition_to(&mut self, new_state: State) {
        let old_state = self.state;
        let duration_ms = self
            .state_entered_at
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0);

        info!(
            from = %old_state,
            to = %new_state,
            duration_ms = duration_ms,
            "state transition"
        );

        // Emit exit event for the old state
        self.emit_exit_event(old_state, duration_ms);

        // Update state
        self.state = new_state;
        self.state_entered_at = if new_state != State::Idle {
            Some(Instant::now())
        } else {
            None
        };

        // Emit entry event for the new state
        self.emit_entry_event(new_state);
    }

    /// Emit an exit event for the given state
    fn emit_exit_event(&self, state: State, duration_ms: u64) {
        let event = match state {
            State::Idle => return, // No exit event for Idle
            State::DictationActive => StateEvent::DictationComplete { duration_ms },
            State::IntelligentActive => StateEvent::IntelligentRequestComplete { duration_ms },
            State::AgentActive => StateEvent::AgentModeExited { duration_ms },
        };

        debug!(?event, "emitting exit event");
        let _ = self.event_tx.send(event);
    }

    /// Emit an entry event for the given state
    fn emit_entry_event(&self, state: State) {
        let event = match state {
            State::Idle => return, // No entry event for Idle
            State::DictationActive => StateEvent::DictationStarted,
            State::IntelligentActive => StateEvent::IntelligentStarted,
            State::AgentActive => StateEvent::AgentModeEntered,
        };

        debug!(?event, "emitting entry event");
        let _ = self.event_tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_state_machine() -> (StateMachine, broadcast::Receiver<StateEvent>) {
        let (tx, rx) = broadcast::channel(16);
        (StateMachine::new(tx), rx)
    }

    #[test]
    fn test_initial_state() {
        let (sm, _) = create_state_machine();
        assert_eq!(sm.state(), State::Idle);
    }

    #[test]
    fn test_idle_to_dictation() {
        let (mut sm, _) = create_state_machine();
        
        let modifiers = ModifierState {
            control: true,
            option: false,
            command: false,
        };
        
        sm.handle_modifier_change(modifiers);
        assert_eq!(sm.state(), State::DictationActive);
    }

    #[test]
    fn test_idle_to_intelligent() {
        let (mut sm, _) = create_state_machine();
        
        let modifiers = ModifierState {
            control: true,
            option: true,
            command: false,
        };
        
        sm.handle_modifier_change(modifiers);
        assert_eq!(sm.state(), State::IntelligentActive);
    }

    #[test]
    fn test_idle_to_agent() {
        let (mut sm, _) = create_state_machine();
        
        let modifiers = ModifierState {
            control: true,
            option: false,
            command: true,
        };
        
        sm.handle_modifier_change(modifiers);
        assert_eq!(sm.state(), State::AgentActive);
    }

    #[test]
    fn test_dictation_to_intelligent_upgrade() {
        let (mut sm, _) = create_state_machine();
        
        // Enter Dictation
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: false,
            command: false,
        });
        assert_eq!(sm.state(), State::DictationActive);
        
        // Add Option -> upgrade to Intelligent
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: true,
            command: false,
        });
        assert_eq!(sm.state(), State::IntelligentActive);
    }

    #[test]
    fn test_agent_ignores_other_modifiers() {
        let (mut sm, _) = create_state_machine();
        
        // Enter Agent
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: false,
            command: true,
        });
        assert_eq!(sm.state(), State::AgentActive);
        
        // Release Command, keep Control -> still Agent
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: false,
            command: false,
        });
        assert_eq!(sm.state(), State::AgentActive);
        
        // Press Control+Option -> still Agent
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: true,
            command: false,
        });
        assert_eq!(sm.state(), State::AgentActive);
    }

    #[test]
    fn test_agent_toggle_off() {
        let (mut sm, _) = create_state_machine();
        
        // Enter Agent
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: false,
            command: true,
        });
        assert_eq!(sm.state(), State::AgentActive);
        
        // Release both
        sm.handle_modifier_change(ModifierState::default());
        assert_eq!(sm.state(), State::AgentActive); // Still in Agent
        
        // Press Control+Command again -> toggle off
        sm.handle_modifier_change(ModifierState {
            control: true,
            option: false,
            command: true,
        });
        assert_eq!(sm.state(), State::Idle);
    }
}
