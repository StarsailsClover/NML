//! Sandbox isolation environment (stub)

use std::process::Command;
use crate::error::Result;

pub struct IsolationEnvironment;

impl IsolationEnvironment {
    pub fn new() -> Self { Self }
    pub async fn create(&self) -> Result<Self> { Ok(Self) }
    pub fn apply_to_command(&self, _cmd: &mut Command) {}
}
