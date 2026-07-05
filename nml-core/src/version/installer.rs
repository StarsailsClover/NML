//! Version installer - downloads and installs Minecraft versions

use std::path::PathBuf;

use crate::download::{DownloadEngine, DownloadTask};
use crate::error::{NMLError, Result};
use crate::version::models::*;
use crate::version::ProgressCallback;

pub struct VersionInstaller {
    data_dir: PathBuf,
    client: reqwest::Client,
}

impl VersionInstaller {
    pub fn new(data_dir: PathBuf, client: reqwest::Client) -> Self {
        Self { data_dir, client }
    }

    /// Install a Minecraft version
    pub async fn install(&self, id: &str, mut progress: ProgressCallback) -> Result<()> {
        // Step 1: Fetch version manifest
        progress(0.02);
        let manifest: VersionManifest = self
            .client
            .get("https://piston-meta.mojang.com/mc/game/version_manifest.json")
            .send()
            .await
            .map_err(|e| NMLError::DownloadFailed(format!("Failed to fetch manifest: {}", e)))?
            .json()
            .await
            .map_err(|e| NMLError::DownloadFailed(format!("Failed to parse manifest: {}", e)))?;

        // Step 2: Find version entry
        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == id)
            .ok_or_else(|| NMLError::VersionNotFound(id.to_string()))?
            .clone();

        progress(0.05);

        // Step 3: Fetch full version JSON
        let info: VersionInfo = self
            .client
            .get(&entry.url)
            .send()
            .await
            .map_err(|e| NMLError::DownloadFailed(format!("Failed to fetch version json: {}", e)))?
            .json()
            .await
            .map_err(|e| {
                NMLError::DownloadFailed(format!("Failed to parse version json: {}", e))
            })?;

        let total_libraries = info.libraries.len();
        let mut completed = 0usize;

        // Step 4: Create directories
        let version_dir = self.data_dir.join("versions").join(id);
        tokio::fs::create_dir_all(&version_dir).await?;
        let libraries_dir = self.data_dir.join("libraries");
        tokio::fs::create_dir_all(&libraries_dir).await?;

        progress(0.08);

        // Step 5: Save version JSON
        let json = serde_json::to_string_pretty(&info)?;
        tokio::fs::write(version_dir.join(format!("{}.json", id)), json).await?;

        progress(0.10);

        // Step 6: Download client jar
        if let Some(downloads) = &info.downloads {
            if let Some(client_dl) = &downloads.client {
                let engine = DownloadEngine::new();
                let task = DownloadTask {
                    url: client_dl.url.clone(),
                    dest: version_dir.join(format!("{}.jar", id)),
                    sha1: Some(client_dl.sha1.clone()),
                    size: Some(client_dl.size as u64),
                    priority: 1,
                };
                engine.download(&task, |p| {
                    progress(0.10 + p * 0.20);
                }).await?;
            }
        }

        progress(0.30);

        // Step 7: Download libraries
        if total_libraries > 0 {
            let engine = DownloadEngine::new();

            for library in &info.libraries {
                // Skip platform-incompatible libraries
                if !library_applies_to_current_os(library) {
                    completed += 1;
                    continue;
                }

                // Download main artifact
                if let Some(artifact) = &library.downloads.artifact {
                    let dest = get_library_path(&libraries_dir, &library.name);
                    if !dest.exists() {
                        if let Some(parent) = dest.parent() {
                            tokio::fs::create_dir_all(parent).await?;
                        }
                        let task = DownloadTask {
                            url: artifact.url.clone(),
                            dest,
                            sha1: Some(artifact.sha1.clone()),
                            size: Some(artifact.size as u64),
                            priority: 3,
                        };
                        engine.download(&task, |_| {}).await?;
                    }
                }

                // Download natives for current OS
                if let Some(natives) = &library.natives {
                    let os_key = current_os_native_key();
                    if let Some(classifier) = natives.get(&os_key) {
                        if let Some(classifiers) = &library.downloads.classifiers {
                            if let Some(native_dl) = classifiers.get(classifier) {
                                let native_name = format!("{}-{}", library.name, classifier);
                                let dest = get_library_path(&libraries_dir, &native_name);
                                if !dest.exists() {
                                    if let Some(parent) = dest.parent() {
                                        tokio::fs::create_dir_all(parent).await?;
                                    }
                                    let task = DownloadTask {
                                        url: native_dl.url.clone(),
                                        dest,
                                        sha1: Some(native_dl.sha1.clone()),
                                        size: Some(native_dl.size as u64),
                                        priority: 4,
                                    };
                                    engine.download(&task, |_| {}).await?;
                                }
                            }
                        }
                    }
                }

                completed += 1;
                let pct = 0.30 + (completed as f32 / total_libraries as f32) * 0.65;
                progress(pct);
            }
        }

        progress(1.0);
        Ok(())
    }
}

fn library_applies_to_current_os(library: &Library) -> bool {
    if let Some(rules) = &library.rules {
        let mut allowed = false;
        for rule in rules {
            let os_ok = match &rule.os {
                Some(os) => match &os.name {
                    Some(name) => name == current_os_name(),
                    None => true,
                },
                None => true,
            };
            if rule.action == RuleAction::Allow {
                allowed = allowed || os_ok;
            } else {
                allowed = !os_ok;
            }
        }
        if !allowed {
            return false;
        }
    }
    true
}

/// Build library path from Maven coordinates (group:artifact:version)
fn get_library_path(base_dir: &PathBuf, name: &str) -> PathBuf {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return PathBuf::new();
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts.get(3).map(|c| format!("-{}", c)).unwrap_or_default();

    base_dir
        .join(group)
        .join(artifact)
        .join(version)
        .join(format!("{}-{}{}.jar", artifact, version, classifier))
}

fn current_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn current_os_native_key() -> String {
    if cfg!(target_os = "windows") {
        "natives-windows".to_string()
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "natives-macos-arm64".to_string()
        } else {
            "natives-macos".to_string()
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "natives-linux-arm64".to_string()
        } else {
            "natives-linux".to_string()
        }
    } else {
        "natives-linux".to_string()
    }
}
