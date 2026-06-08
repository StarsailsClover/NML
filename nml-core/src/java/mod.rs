//! Java auto-download and management

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::error::{NMLError, Result};
use crate::version::JavaVersionInfo;

pub mod detector;
pub mod downloader;

/// Java distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JavaDistribution {
    Adoptium,      // Eclipse Adoptium (formerly AdoptOpenJDK)
    Oracle,        // Oracle JDK
    Microsoft,     // Microsoft Build of OpenJDK
    Amazon,        // Amazon Corretto
    Azul,          // Azul Zulu
    Alibaba,       // Alibaba Dragonwell
    Tencent,       // Tencent Kona
    BiSheng,       // Huawei BiSheng
}

impl JavaDistribution {
    pub fn name(&self) -> &'static str {
        match self {
            JavaDistribution::Adoptium => "Adoptium",
            JavaDistribution::Oracle => "Oracle",
            JavaDistribution::Microsoft => "Microsoft",
            JavaDistribution::Amazon => "Amazon Corretto",
            JavaDistribution::Azul => "Azul Zulu",
            JavaDistribution::Alibaba => "Alibaba Dragonwell",
            JavaDistribution::Tencent => "Tencent Kona",
            JavaDistribution::BiSheng => "Huawei BiSheng",
        }
    }
    
    pub fn download_url(&self, version: u8, os: &str, arch: &str) -> String {
        match self {
            JavaDistribution::Adoptium => {
                format!(
                    "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
                    version, os, arch
                )
            }
            JavaDistribution::Microsoft => {
                format!(
                    "https://aka.ms/download-jdk/microsoft-{0}-{1}-{2}.zip",
                    version, os, arch
                )
            }
            _ => String::new(),
        }
    }
}

/// Installed Java instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstance {
    pub version: String,
    pub major_version: u8,
    pub path: PathBuf,
    pub distribution: JavaDistribution,
    pub is_64bit: bool,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl JavaInstance {
    /// Check if suitable for Minecraft version
    pub fn is_suitable_for(&self, mc_version: &str) -> bool {
        let required = required_java_version(mc_version);
        self.major_version >= required
    }
    
    /// Get executable path
    pub fn executable(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        return self.path.join("bin").join("java.exe");
        
        #[cfg(not(target_os = "windows"))]
        return self.path.join("bin").join("java");
    }
}

/// Java manager
pub struct JavaManager {
    install_dir: PathBuf,
    instances: Vec<JavaInstance>,
}

impl JavaManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            install_dir: data_dir.join("java"),
            instances: Vec::new(),
        }
    }
    
    /// Detect all installed Java versions
    pub async fn detect_all(&mut self) -> Result<()> {
        self.instances.clear();
        
        // Detect system Java
        if let Some(system_java) = detector::detect_system_java().await? {
            self.instances.push(system_java);
        }
        
        // Detect NML installed Java
        if self.install_dir.exists() {
            for entry in std::fs::read_dir(&self.install_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(java) = detector::detect_java_at(&entry.path()).await? {
                        self.instances.push(java);
                    }
                }
            }
        }
        
        // Detect common locations
        let common_locations = detector::get_common_java_locations();
        for location in common_locations {
            if let Some(java) = detector::detect_java_at(&location).await? {
                if !self.instances.iter().any(|j| j.path == java.path) {
                    self.instances.push(java);
                }
            }
        }
        
        // Sort by version (newest first)
        self.instances.sort_by(|a, b| b.major_version.cmp(&a.major_version));
        
        Ok(())
    }
    
    /// Get all detected Java instances
    pub fn get_all(&self) -> &[JavaInstance] {
        &self.instances
    }
    
    /// Find best Java for Minecraft version
    pub fn find_best_for(&self, mc_version: &str) -> Option<&JavaInstance> {
        let required = required_java_version(mc_version);
        
        self.instances.iter()
            .filter(|j| j.major_version >= required)
            .min_by_key(|j| j.major_version) // Prefer lowest compatible
    }
    
    /// Download and install Java
    pub async fn install(&self, version: u8, distribution: JavaDistribution) -> Result<JavaInstance> {
        let install_path = self.install_dir.join(format!("{}-{}", distribution.name(), version));
        
        // Download
        let archive = downloader::download_java(version, distribution, &install_path).await?;
        
        // Extract
        downloader::extract_java(&archive, &install_path).await?;
        
        // Detect
        let java = detector::detect_java_at(&install_path).await?
            .ok_or_else(|| NMLError::Other("Failed to detect installed Java".to_string()))?;
        
        Ok(java)
    }
    
    /// Auto-install if needed
    pub async fn auto_install(&self, mc_version: &str) -> Result<Option<JavaInstance>> {
        let required = required_java_version(mc_version);
        
        // Check if already have suitable Java
        if let Some(java) = self.find_best_for(mc_version) {
            return Ok(Some(java.clone()));
        }
        
        // Need to install
        tracing::info!("Auto-installing Java {} for Minecraft {}", required, mc_version);
        
        let java = self.install(required, JavaDistribution::Adoptium).await?;
        
        Ok(Some(java))
    }
}

/// Get required Java version for Minecraft
pub fn required_java_version(mc_version: &str) -> u8 {
    let parts: Vec<&str> = mc_version.split('.').collect();
    
    if parts.len() < 2 {
        return 8;
    }
    
    let major = parts[0].parse::<u32>().unwrap_or(1);
    let minor = parts[1].parse::<u32>().unwrap_or(0);
    
    if major == 1 {
        match minor {
            0..=15 => 8,
            16 => {
                // 1.16 needs Java 8, but 1.16.5+ can use 11
                if parts.len() >= 3 {
                    let patch = parts[2].parse::<u32>().unwrap_or(0);
                    if patch >= 5 {
                        return 11;
                    }
                }
                8
            }
            17 => 16,
            18 => 17,
            19 => 17,
            20..=u32::MAX => 17,
            _ => 8,
        }
    } else {
        17 // Future versions
    }
}
