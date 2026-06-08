//! NML-Core - N0th1ngness Minecraft Launcher Core Library
//!
//! This crate provides the core functionality for the NML Minecraft launcher,
//! including version management, download engine, account management, and P2P networking.

#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

use std::path::PathBuf;

pub mod config;
pub mod download;
pub mod error;
pub mod ffi;
pub mod launch;
pub mod platform;
pub mod version;
pub mod account;
pub mod mod_manager;
pub mod p2p;

/// Library version
pub const VERSION: &str = "0.1.0";

/// Core configuration
#[derive(Debug, Clone)]
pub struct NMLCore {
    config: config::NMLConfig,
}

impl NMLCore {
    /// Initialize the core library
    pub fn new() -> crate::error::Result<Self> {
        let config = config::NMLConfig::load()?;
        Ok(Self { config })
    }

    /// Initialize with custom config path
    pub fn with_config_path(path: PathBuf) -> crate::error::Result<Self> {
        let config = config::NMLConfig::load_from(path)?;
        Ok(Self { config })
    }

    /// Get the configuration
    pub fn config(&self) -> &config::NMLConfig {
        &self.config
    }

    /// Get the base data directory
    pub fn data_dir(&self) -> PathBuf {
        self.config.data_dir.clone()
    }
}

/// Initialize logging
pub fn init_logging() {
    tracing_subscriber::fmt::init();
}

/// Get the default data directory
pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("NML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_default_data_dir() {
        let dir = default_data_dir();
        assert!(dir.ends_with("NML"));
    }
}
