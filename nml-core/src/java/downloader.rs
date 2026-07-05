//! Java downloader (stub)

use std::path::{Path, PathBuf};
use crate::error::Result;
use super::JavaDistribution;

pub async fn download_java(_version: u8, _distribution: JavaDistribution, _install_path: &Path) -> Result<PathBuf> {
    Ok(PathBuf::new())
}

pub async fn extract_java(_archive: &Path, _install_path: &Path) -> Result<()> {
    Ok(())
}
