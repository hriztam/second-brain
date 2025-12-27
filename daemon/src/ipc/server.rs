//! Unix domain socket server for IPC
//!
//! Provides request-response communication and push notifications for
//! state change events to subscribed clients.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::events::StateEvent;
use crate::state::State;

use super::protocol::{DaemonStatus, Mode, Notification, Request, Response};

/// IPC Server handling client connections
pub struct Server {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    state: Arc<RwLock<ServerState>>,
    shutdown_tx: broadcast::Sender<()>,
    /// Channel for receiving state events to broadcast to subscribed clients
    event_rx: Option<broadcast::Receiver<StateEvent>>,
}

/// Shared server state
struct ServerState {
    status: DaemonStatus,
    start_time: std::time::Instant,
    /// Current internal state (for mode tracking)
    current_state: State,
}

impl Server {
    /// Create a new IPC server
    pub fn new(socket_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)
                .context("failed to create socket directory")?;
        }

        // Remove stale socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path)
                .context("failed to remove stale socket")?;
        }

        let listener = UnixListener::bind(socket_path)
            .context("failed to bind Unix socket")?;

        // Set socket permissions to owner-only (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        let (shutdown_tx, _) = broadcast::channel(1);

        let state = Arc::new(RwLock::new(ServerState {
            status: DaemonStatus::default(),
            start_time: std::time::Instant::now(),
            current_state: State::Idle,
        }));

        info!(?socket_path, "IPC server listening");

        Ok(Self {
            socket_path: socket_path.to_owned(),
            listener: Some(listener),
            state,
            shutdown_tx,
            event_rx: None,
        })
    }

    /// Create a new IPC server with state event subscription
    pub fn with_events(socket_path: &Path, event_rx: broadcast::Receiver<StateEvent>) -> Result<Self> {
        let mut server = Self::new(socket_path)?;
        server.event_rx = Some(event_rx);
        Ok(server)
    }

    /// Update the current mode in server state
    pub async fn set_state(&self, state: State) {
        let mut server_state = self.state.write().await;
        let old_state = server_state.current_state;
        server_state.current_state = state;
        server_state.status.mode = state.into();
        server_state.status.hotkey_registered = true;
        
        if old_state != state {
            info!(
                from = ?old_state,
                to = ?state,
                "IPC server: mode updated"
            );
        }
    }

    /// Run the server, accepting connections
    pub async fn run(&self) -> Result<()> {
        let listener = self.listener.as_ref()
            .context("server not initialized")?;

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("client connected");
                    let state = Arc::clone(&self.state);
                    let mut shutdown_rx = self.shutdown_tx.subscribe();
                    
                    tokio::spawn(async move {
                        tokio::select! {
                            result = Self::handle_client(stream, state) => {
                                if let Err(e) = result {
                                    warn!(?e, "client handler error");
                                }
                            }
                            _ = shutdown_rx.recv() => {
                                debug!("client handler shutting down");
                            }
                        }
                    });
                }
                Err(e) => {
                    error!(?e, "accept error");
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_client(mut stream: UnixStream, state: Arc<RwLock<ServerState>>) -> Result<()> {
        let mut len_buf = [0u8; 4];
        let mut is_subscribed = false;

        loop {
            // Read message length (4-byte little-endian)
            match stream.read_exact(&mut len_buf).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    debug!("client disconnected");
                    return Ok(());
                }
                Err(e) => return Err(e.into()),
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            if len > 1024 * 1024 {
                warn!(len, "message too large, disconnecting");
                return Ok(());
            }

            // Read message body
            let mut msg_buf = vec![0u8; len];
            stream.read_exact(&mut msg_buf).await?;

            // Parse request
            let request: Request = serde_json::from_slice(&msg_buf)
                .context("failed to parse request")?;
            
            debug!(?request, "received request");

            // Process request
            let (response, subscribe) = Self::process_request(request, &state).await;
            if subscribe {
                is_subscribed = true;
                debug!("client subscribed to notifications");
            }

            // Send response
            Self::send_message(&mut stream, &response).await?;
        }
    }

    /// Send a length-prefixed JSON message
    async fn send_message<T: serde::Serialize>(stream: &mut UnixStream, msg: &T) -> Result<()> {
        let msg_bytes = serde_json::to_vec(msg)?;
        let msg_len = (msg_bytes.len() as u32).to_le_bytes();
        
        stream.write_all(&msg_len).await?;
        stream.write_all(&msg_bytes).await?;
        
        Ok(())
    }

    /// Process a request and return a response
    /// Returns (Response, should_subscribe)
    async fn process_request(request: Request, state: &Arc<RwLock<ServerState>>) -> (Response, bool) {
        match request {
            Request::Ping => (Response::Pong, false),
            
            Request::GetStatus => {
                let mut state = state.write().await;
                state.status.uptime_secs = state.start_time.elapsed().as_secs();
                (Response::Status(state.status.clone()), false)
            }
            
            Request::SetMode { mode } => {
                let mut state = state.write().await;
                let old_mode = state.status.mode;
                state.status.mode = mode;
                info!(?old_mode, ?mode, "mode changed via IPC");
                (Response::ModeChange { mode, active: mode != Mode::Idle }, false)
            }
            
            Request::Subscribe => {
                (Response::Subscribed, true)
            }
        }
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
        
        // Remove socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!(?e, "failed to remove socket file");
            }
        }
        
        info!("IPC server shutdown complete");
    }
}
