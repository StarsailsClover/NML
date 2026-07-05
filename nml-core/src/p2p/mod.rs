//! P2P networking for Minecraft LAN multiplayer

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, RwLock};

use crate::error::{NMLError, Result};

pub mod discovery;
pub mod lan_injector;
pub mod game_proxy;

use discovery::WorldDiscovery;
use lan_injector::LANInjector;
use game_proxy::GameProxy;

/// P2P node configuration
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Node ID
    pub node_id: String,
    /// Display name
    pub display_name: String,
    /// Listen port (0 for auto)
    pub port: u16,
    /// Max connections
    pub max_connections: usize,
    /// Enable NAT hole punching
    pub enable_hole_punching: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            display_name: "NML Node".to_string(),
            port: 0,
            max_connections: 50,
            enable_hole_punching: true,
            enable_encryption: true,
            bootstrap_nodes: vec![],
        }
    }
}

/// World information for P2P
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldInfo {
    /// World ID
    pub world_id: String,
    /// Host node ID
    pub host_node_id: String,
    /// World name
    pub world_name: String,
    /// MOTD
    pub motd: String,
    /// Game version
    pub game_version: String,
    /// Current player count
    pub player_count: u32,
    /// Max players
    pub max_players: u32,
    /// Has password
    pub has_password: bool,
    /// Host latency
    pub latency_ms: u64,
    /// Last update time
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// P2P manager trait
#[async_trait]
pub trait P2PManager: Send + Sync {
    /// Start the P2P node
    async fn start(&self, config: NodeConfig) -> Result<()>;
    /// Stop the P2P node
    async fn stop(&self) -> Result<()>;
    /// Host a world
    async fn host_world(&self, world: WorldInfo, local_port: u16) -> Result<()>;
    /// Stop hosting
    async fn stop_hosting(&self) -> Result<()>;
    /// Discover worlds
    async fn discover_worlds(&self) -> Result<Vec<WorldInfo>>;
    /// Join a world
    async fn join_world(&self, world_id: &str) -> Result<()>;
    /// Get connected nodes
    async fn get_nodes(&self) -> Result<Vec<NodeInfo>>;
    /// Send message to a node
    async fn send_message(&self, node_id: &str, message: P2PMessage) -> Result<()>;
    /// Broadcast message
    async fn broadcast(&self, message: P2PMessage) -> Result<()>;
}

/// Node info
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Node ID
    pub node_id: String,
    /// Display name
    pub display_name: String,
    /// Public endpoint
    pub public_endpoint: Option<SocketAddr>,
    /// Local endpoint
    pub local_endpoint: Option<SocketAddr>,
    /// Latency in ms
    pub latency_ms: u64,
    /// Last seen
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// P2P message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct P2PMessage {
    /// Message type
    pub msg_type: String,
    /// Sender node ID
    pub sender_id: String,
    /// Target node ID (None for broadcast)
    pub target_id: Option<String>,
    /// Payload
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Default P2P manager implementation
pub struct DefaultP2PManager {
    data_dir: PathBuf,
    inner: Arc<RwLock<Option<P2PManagerInner>>>,
}

struct P2PManagerInner {
    config: NodeConfig,
    discovery: WorldDiscovery,
    lan_injector: LANInjector,
    game_proxy: GameProxy,
    event_sender: broadcast::Sender<P2PEvent>,
}

/// P2P events
#[derive(Debug, Clone)]
pub enum P2PEvent {
    /// Node discovered
    NodeDiscovered(NodeInfo),
    /// Node connected
    NodeConnected(NodeInfo),
    /// Node disconnected
    NodeDisconnected(String),
    /// World discovered
    WorldDiscovered(WorldInfo),
    /// World updated
    WorldUpdated(WorldInfo),
    /// World closed
    WorldClosed(String),
    /// Message received
    MessageReceived(P2PMessage),
    /// Error occurred
    Error(String),
}

use std::path::PathBuf;

impl DefaultP2PManager {
    /// Create new P2P manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Subscribe to events
    pub async fn subscribe(&self) -> Result<broadcast::Receiver<P2PEvent>> {
        let inner = self.inner.read().await;
        if let Some(inner) = inner.as_ref() {
            Ok(inner.event_sender.subscribe())
        } else {
            Err(NMLError::P2PError("P2P not started".to_string()))
        }
    }
}

#[async_trait]
impl P2PManager for DefaultP2PManager {
    async fn start(&self, config: NodeConfig) -> Result<()> {
        let (event_sender, _) = broadcast::channel(100);

        let discovery = WorldDiscovery::new(&config.node_id).await?;
        let lan_injector = LANInjector::new(self.data_dir.clone()).await?;
        let game_proxy = GameProxy::new().await?;

        let node_id = config.node_id.clone();
        let inner = P2PManagerInner {
            config,
            discovery,
            lan_injector,
            game_proxy,
            event_sender,
        };

        let mut guard = self.inner.write().await;
        *guard = Some(inner);

        tracing::info!("P2P node {} started", node_id);
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        let mut guard = self.inner.write().await;
        if guard.is_none() {
            return Err(NMLError::P2PError("P2P not started".to_string()));
        }
        
        *guard = None;
        tracing::info!("P2P node stopped");
        Ok(())
    }

    async fn host_world(&self, world: WorldInfo, local_port: u16) -> Result<()> {
        let inner = self.inner.read().await;
        let inner = inner.as_ref()
            .ok_or_else(|| NMLError::P2PError("P2P not started".to_string()))?;

        // Start game proxy
        inner.game_proxy.start_proxy(&world.world_id, local_port).await?;

        // Broadcast world to network
        inner.discovery.announce_world(&world).await?;

        tracing::info!("Hosting world {} on port {}", world.world_name, local_port);
        Ok(())
    }

    async fn stop_hosting(&self) -> Result<()> {
        let inner = self.inner.read().await;
        let inner = inner.as_ref()
            .ok_or_else(|| NMLError::P2PError("P2P not started".to_string()))?;

        inner.game_proxy.stop_proxy().await?;
        inner.discovery.unannounce_world().await?;

        tracing::info!("Stopped hosting");
        Ok(())
    }

    async fn discover_worlds(&self) -> Result<Vec<WorldInfo>> {
        let inner = self.inner.read().await;
        let inner = inner.as_ref()
            .ok_or_else(|| NMLError::P2PError("P2P not started".to_string()))?;

        inner.discovery.discover_worlds().await
    }

    async fn join_world(&self, world_id: &str) -> Result<()> {
        let inner = self.inner.read().await;
        let inner = inner.as_ref()
            .ok_or_else(|| NMLError::P2PError("P2P not started".to_string()))?;

        // Get world info
        let worlds = inner.discovery.discover_worlds().await?;
        let world = worlds.iter()
            .find(|w| w.world_id == world_id)
            .ok_or_else(|| NMLError::P2PError(format!("World {} not found", world_id)))?;

        // Connect to host
        inner.game_proxy.connect_to_world(&world.host_node_id).await?;

        // Inject to Minecraft LAN list
        inner.lan_injector.inject_world(world).await?;

        tracing::info!("Joined world {}", world_id);
        Ok(())
    }

    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        let inner = self.inner.read().await;
        let inner = inner.as_ref()
            .ok_or_else(|| NMLError::P2PError("P2P not started".to_string()))?;

        inner.discovery.get_nodes().await
    }

    async fn send_message(&self, _node_id: &str, _message: P2PMessage) -> Result<()> {
        // TODO: Implement via FastLink
        Ok(())
    }

    async fn broadcast(&self, _message: P2PMessage) -> Result<()> {
        // TODO: Implement via FastLink
        Ok(())
    }
}

