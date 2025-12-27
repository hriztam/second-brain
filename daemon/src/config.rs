//! Configuration loading and management

use std::path::PathBuf;
use anyhow::Result;

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the Unix domain socket for IPC
    pub socket_path: PathBuf,
    
    /// Directory for runtime data
    pub data_dir: PathBuf,
}

impl Config {
    /// Load configuration from environment and defaults
    pub fn load() -> Result<Self> {
        let home = std::env::var("HOME")?;
        let data_dir = PathBuf::from(&home)
            .join(".local")
            .join("share")
            .join("second-brain");
        
        let socket_path = data_dir.join("daemon.sock");

        Ok(Self {
            socket_path,
            data_dir,
        })
    }

    /// Ensure data directory exists
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        let config = Config::load().unwrap();
        assert!(config.socket_path.to_string_lossy().contains("second-brain"));
    }
}
