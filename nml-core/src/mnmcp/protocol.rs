//! MnMCP protocol (stub)

use crate::error::Result;
use super::MiniWorldServer;

pub struct MnMCPProtocol;

impl MnMCPProtocol {
    pub fn new() -> Self { Self }
    pub async fn start(&self) -> Result<()> { Ok(()) }
    pub async fn stop(&self) -> Result<()> { Ok(()) }
    pub async fn connect(&self, _ip: &str, _port: u16) -> Result<()> { Ok(()) }
    pub async fn discover_servers(&self) -> Result<Vec<MiniWorldServer>> { Ok(vec![]) }
}
