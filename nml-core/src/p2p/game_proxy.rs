//! Game proxy (stub)

use crate::error::Result;

pub struct GameProxy;

impl GameProxy {
    pub async fn new() -> Result<Self> { Ok(Self) }
    pub async fn start_proxy(&self, _world_id: &str, _local_port: u16) -> Result<()> { Ok(()) }
    pub async fn stop_proxy(&self) -> Result<()> { Ok(()) }
    pub async fn connect_to_world(&self, _host_node_id: &str) -> Result<()> { Ok(()) }
}
