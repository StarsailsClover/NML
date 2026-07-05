//! Server config management (stub)

use std::path::Path;
use crate::error::Result;
use super::ServerConfig;

pub async fn load_config(_dir: &Path) -> Result<ServerConfig> { Ok(ServerConfig::default()) }
pub async fn save_config(_dir: &Path, _config: &ServerConfig) -> Result<()> { Ok(()) }
