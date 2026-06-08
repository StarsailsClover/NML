//! minecraftBC integration for P2P multiplayer
//!
//! Integrates with StarsailsClover's minecraftBC for LAN multiplayer

use std::net::SocketAddr;
use std::collections::HashMap;

use crate::error::{NMLError, Result};
use crate::launch::MinecraftProcess;
use crate::p2p::{P2PManager, WorldInfo};

pub mod network;
pub mod lan_injector;

use network::P2PNetworkManager;
use lan_injector::LANInjector;

/// minecraftBC integration
pub struct MinecraftBCIntegration {
    network: P2PNetworkManager,
    lan_injector: LANInjector,
    hosted_worlds: HashMap<String, HostedWorld>,
}

pub struct HostedWorld {
    pub world_id: String,
    pub local_port: u16,
    pub proxy_port: u16,
}

impl MinecraftBCIntegration {
    /// Create new integration
    pub async fn new() -> Result<Self> {
        Ok(Self {
            network: P2PNetworkManager::new().await?,
            lan_injector: LANInjector::new().await?,
            hosted_worlds: HashMap::new(),
        })
    }
    
    /// Host a world for P2P multiplayer
    pub async fn host_world(&mut self, process: &MinecraftProcess, world_info: WorldInfo, local_port: u16) -> Result<String> {
        tracing::info!("Hosting world {} via minecraftBC", world_info.world_name);
        
        // 1. Start local server (already running at local_port)
        // 2. Create P2P proxy
        let proxy_port = self.network.create_proxy(local_port).await?;
        
        // 3. Announce to P2P network
        let world_id = self.network.announce_world(world_info).await?;
        
        // 4. Start LAN injection
        self.lan_injector.inject_world(world_id, &world_info, proxy_port).await?;
        
        // 5. Store hosted world
        self.hosted_worlds.insert(world_id.clone(), HostedWorld {
            world_id: world_id.clone(),
            local_port,
            proxy_port,
        });
        
        tracing::info!("World hosted: {} on port {}", world_id, proxy_port);
        Ok(world_id)
    }
    
    /// Join a P2P world
    pub async fn join_world(&mut self, process: &MinecraftProcess, world_id: &str) -> Result<()> {
        tracing::info!("Joining world {} via minecraftBC", world_id);
        
        // 1. Get world info from network
        let world_info = self.network.get_world_info(world_id).await?;
        
        // 2. Connect to host
        let proxy_port = self.network.connect_to_world(world_id).await?;
        
        // 3. Inject to Minecraft LAN list
        self.lan_injector.inject_server(&world_info, proxy_port).await?;
        
        tracing::info!("Connected to world {} on port {}", world_id, proxy_port);
        Ok(())
    }
    
    /// Discover available worlds
    pub async fn discover_worlds(&self) -> Result<Vec<WorldInfo>> {
        self.network.discover_worlds().await
    }
    
    /// Leave world
    pub async fn leave_world(&mut self, world_id: &str) -> Result<()> {
        self.network.disconnect_from_world(world_id).await?;
        self.lan_injector.remove_injected_server(world_id).await?;
        Ok(())
    }
    
    /// Stop hosting
    pub async fn stop_hosting(&mut self, world_id: &str) -> Result<()> {
        if let Some(world) = self.hosted_worlds.remove(world_id) {
            self.network.stop_proxy(world_id).await?;
            self.lan_injector.remove_injected_world(world_id).await?;
            self.network.unannounce_world(world_id).await?;
        }
        Ok(())
    }
}
