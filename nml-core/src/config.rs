//! Configuration management for NML

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NMLConfig {
    /// Data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Game directory (.minecraft)
    #[serde(default = "default_game_dir")]
    pub game_dir: PathBuf,

    /// Java settings
    #[serde(default)]
    pub java: JavaConfig,

    /// Download settings
    #[serde(default)]
    pub download: DownloadConfig,

    /// Launch settings
    #[serde(default)]
    pub launch: LaunchConfig,

    /// P2P settings
    #[serde(default)]
    pub p2p: P2PConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UIConfig,
}

impl NMLConfig {
    /// Load configuration from default location
    pub fn load() -> Result<Self> {
        let config_path = default_config_path();
        Self::load_from(config_path)
    }

    /// Load configuration from specific path
    pub fn load_from(path: PathBuf) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: NMLConfig = serde_yaml::from_str(&content)?;
            Ok(config)
        } else {
            let config = NMLConfig::default();
            config.save_to(&path)?;
            Ok(config)
        }
    }

    /// Save configuration to default location
    pub fn save(&self) -> Result<()> {
        self.save_to(&default_config_path())
    }

    /// Save configuration to specific path
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(path.parent().unwrap_or(PathBuf::from(".").as_path()))?;
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for NMLConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            game_dir: default_game_dir(),
            java: JavaConfig::default(),
            download: DownloadConfig::default(),
            launch: LaunchConfig::default(),
            p2p: P2PConfig::default(),
            ui: UIConfig::default(),
        }
    }
}

/// Java configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    /// Auto-detect Java
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    /// Custom Java paths
    #[serde(default)]
    pub custom_paths: Vec<PathBuf>,

    /// Default memory allocation (MB)
    #[serde(default = "default_memory")]
    pub max_memory: u32,

    /// Minimum memory (MB)
    #[serde(default = "default_min_memory")]
    pub min_memory: u32,

    /// JVM arguments
    #[serde(default)]
    pub jvm_args: Vec<String>,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            custom_paths: Vec::new(),
            max_memory: 4096,
            min_memory: 512,
            jvm_args: Vec::new(),
        }
    }
}

/// Download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Max concurrent downloads
    #[serde(default = "default_concurrent")]
    pub max_concurrent: u32,

    /// Use mirror sources
    #[serde(default = "default_true")]
    pub use_mirror: bool,

    /// Mirror sources priority
    #[serde(default = "default_mirrors")]
    pub mirrors: Vec<String>,

    /// Enable resume
    #[serde(default = "default_true")]
    pub resume: bool,

    /// Chunk size for large files (bytes)
    #[serde(default = "default_chunk_size")]
    pub chunk_size: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 8,
            use_mirror: true,
            mirrors: vec![
                "https://bmclapi2.bangbang93.com".to_string(),
                "https://download.mcbbs.net".to_string(),
            ],
            resume: true,
            chunk_size: 1024 * 1024, // 1MB
        }
    }
}

/// Launch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Auto-optimize JVM args
    #[serde(default = "default_true")]
    pub auto_optimize: bool,

    /// Enable MCJEBooster
    #[serde(default = "default_true")]
    pub enable_booster: bool,

    /// Window width
    #[serde(default)]
    pub window_width: Option<u32>,

    /// Window height
    #[serde(default)]
    pub window_height: Option<u32>,

    /// Fullscreen
    #[serde(default)]
    pub fullscreen: bool,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            auto_optimize: true,
            enable_booster: true,
            window_width: None,
            window_height: None,
            fullscreen: false,
        }
    }
}

/// P2P configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Enable P2P
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enable hole punching
    #[serde(default = "default_true")]
    pub enable_hole_punching: bool,

    /// Node display name
    #[serde(default)]
    pub node_name: Option<String>,

    /// Max connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Discovery interval (seconds)
    #[serde(default = "default_discovery_interval")]
    pub discovery_interval: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enable_hole_punching: true,
            node_name: None,
            max_connections: 50,
            discovery_interval: 10,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Theme
    #[serde(default)]
    pub theme: Theme,

    /// Language
    #[serde(default = "default_language")]
    pub language: String,

    /// Accent color
    #[serde(default)]
    pub accent_color: Option<String>,

    /// Background image
    #[serde(default)]
    pub background: Option<PathBuf>,

    /// Background music
    #[serde(default)]
    pub bgm: Option<PathBuf>,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: "zh-CN".to_string(),
            accent_color: None,
            background: None,
            bgm: None,
        }
    }
}

/// Theme setting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Follow system
    #[default]
    System,
    /// Light theme
    Light,
    /// Dark theme
    Dark,
}

// Default value functions
fn default_data_dir() -> PathBuf {
    crate::default_data_dir()
}

fn default_game_dir() -> PathBuf {
    default_data_dir().join(".minecraft")
}

fn default_config_path() -> PathBuf {
    default_data_dir().join("config.yaml")
}

fn default_true() -> bool {
    true
}

fn default_memory() -> u32 {
    4096
}

fn default_min_memory() -> u32 {
    512
}

fn default_concurrent() -> u32 {
    8
}

fn default_chunk_size() -> u64 {
    1024 * 1024
}

fn default_mirrors() -> Vec<String> {
    vec![
        "https://bmclapi2.bangbang93.com".to_string(),
        "https://download.mcbbs.net".to_string(),
    ]
}

fn default_max_connections() -> u32 {
    50
}

fn default_discovery_interval() -> u64 {
    10
}

fn default_language() -> String {
    "zh-CN".to_string()
}
