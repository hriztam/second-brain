//! Signal handling for graceful shutdown

use tokio::signal::unix::{signal, SignalKind};
use tracing::debug;

/// Handles shutdown signals (SIGTERM, SIGINT)
pub struct ShutdownSignal;

impl ShutdownSignal {
    /// Create a new shutdown signal handler
    pub fn new() -> Self {
        Self
    }

    /// Wait for a shutdown signal
    pub async fn wait(&self) {
        let mut sigterm = signal(SignalKind::terminate())
            .expect("failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt())
            .expect("failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                debug!("received SIGTERM");
            }
            _ = sigint.recv() => {
                debug!("received SIGINT");
            }
        }
    }
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}
