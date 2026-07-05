//! World discovery (stub)

use crate::error::Result;
use crate::p2p::{WorldInfo, NodeInfo};

pub struct WorldDiscovery;

impl WorldDiscovery {
    pub async fn new(_node_id: &str) -> Result<Self> { Ok(Self) }
    pub async fn announce_world(&self, _world: &WorldInfo) -> Result<()> { Ok(()) }
    pub async fn unannounce_world(&self) -> Result<()> { Ok(()) }
    pub async fn discover_worlds(&self) -> Result<Vec<WorldInfo>> { Ok(vec![]) }
    pub async fn get_nodes(&self) -> Result<Vec<NodeInfo>> { Ok(vec![]) }
}
