//! Mod dependency resolver (stub)

use crate::error::Result;
use super::ModInfo;

pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new() -> Self { Self }
    pub async fn resolve_dependencies(&self, _mod_info: &ModInfo) -> Result<Vec<ModInfo>> { Ok(vec![]) }
}
