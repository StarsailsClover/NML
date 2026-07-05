//! Linux hook stub

use crate::error::Result;

pub async fn install_hook(_pid: u32) -> Result<()> { Ok(()) }
pub async fn uninstall_hook(_pid: u32) -> Result<()> { Ok(()) }
