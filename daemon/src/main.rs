//! second-brain-daemon: Background daemon for voice-first macOS assistant
//!
//! This daemon runs as a LaunchAgent and provides:
//! - IPC server for menu bar app communication
//! - Global hotkey registration (future)
//! - Mode state machine management

mod config;
mod ipc;
mod lifecycle;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::Config;
use crate::ipc::Server;
use crate::lifecycle::ShutdownSignal;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    info!("second-brain-daemon starting");

    // Load configuration
    let config = Config::load()?;
    info!(?config.socket_path, "configuration loaded");

    // Create shutdown signal handler
    let shutdown = ShutdownSignal::new();

    // Start IPC server
    let server = Server::new(&config.socket_path)?;
    
    tokio::select! {
        result = server.run() => {
            if let Err(e) = result {
                error!(?e, "IPC server error");
            }
        }
        _ = shutdown.wait() => {
            info!("shutdown signal received");
        }
    }

    // Cleanup
    server.shutdown().await;
    info!("second-brain-daemon stopped");

    Ok(())
}
