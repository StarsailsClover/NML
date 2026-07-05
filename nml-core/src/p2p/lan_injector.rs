//! LAN world injector (stub)

use std::path::PathBuf;
use crate::error::Result;
use crate::p2p::WorldInfo;

pub struct LANInjector;

impl LANInjector {
    pub async fn new(_data_dir: PathBuf) -> Result<Self> { Ok(Self) }
    pub async fn inject_world(&self, _world: &WorldInfo) -> Result<()> { Ok(()) }
}
