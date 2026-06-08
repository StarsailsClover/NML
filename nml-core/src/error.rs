//! Error types for NML-Core

use std::io;
use thiserror::Error;

/// Result type alias for NML-Core
pub type Result<T> = std::result::Result<T, NMLError>;

/// Main error type for NML-Core
#[derive(Error, Debug)]
pub enum NMLError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Version not found
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    /// Version already installed
    #[error("Version already installed: {0}")]
    VersionAlreadyInstalled(String),

    /// Java not found
    #[error("Java not found, version required: {0}")]
    JavaNotFound(u8),

    /// Launch failed
    #[error("Launch failed: {0}")]
    LaunchFailed(String),

    /// Download failed
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// P2P error
    #[error("P2P error: {0}")]
    P2PError(String),

    /// Account error
    #[error("Account error: {0}")]
    AccountError(String),

    /// Mod error
    #[error("Mod error: {0}")]
    ModError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl NMLError {
    /// Create a new generic error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        NMLError::Other(msg.into())
    }
}
