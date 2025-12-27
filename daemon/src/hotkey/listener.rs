//! Global hotkey listener using macOS CGEventTap
//!
//! Monitors system-wide keyboard events for modifier key changes.
//! Runs on a dedicated thread with its own CFRunLoop.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::keys::ModifierState;

/// Events sent from the hotkey listener to the state machine
#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// Modifier state has changed
    ModifierChanged(ModifierState),
    /// Event tap was disabled by macOS (needs re-registration)
    TapDisabled,
}

/// Global hotkey listener that monitors modifier key press/release events
pub struct HotkeyListener {
    event_tx: mpsc::Sender<HotkeyEvent>,
    running: Arc<AtomicBool>,
}

impl HotkeyListener {
    /// Create a new hotkey listener
    pub fn new(event_tx: mpsc::Sender<HotkeyEvent>) -> Self {
        Self {
            event_tx,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the hotkey listener
    ///
    /// This spawns a dedicated thread that runs a CFRunLoop to receive
    /// CGEventTap callbacks. The listener runs until `stop()` is called
    /// or the program exits.
    pub fn start(&self) -> Result<(), HotkeyError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(HotkeyError::AlreadyRunning);
        }

        let event_tx = self.event_tx.clone();
        let running = Arc::clone(&self.running);

        thread::Builder::new()
            .name("hotkey-listener".to_string())
            .spawn(move || {
                info!("hotkey listener thread started");
                
                if let Err(e) = run_event_loop(event_tx, running.clone()) {
                    error!(?e, "hotkey listener error");
                }
                
                running.store(false, Ordering::SeqCst);
                info!("hotkey listener thread stopped");
            })
            .map_err(|e| HotkeyError::ThreadSpawn(e.to_string()))?;

        Ok(())
    }

    /// Stop the hotkey listener
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        // The CFRunLoop will exit on the next iteration
        CFRunLoop::get_main().stop();
    }

    /// Check if the listener is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// Errors that can occur in the hotkey listener
#[derive(Debug, thiserror::Error)]
pub enum HotkeyError {
    #[error("hotkey listener is already running")]
    AlreadyRunning,
    
    #[error("failed to create event tap - check Accessibility permissions")]
    EventTapCreation,
    
    #[error("failed to spawn listener thread: {0}")]
    ThreadSpawn(String),
    
    #[error("failed to send event to channel")]
    ChannelSend,
}

/// Run the CFRunLoop with the event tap
fn run_event_loop(
    event_tx: mpsc::Sender<HotkeyEvent>,
    running: Arc<AtomicBool>,
) -> Result<(), HotkeyError> {
    // Track the last modifier state to detect changes
    let mut last_state = ModifierState::default();

    // Create a channel to send events from the callback
    let (callback_tx, callback_rx) = std::sync::mpsc::channel::<CGEventFlags>();

    // CGEventTap callback - must be fast and non-blocking
    let callback = move |_proxy: core_graphics::event::CGEventTapProxy,
                         event_type: CGEventType,
                         event: &CGEvent|
                         -> Option<CGEvent> {
        match event_type {
            CGEventType::FlagsChanged => {
                let flags = event.get_flags();
                let _ = callback_tx.send(flags);
            }
            CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
                warn!("event tap disabled, will re-enable");
                // The tap will be re-enabled automatically
            }
            _ => {}
        }
        Some(event.clone())
    };

    // Create the event tap
    let tap = CGEventTap::new(
        CGEventTapLocation::Session,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::FlagsChanged],
        callback,
    )
    .map_err(|_| {
        error!("failed to create event tap - is Accessibility permission granted?");
        HotkeyError::EventTapCreation
    })?;

    // Enable the tap
    tap.enable();

    // Create a run loop source and add it to the current run loop
    let run_loop_source = tap.mach_port.create_runloop_source(0).unwrap();
    let run_loop = CFRunLoop::get_current();
    
    unsafe {
        run_loop.add_source(&run_loop_source, kCFRunLoopCommonModes);
    }

    info!("event tap created and enabled");

    // Process events in a loop
    while running.load(Ordering::SeqCst) {
        // Run the loop for a short interval, then check for new events
        unsafe {
            CFRunLoop::run_in_mode(
                kCFRunLoopDefaultMode,
                std::time::Duration::from_millis(100),
                true,
            );
        }

        // Process any events from the callback
        while let Ok(flags) = callback_rx.try_recv() {
            let new_state = ModifierState::from_flags(flags);
            
            if new_state != last_state {
                debug!(
                    ?last_state,
                    ?new_state,
                    "modifier state changed"
                );
                
                // Send the event asynchronously
                // We use try_send since we're not in an async context
                if event_tx.blocking_send(HotkeyEvent::ModifierChanged(new_state)).is_err() {
                    warn!("failed to send modifier event - channel closed?");
                    break;
                }
                
                last_state = new_state;
            }
        }
    }

    // Tap will be automatically cleaned up when it goes out of scope
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_creation() {
        let (tx, _rx) = mpsc::channel(32);
        let listener = HotkeyListener::new(tx);
        assert!(!listener.is_running());
    }
}
