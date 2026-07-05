//! LAN injector for minecraftBC (stub)

use crate::error::Result;
use crate::p2p::WorldInfo;

pub struct LANInjector;

impl LANInjector {
    pub async fn new() -> Result<Self> { Ok(Self) }
    pub async fn inject_world(&self, _world_id: &str, _world_info: &WorldInfo, _port: u16) -> Result<()> { Ok(()) }
    pub async fn inject_server(&self, _world_info: &WorldInfo, _port: u16) -> Result<()> { Ok(()) }
    pub async fn remove_injected_world(&self, _world_id: &str) -> Result<()> { Ok(()) }
    pub async fn remove_injected_server(&self, _world_id: &str) -> Result<()> { Ok(()) }
}
