//! Minecraft server management
//!
//! Local server hosting with full management

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Child;

use crate::error::{NMLError, Result};
use crate::download::DownloadEngine;

pub mod config;
pub mod plugins;
pub mod players;
pub mod console;

/// Server manager
pub struct ServerManager {
    servers: HashMap<String, MinecraftServer>,
    download_engine: DownloadEngine,
    data_dir: PathBuf,
}

/// Minecraft server instance
pub struct MinecraftServer {
    pub id: String,
    pub name: String,
    pub version: String,
    pub modloader: ServerModLoader,
    pub port: u16,
    pub process: Option<Child>,
    pub config: ServerConfig,
    pub players: Vec<ServerPlayer>,
    pub status: ServerStatus,
}

/// Server mod loader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerModLoader {
    Vanilla,
    Forge,
    Fabric,
    NeoForge,
}

/// Server configuration
#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    pub server_properties: HashMap<String, String>,
    pub ops: Vec<String>,
    pub whitelist: Vec<String>,
    pub banned_players: Vec<String>,
    pub max_players: u32,
    pub gamemode: String,
    pub difficulty: String,
    pub motd: String,
}

/// Server player
#[derive(Debug, Clone)]
pub struct ServerPlayer {
    pub name: String,
    pub uuid: String,
    pub is_op: bool,
    pub is_online: bool,
}

/// Server status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl ServerManager {
    /// Create new manager
    pub fn new(data_dir: PathBuf, download_engine: DownloadEngine) -> Self {
        Self {
            servers: HashMap::new(),
            download_engine,
            data_dir: data_dir.join("servers"),
        }
    }
    
    /// Create new server
    pub async fn create_server(&mut self, name: &str, version: &str, modloader: ServerModLoader) -> Result<MinecraftServer> {
        tracing::info!("Creating server: {} ({})", name, version);
        
        let server_id = format!("server_{}", uuid::Uuid::new_v4().to_simple());
        let server_dir = self.data_dir.join(&server_id);
        
        std::fs::create_dir_all(&server_dir)?;
        
        // Download server jar
        self.download_server_jar(version, modloader, &server_dir).await?;
        
        // Create default config
        let config = self.create_default_config().await?;
        self.save_server_properties(&server_dir, &config).await?;
        
        let server = MinecraftServer {
            id: server_id.clone(),
            name: name.to_string(),
            version: version.to_string(),
            modloader,
            port: 25565,
            process: None,
            config,
            players: vec![],
            status: ServerStatus::Stopped,
        };
        
        self.servers.insert(server_id, server.clone());
        
        Ok(server)
    }
    
    /// Start server
    pub async fn start_server(&mut self, server_id: &str) -> Result<()> {
        let server = self.servers.get_mut(server_id)
            .ok_or_else(|| NMLError::Other("Server not found".to_string()))?;
        
        if server.status == ServerStatus::Running {
            return Ok(());
        }
        
        tracing::info!("Starting server: {}", server.name);
        server.status = ServerStatus::Starting;
        
        let server_dir = self.data_dir.join(&server.id);
        let jar_name = self.get_server_jar_name(&server.version, server.modloader);
        
        let mut cmd = std::process::Command::new("java");
        cmd.current_dir(&server_dir)
            .arg("-Xmx2G")
            .arg("-jar")
            .arg(&jar_name)
            .arg("nogui");
        
        let child = cmd.spawn()?;
        server.process = Some(child);
        server.status = ServerStatus::Running;
        
        tracing::info!("Server started on port {}", server.port);
        Ok(())
    }
    
    /// Stop server
    pub async fn stop_server(&mut self, server_id: &str) -> Result<()> {
        let server = self.servers.get_mut(server_id)
            .ok_or_else(|| NMLError::Other("Server not found".to_string()))?;
        
        if server.status != ServerStatus::Running {
            return Ok(());
        }
        
        tracing::info!("Stopping server: {}", server.name);
        server.status = ServerStatus::Stopping;
        
        // Send stop command
        if let Some(ref mut process) = server.process {
            // Send /stop command via RCon or input
            // For now, just kill the process
            let _ = process.kill();
        }
        
        server.process = None;
        server.status = ServerStatus::Stopped;
        
        tracing::info!("Server stopped");
        Ok(())
    }
    
    /// Get server console output
    pub async fn get_console(&self, server_id: &str, lines: usize) -> Result<Vec<String>> {
        let server = self.servers.get(server_id)
            .ok_or_else(|| NMLError::Other("Server not found".to_string()))?;
        
        let log_file = self.data_dir.join(&server.id).join("logs").join("latest.log");
        
        if !log_file.exists() {
            return Ok(vec![]);
        }
        
        let content = tokio::fs::read_to_string(&log_file).await?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        let start = if lines.len() > lines { lines.len() - lines } else { 0 };
        Ok(lines[start..].to_vec())
    }
    
    /// Execute command
    pub async fn execute_command(&self, server_id: &str, command: &str) -> Result<String> {
        // Send command via RCon
        tracing::info!("Executing command on {}: {}", server_id, command);
        
        // TODO: Implement RCon communication
        
        Ok("Command executed".to_string())
    }
    
    /// Add OP
    pub async fn add_op(&mut self, server_id: &str, player: &str) -> Result<()> {
        let server = self.servers.get_mut(server_id)
            .ok_or_else(|| NMLError::Other("Server not found".to_string()))?;
        
        server.config.ops.push(player.to_string());
        
        // Execute /op command if running
        if server.status == ServerStatus::Running {
            self.execute_command(server_id, &format!("/op {}", player)).await?;
        }
        
        Ok(())
    }
    
    /// Ban player
    pub async fn ban_player(&mut self, server_id: &str, player: &str) -> Result<()> {
        let server = self.servers.get_mut(server_id)
            .ok_or_else(|| NMLError::Other("Server not found".to_string()))?;
        
        server.config.banned_players.push(player.to_string());
        
        if server.status == ServerStatus::Running {
            self.execute_command(server_id, &format!("/ban {}", player)).await?;
        }
        
        Ok(())
    }
    
    /// Get all servers
    pub fn get_servers(&self) -> Vec<&MinecraftServer> {
        self.servers.values().collect()
    }
    
    /// Get server by ID
    pub fn get_server(&self, server_id: &str) -> Option<&MinecraftServer> {
        self.servers.get(server_id)
    }
    
    /// Download server jar
    async fn download_server_jar(&self, version: &str, modloader: ServerModLoader, dir: &PathBuf) -> Result<()> {
        let url = match modloader {
            ServerModLoader::Vanilla => {
                format!("https://piston-data.mojang.com/v1/objects/{}/server.jar", 
                    self.get_server_hash(version)?)
            }
            _ => {
                // Download from modloader website
                return Err(NMLError::Other("Modloader servers not yet supported".to_string()));
            }
        };
        
        let dest = dir.join(self.get_server_jar_name(version, modloader));
        
        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(NMLError::DownloadFailed(format!(
                "Failed to download server jar for {}: HTTP {}",
                version,
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        tokio::fs::write(&dest, bytes).await?;
        
        Ok(())
    }
    
    /// Create default config
    async fn create_default_config(&self) -> Result<ServerConfig> {
        let mut props = HashMap::new();
        props.insert("server-port".to_string(), "25565".to_string());
        props.insert("max-players".to_string(), "20".to_string());
        props.insert("gamemode".to_string(), "survival".to_string());
        props.insert("difficulty".to_string(), "normal".to_string());
        props.insert("motd".to_string(), "Minecraft Server".to_string());
        props.insert("enable-rcon".to_string(), "true".to_string());
        props.insert("rcon.port".to_string(), "25575".to_string());
        props.insert("rcon.password".to_string(), self.generate_rcon_password());
        
        Ok(ServerConfig {
            server_properties: props,
            ops: vec![],
            whitelist: vec![],
            banned_players: vec![],
            max_players: 20,
            gamemode: "survival".to_string(),
            difficulty: "normal".to_string(),
            motd: "Minecraft Server".to_string(),
        })
    }
    
    /// Save server.properties
    async fn save_server_properties(&self, dir: &PathBuf, config: &ServerConfig) -> Result<()> {
        let props_file = dir.join("server.properties");
        
        let mut content = String::from("# Minecraft server properties\n");
        for (key, value) in &config.server_properties {
            content.push_str(&format!("{}={}\n", key, value));
        }
        
        tokio::fs::write(&props_file, content).await?;
        
        Ok(())
    }
    
    /// Get server jar name
    fn get_server_jar_name(&self, _version: &str, modloader: ServerModLoader) -> String {
        match modloader {
            ServerModLoader::Vanilla => "server.jar".to_string(),
            ServerModLoader::Forge => "forge-server.jar".to_string(),
            ServerModLoader::Fabric => "fabric-server.jar".to_string(),
            ServerModLoader::NeoForge => "neoforge-server.jar".to_string(),
        }
    }
    
    /// Get server hash (simplified)
    fn get_server_hash(&self, version: &str) -> Result<String> {
        // In real implementation, this would lookup from version manifest
        Ok(version.to_string())
    }
    
    /// Generate RCon password
    fn generate_rcon_password(&self) -> String {
        use rand::Rng;
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..16)
            .map(|_| charset[rng.gen_range(0..charset.len())] as char)
            .collect()
    }
}
