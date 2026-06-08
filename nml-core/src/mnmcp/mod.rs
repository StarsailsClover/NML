//! MnMCP integration for Mini World cross-play
//!
//! Bridges Minecraft Java Edition with Mini World

use std::process::Child;
use std::path::PathBuf;

use crate::error::{NMLError, Result};
use crate::p2p::{P2PManager, WorldInfo};
use crate::launch::MinecraftProcess;

pub mod bridge;
pub mod protocol;
pub mod mapping;

use bridge::MnMCPBridge;
use protocol::MnMCPProtocol;
use mapping::BlockMapping;

/// MnMCP integration
pub struct MnMCPIntegration {
    bridge: MnMCPBridge,
    protocol: MnMCPProtocol,
    mapping: BlockMapping,
    python_process: Option<Child>,
}

/// Mini World server info
#[derive(Debug, Clone)]
pub struct MiniWorldServer {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub player_count: u32,
    pub max_players: u32,
}

impl MnMCPIntegration {
    /// Create new integration
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        // Start Python bridge
        let bridge = MnMCPBridge::start(&data_dir).await?;
        
        Ok(Self {
            bridge,
            protocol: MnMCPProtocol::new(),
            mapping: BlockMapping::load().await?,
            python_process: None,
        })
    }
    
    /// Enable cross-play
    pub async fn enable_crossplay(&mut self, mc_process: &MinecraftProcess) -> Result<()> {
        tracing::info!("Enabling Mini World cross-play");
        
        // 1. Start MnMCP proxy
        self.start_proxy().await?;
        
        // 2. Connect to MC process
        self.bridge.connect_to_minecraft(mc_process.pid).await?;
        
        // 3. Start protocol bridge
        self.protocol.start().await?;
        
        tracing::info!("Cross-play enabled");
        Ok(())
    }
    
    /// Connect to Mini World server
    pub async fn connect_to_mini(&self, server: &MiniWorldServer) -> Result<()> {
        tracing::info!("Connecting to Mini World server: {}", server.name);
        
        // Use MnMCP protocol to connect
        self.protocol.connect(&server.ip, server.port).await?;
        
        tracing::info!("Connected to Mini World");
        Ok(())
    }
    
    /// List Mini World servers
    pub async fn list_servers(&self) -> Result<Vec<MiniWorldServer>> {
        // Query MnMCP for available servers
        self.protocol.discover_servers().await
    }
    
    /// Translate block from MC to Mini World
    pub fn translate_block_mc_to_mini(&self, mc_block_id: u32) -> u32 {
        self.mapping.mc_to_mini(mc_block_id)
    }
    
    /// Translate block from Mini World to MC
    pub fn translate_block_mini_to_mc(&self, mini_block_id: u32) -> u32 {
        self.mapping.mini_to_mc(mini_block_id)
    }
    
    /// Start proxy
    async fn start_proxy(&mut self) -> Result<()> {
        // Start MnMCP proxy process
        tracing::info!("Starting MnMCP proxy");
        
        // In real implementation, this would start the Python process
        
        Ok(())
    }
    
    /// Shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down MnMCP integration");
        
        if let Some(mut process) = self.python_process.take() {
            let _ = process.kill();
        }
        
        self.protocol.stop().await?;
        
        Ok(())
    }
}
