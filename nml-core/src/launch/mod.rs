//! Launch engine for Minecraft

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command as TokioCommand};
use tokio::sync::Mutex;

use crate::error::{NMLError, Result};
use crate::version::VersionInfo;

pub mod arguments;
pub mod java;

use arguments::ArgumentBuilder;
use java::JavaDetector;

/// Minecraft process handle
#[derive(Debug)]
pub struct MinecraftProcess {
    /// Process ID
    pub pid: u32,
    /// Child process
    pub child: Arc<Mutex<Child>>,
    /// Version ID
    pub version_id: String,
    /// Start time
    pub start_time: std::time::Instant,
}

impl MinecraftProcess {
    /// Check if process is still running
    pub async fn is_running(&self) -> bool {
        let mut child = self.child.lock().await;
        matches!(child.try_wait(), Ok(None))
    }

    /// Kill the process
    pub async fn kill(&self) -> Result<()> {
        let mut child = self.child.lock().await;
        child.kill().await?;
        Ok(())
    }

    /// Wait for process to exit
    pub async fn wait(&self) -> Result<std::process::ExitStatus> {
        let mut child = self.child.lock().await;
        let status = child.wait().await?;
        Ok(status)
    }
}

/// Launch configuration
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    /// Version ID to launch
    pub version_id: String,
    /// Player name (for offline mode)
    pub player_name: String,
    /// Access token (for online mode)
    pub access_token: Option<String>,
    /// User UUID
    pub uuid: Option<String>,
    /// User type (mojang, microsoft, legacy)
    pub user_type: String,
    /// Max memory in MB
    pub max_memory: u32,
    /// Min memory in MB
    pub min_memory: u32,
    /// Custom JVM arguments
    pub jvm_args: Vec<String>,
    /// Custom game arguments
    pub game_args: Vec<String>,
    /// Window width
    pub window_width: Option<u32>,
    /// Window height
    pub window_height: Option<u32>,
    /// Fullscreen
    pub fullscreen: bool,
    /// Server to auto-connect
    pub server: Option<String>,
    /// Game directory
    pub game_dir: PathBuf,
    /// Java path (auto-detected if None)
    pub java_path: Option<PathBuf>,
    /// Enable optimization (MCJEBooster)
    pub enable_optimization: bool,
}

impl LaunchConfig {
    /// Create default config for version
    pub fn for_version(version_id: &str, game_dir: PathBuf) -> Self {
        Self {
            version_id: version_id.to_string(),
            player_name: "Player".to_string(),
            access_token: None,
            uuid: None,
            user_type: "mojang".to_string(),
            max_memory: 4096,
            min_memory: 512,
            jvm_args: Vec::new(),
            game_args: Vec::new(),
            window_width: None,
            window_height: None,
            fullscreen: false,
            server: None,
            game_dir,
            java_path: None,
            enable_optimization: true,
        }
    }
}

/// Launch engine
pub struct LaunchEngine {
    data_dir: PathBuf,
    java_detector: JavaDetector,
}

impl LaunchEngine {
    /// Create new launch engine
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            java_detector: JavaDetector::new(),
        }
    }

    /// Launch Minecraft
    pub async fn launch(&self, config: &LaunchConfig) -> Result<MinecraftProcess> {
        // 1. Get version info
        let version_info = self.load_version_info(&config.version_id).await?;

        // 2. Detect Java
        let java_path = if let Some(path) = &config.java_path {
            path.clone()
        } else {
            self.detect_java(&version_info).await?
        };

        // 3. Build JVM arguments
        let jvm_args = self.build_jvm_args(config, &version_info)?;

        // 4. Build classpath
        let classpath = self.build_classpath(&version_info).await?;

        // 5. Extract natives
        self.extract_natives(&version_info).await?;

        // 6. Build game arguments
        let game_args = self.build_game_args(config, &version_info)?;

        // 7. Create command
        let mut command = TokioCommand::new(&java_path);
        command
            .current_dir(&config.game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add JVM args
        for arg in jvm_args {
            command.arg(arg);
        }

        // Add classpath
        command.arg("-cp").arg(classpath);

        // Add main class
        command.arg(&version_info.main_class);

        // Add game args
        for arg in game_args {
            command.arg(arg);
        }

        // 7. Spawn process
        let child = command.spawn()?;
        let pid = child.id()
            .ok_or_else(|| NMLError::LaunchFailed("Failed to get Minecraft process ID".to_string()))?;

        let process = MinecraftProcess {
            pid,
            child: Arc::new(Mutex::new(child)),
            version_id: config.version_id.clone(),
            start_time: std::time::Instant::now(),
        };

        // 8. Apply optimization if enabled
        if config.enable_optimization {
            self.apply_optimization(&process).await?;
        }

        Ok(process)
    }

    /// Load version info from JSON
    async fn load_version_info(&self, version_id: &str) -> Result<VersionInfo> {
        let version_json = self
            .data_dir
            .join("versions")
            .join(version_id)
            .join(format!("{}.json", version_id));

        if !version_json.exists() {
            return Err(NMLError::VersionNotFound(version_id.to_string()));
        }

        let content = tokio::fs::read_to_string(&version_json).await?;
        let info: VersionInfo = serde_json::from_str(&content)?;
        Ok(info)
    }

    /// Detect appropriate Java version
    async fn detect_java(&self, version_info: &VersionInfo) -> Result<PathBuf> {
        let required_version = version_info
            .java_version
            .as_ref()
            .map(|jv| jv.major_version)
            .unwrap_or(8);

        let java_path = self.java_detector.find_java(required_version).await?;
        Ok(java_path)
    }

    /// Build JVM arguments
    fn build_jvm_args(&self, config: &LaunchConfig, version_info: &VersionInfo) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Memory settings
        args.push(format!("-Xms{}M", config.min_memory));
        args.push(format!("-Xmx{}M", config.max_memory));

        // G1GC optimization
        args.push("-XX:+UseG1GC".to_string());
        args.push("-XX:+ParallelRefProcEnabled".to_string());
        args.push("-XX:MaxGCPauseMillis=200".to_string());
        args.push("-XX:+UnlockExperimentalVMOptions".to_string());
        args.push("-XX:+DisableExplicitGC".to_string());
        args.push("-XX:+AlwaysPreTouch".to_string());
        args.push("-XX:G1NewSizePercent=20".to_string());
        args.push("-XX:G1MaxNewSizePercent=40".to_string());
        args.push("-XX:G1HeapRegionSize=16M".to_string());
        args.push("-XX:G1ReservePercent=15".to_string());
        args.push("-XX:G1HeapWastePercent=10".to_string());

        // System properties
        args.push("-Dfile.encoding=UTF-8".to_string());
        args.push("-Djava.awt.headless=false".to_string());

        // Version-specific JVM args (new format)
        if let Some(arguments) = &version_info.arguments {
            for arg in &arguments.jvm {
                match arg {
                    crate::version::ArgumentValue::Simple(s) => args.push(s.clone()),
                    _ => {}
                }
            }
        }

        // Custom JVM args
        for arg in &config.jvm_args {
            args.push(arg.clone());
        }

        // Native library path
        let natives_path = self
            .data_dir
            .join("versions")
            .join(&config.version_id)
            .join(format!("{}-natives", config.version_id));
        args.push(format!("-Djava.library.path={}", natives_path.display()));

        Ok(args)
    }

    /// Build classpath
    async fn build_classpath(&self, version_info: &VersionInfo) -> Result<String> {
        let mut paths = Vec::new();

        // Add client JAR
        let client_jar = self
            .data_dir
            .join("versions")
            .join(&version_info.id)
            .join(format!("{}.jar", version_info.id));
        paths.push(client_jar);

        // Add libraries
        for library in &version_info.libraries {
            // Skip libraries that don't apply to this OS
            if let Some(rules) = &library.rules {
                if !self.check_rules(rules) {
                    continue;
                }
            }

            // Add library JAR
            if let Some(artifact) = &library.downloads.artifact {
                let lib_path = self.get_library_path(&library.name);
                if lib_path.exists() {
                    paths.push(lib_path);
                }
            }

            // Add native library
            if let Some(natives) = &library.natives {
                if let Some(classifiers) = &library.downloads.classifiers {
                    let os_key = self.get_os_key();
                    if let Some(native_classifier) = natives.get(&os_key) {
                        if let Some(native) = classifiers.get(native_classifier) {
                            let native_path = self.get_native_path(&library.name, native_classifier);
                            if native_path.exists() {
                                paths.push(native_path);
                            }
                        }
                    }
                }
            }
        }

        // Convert to classpath string
        let separator = if cfg!(windows) { ";" } else { ":" };
        let classpath = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(separator);

        Ok(classpath)
    }

    /// Build game arguments
    fn build_game_args(&self, config: &LaunchConfig, version_info: &VersionInfo) -> Result<Vec<String>> {
        let mut args = Vec::new();

        let uuid = config.uuid.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let token = config.access_token.clone().unwrap_or_else(|| "0".to_string());

        // Version-specific game args (new format)
        if let Some(arguments) = &version_info.arguments {
            for arg in &arguments.game {
                match arg {
                    crate::version::ArgumentValue::Simple(s) => {
                        let processed = self.process_argument(s, config, &uuid, &token);
                        args.push(processed);
                    }
                    _ => {}
                }
            }
        }
        // Old format
        else if let Some(mc_args) = &version_info.minecraft_arguments {
            let processed = self.process_argument(mc_args, config, &uuid, &token);
            args.extend(split_legacy_arguments(&processed)?);
        }

        // Window size
        if let Some(width) = config.window_width {
            args.push("--width".to_string());
            args.push(width.to_string());
        }
        if let Some(height) = config.window_height {
            args.push("--height".to_string());
            args.push(height.to_string());
        }

        // Fullscreen
        if config.fullscreen {
            args.push("--fullscreen".to_string());
            args.push("true".to_string());
        }

        // Server auto-connect
        if let Some(server) = &config.server {
            args.push("--server".to_string());
            args.push(server.clone());
        }

        // Custom game args
        for arg in &config.game_args {
            args.push(arg.clone());
        }

        Ok(args)
    }

    /// Process argument placeholders
    fn process_argument(&self, arg: &str, config: &LaunchConfig, uuid: &str, token: &str) -> String {
        let game_dir = config.game_dir.to_string_lossy().to_string();
        let assets_dir = self.data_dir.join("assets");
        let natives_dir = self
            .data_dir
            .join("versions")
            .join(&config.version_id)
            .join(format!("{}-natives", config.version_id));

        arg.replace("${auth_player_name}", &config.player_name)
            .replace("${auth_uuid}", uuid)
            .replace("${auth_access_token}", token)
            .replace("${auth_session}", token)
            .replace("${user_type}", &config.user_type)
            .replace("${user_properties}", "{}")
            .replace("${version_name}", &config.version_id)
            .replace("${version_type}", "NML")
            .replace("${game_directory}", &game_dir)
            .replace("${assets_root}", &assets_dir.to_string_lossy())
            .replace("${assets_index_name}", &config.version_id)
            .replace("${natives_directory}", &natives_dir.to_string_lossy())
            .replace("${launcher_name}", "nml")
            .replace("${launcher_version}", "1.0.0")
    }

    /// Apply optimization (MCJEBooster)
    async fn apply_optimization(&self, process: &MinecraftProcess) -> Result<()> {
        // TODO: Integrate MCJEBooster
        // For now, just log
        tracing::info!("Optimization would be applied to process {}", process.pid);
        Ok(())
    }

    /// Extract native libraries from JARs to the natives directory
    async fn extract_natives(&self, version_info: &VersionInfo) -> Result<()> {
        let natives_dir = self
            .data_dir
            .join("versions")
            .join(&version_info.id)
            .join(format!("{}-natives", version_info.id));

        // Skip if already extracted
        if natives_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&natives_dir) {
                if entries.count() > 0 {
                    return Ok(());
                }
            }
        }

        tokio::fs::create_dir_all(&natives_dir).await?;
        let os_key = self.get_os_key();

        for library in &version_info.libraries {
            // Check platform rules
            if let Some(rules) = &library.rules {
                if !self.check_rules(rules) {
                    continue;
                }
            }

            if let Some(natives) = &library.natives {
                if let Some(native_classifier) = natives.get(&os_key) {
                    if let Some(classifiers) = &library.downloads.classifiers {
                        if let Some(_) = classifiers.get(native_classifier) {
                            let native_jar_path = self.get_native_path(&library.name, native_classifier);
                            if native_jar_path.exists() {
                                let file = std::fs::File::open(&native_jar_path)?;
                                let mut archive = zip::ZipArchive::new(file)
                                    .map_err(|e| NMLError::other(format!("Failed to open native JAR: {}", e)))?;

                                for i in 0..archive.len() {
                                    let mut entry = archive.by_index(i)
                                        .map_err(|e| NMLError::other(format!("Failed to read zip entry: {}", e)))?;

                                    let name = entry.name().to_string();
                                    if entry.is_dir() || name.starts_with("META-INF/") {
                                        continue;
                                    }

                                    // Only extract native library files
                                    let is_native = name.ends_with(".dll")
                                        || name.ends_with(".so")
                                        || name.ends_with(".dylib")
                                        || name.ends_with(".jnilib");
                                    if !is_native {
                                        continue;
                                    }

                                    let Some(file_name) = std::path::Path::new(&name).file_name() else {
                                        continue;
                                    };
                                    let dest_path = natives_dir.join(file_name);

                                    let mut buf = Vec::new();
                                    entry.read_to_end(&mut buf)?;
                                    std::fs::write(&dest_path, &buf)?;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if rules apply
    fn check_rules(&self, rules: &[crate::version::Rule]) -> bool {
        for rule in rules {
            let allowed = if rule.action == crate::version::RuleAction::Allow {
                self.check_rule_os(&rule.os)
            } else {
                !self.check_rule_os(&rule.os)
            };

            if allowed {
                return true;
            }
        }
        false
    }

    fn check_rule_os(&self, os: &Option<crate::version::OsCondition>) -> bool {
        if let Some(os) = os {
            if let Some(name) = &os.name {
                if name != self.get_os_name() {
                    return false;
                }
            }
        }
        true
    }

    /// Get OS name for rules
    fn get_os_name(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        }
    }

    /// Get OS key for natives
    fn get_os_key(&self) -> String {
        if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "osx-arm64"
            } else {
                "osx"
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") {
                "linux-arm64"
            } else {
                "linux"
            }
        } else {
            "unknown"
        }
        .to_string()
    }

    /// Get library path from Maven coordinate
    fn get_library_path(&self, name: &str) -> PathBuf {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 {
            return PathBuf::new();
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let classifier = parts.get(3).map(|c| format!("-{}", c)).unwrap_or_default();

        self.data_dir
            .join("libraries")
            .join(group)
            .join(artifact)
            .join(version)
            .join(format!("{}-{}{}.jar", artifact, version, classifier))
    }

    /// Get native library path
    fn get_native_path(&self, name: &str, classifier: &str) -> PathBuf {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() < 3 {
            return PathBuf::new();
        }

        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];

        self.data_dir
            .join("libraries")
            .join(group)
            .join(artifact)
            .join(version)
            .join(format!("{}-{}-{}.jar", artifact, version, classifier))
    }
}

fn split_legacy_arguments(input: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push(ch);
                }
            }
            '\'' | '"' => {
                if quote == Some(ch) {
                    quote = None;
                } else if quote.is_none() {
                    quote = Some(ch);
                } else {
                    current.push(ch);
                }
            }
            c if c.is_whitespace() && quote.is_none() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err(NMLError::LaunchFailed("Unclosed quote in legacy Minecraft arguments".to_string()));
    }

    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}
