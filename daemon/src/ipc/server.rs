//! Unix domain socket server for IPC

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use super::protocol::{DaemonStatus, Mode, Request, Response};

/// IPC Server handling client connections
pub struct Server {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    state: Arc<RwLock<ServerState>>,
    shutdown_tx: broadcast::Sender<()>,
}

/// Shared server state
struct ServerState {
    status: DaemonStatus,
    start_time: std::time::Instant,
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
        }));

        info!(?socket_path, "IPC server listening");

        Ok(Self {
            socket_path: socket_path.to_owned(),
            listener: Some(listener),
            state,
            shutdown_tx,
        })
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
            let response = Self::process_request(request, &state).await;

            // Send response
            let response_bytes = serde_json::to_vec(&response)?;
            let response_len = (response_bytes.len() as u32).to_le_bytes();
            
            stream.write_all(&response_len).await?;
            stream.write_all(&response_bytes).await?;
        }
    }

    /// Process a request and return a response
    async fn process_request(request: Request, state: &Arc<RwLock<ServerState>>) -> Response {
        match request {
            Request::Ping => Response::Pong,
            
            Request::GetStatus => {
                let mut state = state.write().await;
                state.status.uptime_secs = state.start_time.elapsed().as_secs();
                Response::Status(state.status.clone())
            }
            
            Request::SetMode { mode } => {
                let mut state = state.write().await;
                let old_mode = state.status.mode;
                state.status.mode = mode;
                info!(?old_mode, ?mode, "mode changed");
                Response::ModeChange { mode, active: mode != Mode::Idle }
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
