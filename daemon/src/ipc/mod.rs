//! IPC module for daemon-UI communication

mod protocol;
mod server;

pub use protocol::{Request, Response, DaemonStatus, Mode, Notification};
pub use server::Server;
