//! second-brain-daemon: Background daemon for voice-first macOS assistant
//!
//! This daemon runs as a LaunchAgent and provides:
//! - Global hotkey detection via CGEventTap
//! - Explicit state machine for mode management
//! - IPC server for menu bar app communication
//!
//! Phase 0 scope:
//! - System hooks for modifier keys (Control, Option, Command)
//! - State machine with Idle, Dictation, Intelligent, Agent modes
//! - IPC for status queries and mode notifications
//! - NO audio capture, LLM calls, or text insertion

mod config;
mod events;
mod hotkey;
mod ipc;
mod lifecycle;
mod state;

use anyhow::Result;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::events::StateEvent;
use crate::hotkey::HotkeyListener;
use crate::ipc::Server;
use crate::lifecycle::ShutdownSignal;
use crate::state::StateMachine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "second-brain-daemon starting"
    );

    // Load configuration
    let config = Config::load()?;
    info!(?config.socket_path, "configuration loaded");

    // Create shutdown signal handler
    let shutdown = ShutdownSignal::new();

    // Create channels for inter-component communication
    // Hotkey listener -> State machine
    let (hotkey_tx, hotkey_rx) = mpsc::channel(32);
    // State machine -> IPC server (for broadcasting state events)
    let (event_tx, _event_rx) = broadcast::channel::<StateEvent>(64);

    // Create the state machine
    let mut state_machine = StateMachine::new(event_tx.clone());

    // Create the hotkey listener
    let hotkey_listener = HotkeyListener::new(hotkey_tx);

    // Start the hotkey listener (runs on dedicated thread)
    match hotkey_listener.start() {
        Ok(()) => {
            info!("hotkey listener started");
        }
        Err(e) => {
            error!(?e, "failed to start hotkey listener");
            warn!("continuing without hotkey support - check Accessibility permissions");
        }
    }

    // Create IPC server with event subscription
    let server = Server::with_events(&config.socket_path, event_tx.subscribe())?;

    // Subscribe to state events for IPC updates
    let mut ipc_event_rx = event_tx.subscribe();
    let server_for_events = &server;

    info!("daemon initialized, entering main loop");

    // Main event loop
    tokio::select! {
        // Run the state machine (processes hotkey events)
        _ = state_machine.run(hotkey_rx) => {
            info!("state machine exited");
        }
        
        // Run the IPC server (accepts client connections)
        result = server.run() => {
            if let Err(e) = result {
                error!(?e, "IPC server error");
            }
        }
        
        // Handle state events for IPC synchronization
        _ = async {
            loop {
                match ipc_event_rx.recv().await {
                    Ok(event) => {
                        info!(?event, "state event received");
                        // Update the IPC server's view of current state
                        let new_state = match &event {
                            StateEvent::DictationStarted => state::State::DictationActive,
                            StateEvent::DictationComplete { .. } => state::State::Idle,
                            StateEvent::IntelligentStarted => state::State::IntelligentActive,
                            StateEvent::IntelligentRequestComplete { .. } => state::State::Idle,
                            StateEvent::AgentModeEntered => state::State::AgentActive,
                            StateEvent::AgentModeExited { .. } => state::State::Idle,
                            StateEvent::AudioCaptureStarted | StateEvent::AudioCaptureStopped => {
                                continue; // Don't update state for audio events
                            }
                        };
                        server_for_events.set_state(new_state).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "state event receiver lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        } => {
            info!("state event handler exited");
        }
        
        // Wait for shutdown signal
        _ = shutdown.wait() => {
            info!("shutdown signal received");
        }
    }

    // Cleanup
    info!("shutting down...");
    
    hotkey_listener.stop();
    server.shutdown().await;
    
    info!("second-brain-daemon stopped");

    Ok(())
}
