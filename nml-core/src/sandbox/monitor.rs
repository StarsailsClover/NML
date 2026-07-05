//! Sandbox monitor (stub)

use std::path::PathBuf;
use std::time::Duration;
use crate::error::Result;
use crate::launch::MinecraftProcess;
use super::MonitorResult;

pub struct SandboxMonitor;

impl SandboxMonitor {
    pub fn new() -> Self { Self }
    pub async fn monitor(&self, _process: &mut MinecraftProcess, _timeout: Duration) -> Result<MonitorResult> {
        Ok(MonitorResult::Success)
    }
}
