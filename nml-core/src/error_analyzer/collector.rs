//! Log collector (stub)

use std::path::PathBuf;
use crate::error::Result;

pub struct LogCollector;

impl LogCollector {
    pub fn new() -> Self { Self }
    pub async fn collect(&self, _path: &PathBuf) -> Result<String> { Ok(String::new()) }
}
