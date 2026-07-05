//! MnMCP bridge (stub)

use std::path::Path;
use crate::error::Result;

pub struct MnMCPBridge;

impl MnMCPBridge {
    pub async fn start(_data_dir: &Path) -> Result<Self> { Ok(Self) }
    pub async fn connect_to_minecraft(&self, _pid: u32) -> Result<()> { Ok(()) }
}
