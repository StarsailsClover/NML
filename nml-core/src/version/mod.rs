//! Version management for Minecraft

pub mod models;
pub mod parser;
pub mod installer;

use std::path::PathBuf;
use async_trait::async_trait;

use crate::error::Result;
pub use models::*;

/// Version manager trait
#[async_trait]
pub trait VersionManager: Send + Sync {
    /// Get installed versions
    async fn get_installed_versions(&self) -> Result<Vec<InstalledVersion>>;
    
    /// Get remote versions from manifest
    async fn get_remote_versions(&self) -> Result<VersionManifest>;
    
    /// Get version info
    async fn get_version_info(&self, id: &str) -> Result<VersionInfo>;
    
    /// Install a version
    async fn install_version(&self, id: &str, progress: ProgressCallback) -> Result<()>;
    
    /// Uninstall a version
    async fn uninstall_version(&self, id: &str) -> Result<()>;
    
    /// Check if version is installed
    async fn is_version_installed(&self, id: &str) -> Result<bool>;
    
    /// Validate version integrity
    async fn validate_version(&self, id: &str) -> Result<bool>;
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(f32) + Send + Sync>;

/// Default version manager implementation
pub struct DefaultVersionManager {
    data_dir: PathBuf,
    http_client: reqwest::Client,
}

impl DefaultVersionManager {
    /// Create new version manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl VersionManager for DefaultVersionManager {
    async fn get_installed_versions(&self) -> Result<Vec<InstalledVersion>> {
        let versions_dir = self.data_dir.join("versions");
        if !versions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        
        for entry in std::fs::read_dir(&versions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let version_id = entry.file_name().to_string_lossy().to_string();
                let version_json = entry.path().join(format!("{}.json", version_id));
                
                if version_json.exists() {
                    let content = std::fs::read_to_string(&version_json)?;
                    let info: VersionInfo = serde_json::from_str(&content)?;
                    
                    versions.push(InstalledVersion {
                        id: version_id,
                        info,
                        installed_at: entry.metadata()?.modified()?,
                    });
                }
            }
        }

        versions.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
        Ok(versions)
    }

    async fn get_remote_versions(&self) -> Result<VersionManifest> {
        // Try multiple sources
        let sources = [
            "https://piston-meta.mojang.com/mc/game/version_manifest.json",
            "https://bmclapi2.bangbang93.com/mc/game/version_manifest.json",
            "https://download.mcbbs.net/mc/game/version_manifest.json",
        ];

        for url in &sources {
            match self.http_client.get(*url).send().await {
                Ok(response) if response.status().is_success() => {
                    let manifest: VersionManifest = response.json().await?;
                    return Ok(manifest);
                }
                _ => continue,
            }
        }

        Err(crate::error::NMLError::DownloadFailed(
            "Failed to fetch version manifest from all sources".to_string()
        ))
    }

    async fn get_version_info(&self, id: &str) -> Result<VersionInfo> {
        let version_json = self.data_dir.join("versions").join(id).join(format!("{}.json", id));
        
        if version_json.exists() {
            let content = std::fs::read_to_string(&version_json)?;
            let info: VersionInfo = serde_json::from_str(&content)?;
            Ok(info)
        } else {
            Err(crate::error::NMLError::VersionNotFound(id.to_string()))
        }
    }

    async fn install_version(&self, id: &str, progress: ProgressCallback) -> Result<()> {
        let installer = installer::VersionInstaller::new(self.data_dir.clone(), self.http_client.clone());
        installer.install(id, progress).await
    }

    async fn uninstall_version(&self, id: &str) -> Result<()> {
        if id.contains("..") || id.contains('/') || id.contains('\\') {
            return Err(crate::error::NMLError::InvalidConfig(format!("Invalid version id: {}", id)));
        }

        let version_dir = self.data_dir.join("versions").join(id);
        
        if !version_dir.exists() {
            return Err(crate::error::NMLError::VersionNotFound(id.to_string()));
        }

        std::fs::remove_dir_all(&version_dir)?;
        Ok(())
    }

    async fn is_version_installed(&self, id: &str) -> Result<bool> {
        let version_dir = self.data_dir.join("versions").join(id);
        let version_jar = version_dir.join(format!("{}.jar", id));
        let version_json = version_dir.join(format!("{}.json", id));
        
        Ok(version_dir.exists() && version_jar.exists() && version_json.exists())
    }

    async fn validate_version(&self, id: &str) -> Result<bool> {
        let info = self.get_version_info(id).await?;
        let version_jar = self.data_dir.join("versions").join(id).join(format!("{}.jar", id));

        if !version_jar.exists() {
            return Ok(false);
        }

        // Validate SHA1 if available
        if let Some(downloads) = info.downloads {
            if let Some(client) = downloads.client {
                let file_sha1 = util::calculate_sha1(&version_jar).await?;
                if file_sha1 != client.sha1 {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

/// Utility module
mod util {
    use std::path::Path;
    use sha1::{Digest, Sha1};
    use crate::error::Result;

    pub async fn calculate_sha1(path: &Path) -> Result<String> {
        let content = tokio::fs::read(path).await?;
        let hash = Sha1::digest(&content);
        Ok(format!("{:x}", hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager_creation() {
        let manager = DefaultVersionManager::new(PathBuf::from("/tmp/nml"));
        // Just verify it compiles
    }
}
