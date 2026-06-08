//! Mod management system
//!
//! Supports CurseForge and Modrinth APIs

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{NMLError, Result};

pub mod curseforge;
pub mod modrinth;
pub mod resolver;

/// Mod information
#[derive(Debug, Clone)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub mc_version: String,
    pub modloader: ModLoader,
    pub download_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub sha256: Option<String>,
    pub dependencies: Vec<String>,
    pub is_installed: bool,
    pub is_enabled: bool,
}

/// Mod loader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModLoader {
    Forge,
    Fabric,
    NeoForge,
    Quilt,
    LiteLoader,
    OptiFine,
}

impl ModLoader {
    pub fn display_name(&self) -> &'static str {
        match self {
            ModLoader::Forge => "Forge",
            ModLoader::Fabric => "Fabric",
            ModLoader::NeoForge => "NeoForge",
            ModLoader::Quilt => "Quilt",
            ModLoader::LiteLoader => "LiteLoader",
            ModLoader::OptiFine => "OptiFine",
        }
    }
}

/// Mod manager
pub struct ModManager {
    data_dir: PathBuf,
    curseforge: curseforge::CurseForgeAPI,
    modrinth: modrinth::ModrinthAPI,
    resolver: resolver::DependencyResolver,
    installed_mods: HashMap<String, InstalledMod>,
}

pub struct InstalledMod {
    pub info: ModInfo,
    pub install_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

impl ModManager {
    /// Create new mod manager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir: data_dir.join("mods"),
            curseforge: curseforge::CurseForgeAPI::new(),
            modrinth: modrinth::ModrinthAPI::new(),
            resolver: resolver::DependencyResolver::new(),
            installed_mods: HashMap::new(),
        }
    }
    
    /// Search mods
    pub async fn search(&self, query: &str, mc_version: &str, modloader: ModLoader) -> Result<Vec<ModInfo>> {
        // Search both CurseForge and Modrinth
        let cf_results = self.curseforge.search(query, mc_version, modloader).await?;
        let mr_results = self.modrinth.search(query, mc_version, modloader).await?;
        
        // Merge and deduplicate
        let mut results = cf_results;
        results.extend(mr_results);
        
        // Sort by relevance
        results.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(results)
    }
    
    /// Install mod
    pub async fn install(&mut self, mod_info: &ModInfo) -> Result<()> {
        tracing::info!("Installing mod: {}", mod_info.name);
        
        // Resolve dependencies
        let dependencies = self.resolver.resolve_dependencies(mod_info).await?;
        
        // Download mod
        let mod_path = self.download_mod(mod_info).await?;
        
        // Download dependencies
        for dep in dependencies {
            self.download_mod(&dep).await?;
        }
        
        // Add to installed list
        self.installed_mods.insert(
            mod_info.id.clone(),
            InstalledMod {
                info: mod_info.clone(),
                install_path: mod_path,
                installed_at: chrono::Utc::now(),
            }
        );
        
        tracing::info!("Mod installed successfully");
        Ok(())
    }
    
    /// Download mod file
    async fn download_mod(&self, mod_info: &ModInfo) -> Result<PathBuf> {
        let mod_dir = self.data_dir.join("mods").join(&mod_info.mc_version);
        std::fs::create_dir_all(&mod_dir)?;
        
        let mod_path = mod_dir.join(&mod_info.file_name);
        
        // Download
        let response = reqwest::get(&mod_info.download_url).await?;
        let bytes = response.bytes().await?;
        
        tokio::fs::write(&mod_path, bytes).await?;
        
        Ok(mod_path)
    }
    
    /// Uninstall mod
    pub async fn uninstall(&mut self, mod_id: &str) -> Result<()> {
        if let Some(installed) = self.installed_mods.remove(mod_id) {
            tokio::fs::remove_file(&installed.install_path).await?;
            tracing::info!("Mod uninstalled: {}", mod_id);
        }
        
        Ok(())
    }
    
    /// Enable/Disable mod
    pub async fn toggle_mod(&self, mod_id: &str, enabled: bool) -> Result<()> {
        // Rename .jar to .disabled or vice versa
        // TODO: Implement
        Ok(())
    }
    
    /// Check for updates
    pub async fn check_updates(&self) -> Result<Vec<ModInfo>> {
        let mut updates = vec![];
        
        for (id, installed) in &self.installed_mods {
            if let Some(latest) = self.curseforge.get_latest_version(id).await? {
                if latest.version != installed.info.version {
                    updates.push(latest);
                }
            }
        }
        
        Ok(updates)
    }
    
    /// Update mod
    pub async fn update(&mut self, mod_id: &str) -> Result<()> {
        let installed = self.installed_mods.get(mod_id)
            .ok_or_else(|| NMLError::Other("Mod not installed".to_string()))?;
        
        // Search for latest version
        let latest = self.curseforge.get_latest_version(mod_id).await?;
        
        // Download and install
        self.install(&latest).await?;
        
        // Remove old version
        tokio::fs::remove_file(&installed.install_path).await?;
        
        Ok(())
    }
    
    /// Get installed mods
    pub fn get_installed(&self) -> Vec<&InstalledMod> {
        self.installed_mods.values().collect()
    }
}
