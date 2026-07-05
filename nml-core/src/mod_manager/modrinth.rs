//! Modrinth API (stub)

use crate::error::Result;
use super::{ModInfo, ModLoader};

pub struct ModrinthAPI;

impl ModrinthAPI {
    pub fn new() -> Self { Self }
    pub async fn search(&self, _query: &str, _mc_version: &str, _modloader: ModLoader) -> Result<Vec<ModInfo>> { Ok(vec![]) }
}
