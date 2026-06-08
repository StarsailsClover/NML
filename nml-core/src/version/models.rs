//! Version models for Minecraft

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Version manifest (from Mojang)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Latest versions
    pub latest: LatestVersions,
    /// All versions
    pub versions: Vec<VersionEntry>,
}

/// Latest versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    /// Latest release
    pub release: String,
    /// Latest snapshot
    pub snapshot: String,
}

/// Version entry in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Version ID
    pub id: String,
    /// Version type
    #[serde(rename = "type")]
    pub version_type: VersionType,
    /// Manifest URL
    pub url: String,
    /// Release time
    #[serde(with = "chrono::serde::ts_rfc3339")]
    pub time: DateTime<Utc>,
    /// Release time (again)
    #[serde(with = "chrono::serde::ts_rfc3339", rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// SHA1 of version JSON
    pub sha1: String,
}

/// Version type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    /// Release version
    Release,
    /// Snapshot version
    Snapshot,
    /// Old alpha
    #[serde(rename = "old_alpha")]
    OldAlpha,
    /// Old beta
    #[serde(rename = "old_beta")]
    OldBeta,
}

/// Full version info (from version JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Arguments (new format)
    pub arguments: Option<Arguments>,
    /// Asset index
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    /// Assets version
    pub assets: String,
    /// Compliance level
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<i32>,
    /// Downloads
    pub downloads: Option<Downloads>,
    /// Version ID
    pub id: String,
    /// Java version requirement
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    /// Libraries
    pub libraries: Vec<Library>,
    /// Logging config
    pub logging: Option<Logging>,
    /// Main class
    #[serde(rename = "mainClass")]
    pub main_class: String,
    /// Minimum launcher version
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: i32,
    /// Minecraft arguments (old format)
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    /// Release time
    #[serde(with = "chrono::serde::ts_rfc3339")]
    pub time: DateTime<Utc>,
    /// Release time
    #[serde(with = "chrono::serde::ts_rfc3339", rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    /// Version type
    #[serde(rename = "type")]
    pub version_type: String,
}

/// Arguments structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    /// Game arguments
    pub game: Vec<ArgumentValue>,
    /// JVM arguments
    pub jvm: Vec<ArgumentValue>,
}

/// Argument value (can be string or complex)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    /// Simple string
    Simple(String),
    /// Complex with rules
    Complex(ArgumentComplex),
}

/// Complex argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentComplex {
    /// Rules
    pub rules: Vec<Rule>,
    /// Value
    pub value: ArgumentValueInner,
}

/// Inner argument value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValueInner {
    /// Single value
    Single(String),
    /// Multiple values
    Multiple(Vec<String>),
}

/// Rule for arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Action
    pub action: RuleAction,
    /// OS condition
    pub os: Option<OsCondition>,
    /// Features condition
    pub features: Option<HashMap<String, bool>>,
}

/// Rule action
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

/// OS condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsCondition {
    /// OS name
    pub name: Option<String>,
    /// OS version
    pub version: Option<String>,
    /// Architecture
    pub arch: Option<String>,
}

/// Asset index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    /// ID
    pub id: String,
    /// SHA1
    pub sha1: String,
    /// Size
    pub size: i64,
    /// Total size
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    /// URL
    pub url: String,
}

/// Downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    /// Client jar
    pub client: Option<DownloadInfo>,
    /// Client mappings
    #[serde(rename = "client_mappings")]
    pub client_mappings: Option<DownloadInfo>,
    /// Server jar
    pub server: Option<DownloadInfo>,
    /// Server mappings
    #[serde(rename = "server_mappings")]
    pub server_mappings: Option<DownloadInfo>,
    /// Windows server (special)
    #[serde(rename = "windows_server")]
    pub windows_server: Option<DownloadInfo>,
}

/// Download info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// SHA1
    pub sha1: String,
    /// Size
    pub size: i64,
    /// URL
    pub url: String,
}

/// Java version requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    /// Component
    pub component: String,
    /// Major version
    #[serde(rename = "majorVersion")]
    pub major_version: i32,
}

/// Library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// Downloads
    pub downloads: LibraryDownloads,
    /// Library name (Maven coordinate)
    pub name: String,
    /// Native classifiers
    pub natives: Option<HashMap<String, String>>,
    /// Extract rules
    pub extract: Option<ExtractRules>,
    /// Rules
    pub rules: Option<Vec<Rule>>,
}

/// Library downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    /// Artifact
    pub artifact: Option<DownloadInfo>,
    /// Classifiers for natives
    pub classifiers: Option<HashMap<String, DownloadInfo>>,
}

/// Extract rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRules {
    /// Exclude patterns
    pub exclude: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    /// Client logging
    pub client: LoggingClient,
}

/// Logging client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    /// Argument
    pub argument: String,
    /// File
    pub file: DownloadInfo,
    /// Type
    #[serde(rename = "type")]
    pub log_type: String,
}

/// Installed version info
#[derive(Debug, Clone)]
pub struct InstalledVersion {
    /// Version ID
    pub id: String,
    /// Full version info
    pub info: VersionInfo,
    /// Installation time
    pub installed_at: std::time::SystemTime,
}

impl VersionType {
    /// Check if this is a release version
    pub fn is_release(&self) -> bool {
        matches!(self, VersionType::Release)
    }

    /// Check if this is a snapshot
    pub fn is_snapshot(&self) -> bool {
        matches!(self, VersionType::Snapshot)
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            VersionType::Release => "正式版",
            VersionType::Snapshot => "快照",
            VersionType::OldAlpha => "远古Alpha",
            VersionType::OldBeta => "远古Beta",
        }
    }
}
