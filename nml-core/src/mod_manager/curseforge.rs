//! CurseForge API (stub)

use crate::error::Result;
use super::{ModInfo, ModLoader};

pub struct CurseForgeAPI;

impl CurseForgeAPI {
    pub fn new() -> Self { Self }
    pub async fn search(&self, _query: &str, _mc_version: &str, _modloader: ModLoader) -> Result<Vec<ModInfo>> { Ok(vec![]) }
    pub async fn get_latest_version(&self, _id: &str) -> Result<Option<ModInfo>> { Ok(None) }
}
