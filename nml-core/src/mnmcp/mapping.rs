//! Block mapping MC <-> Mini World (stub)

use crate::error::Result;

pub struct BlockMapping;

impl BlockMapping {
    pub async fn load() -> Result<Self> { Ok(Self) }
    pub fn mc_to_mini(&self, id: u32) -> u32 { id }
    pub fn mini_to_mc(&self, id: u32) -> u32 { id }
}
