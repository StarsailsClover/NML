//! Version upgrade assistant
//!
//! Helps migrate configs, mods, and saves between Minecraft versions

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{NMLError, Result};
use crate::mod_manager::{ModManager, ModInfo};

pub mod config_migrator;
pub mod mod_migrator;
pub mod save_migrator;

/// Upgrade assistant
pub struct UpgradeAssistant {
    from_version: String,
    to_version: String,
    game_dir: PathBuf,
}

/// Upgrade plan
#[derive(Debug, Clone)]
pub struct UpgradePlan {
    pub from_version: String,
    pub to_version: String,
    pub steps: Vec<UpgradeStep>,
    pub risks: Vec<String>,
    pub estimated_time: Duration,
}

/// Upgrade step
#[derive(Debug, Clone)]
pub struct UpgradeStep {
    pub name: String,
    pub description: String,
    pub category: UpgradeCategory,
    pub action: UpgradeAction,
    pub auto_fixable: bool,
}

/// Upgrade category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeCategory {
    Config,
    Mod,
    ResourcePack,
    Save,
    Shader,
}

/// Upgrade action
#[derive(Debug, Clone)]
pub enum UpgradeAction {
    Copy { from: PathBuf, to: PathBuf },
    Convert { path: PathBuf, converter: String },
    Remove { path: PathBuf },
    Download { name: String, url: String },
}

impl UpgradeAssistant {
    /// Create new assistant
    pub fn new(from_version: &str, to_version: &str, game_dir: PathBuf) -> Self {
        Self {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            game_dir,
        }
    }
    
    /// Analyze upgrade compatibility
    pub async fn analyze(&self) -> Result<UpgradePlan> {
        tracing::info!("Analyzing upgrade from {} to {}", self.from_version, self.to_version);
        
        let mut steps = vec![];
        let mut risks = vec![];
        
        // Check config migration
        let config_steps = self.analyze_config_migration().await?;
        steps.extend(config_steps);
        
        // Check mod compatibility
        let mod_steps = self.analyze_mod_compatibility().await?;
        steps.extend(mod_steps);
        
        // Check resource pack compatibility
        let rp_steps = self.analyze_resource_pack_compatibility().await?;
        steps.extend(rp_steps);
        
        // Check save compatibility
        let save_steps = self.analyze_save_compatibility().await?;
        steps.extend(save_steps);
        
        // Generate risks
        if self.is_major_upgrade() {
            risks.push("跨大版本升级，部分Mod可能不兼容".to_string());
        }
        
        if self.mod_count() > 50 {
            risks.push("Mod数量较多，升级后需要逐一检查".to_string());
        }
        
        Ok(UpgradePlan {
            from_version: self.from_version.clone(),
            to_version: self.to_version.clone(),
            steps,
            risks,
            estimated_time: Duration::from_secs(self.steps.len() as u64 * 30),
        })
    }
    
    /// Execute upgrade
    pub async fn execute(&self, plan: &UpgradePlan, progress: &dyn Fn(f32)) -> Result<()> {
        let total = plan.steps.len();
        
        for (i, step) in plan.steps.iter().enumerate() {
            tracing::info!("Executing upgrade step: {}", step.name);
            
            match &step.action {
                UpgradeAction::Copy { from, to } => {
                    tokio::fs::copy(from, to).await?;
                }
                UpgradeAction::Convert { path, converter } => {
                    self.convert_file(path, converter).await?;
                }
                UpgradeAction::Remove { path } => {
                    if path.exists() {
                        tokio::fs::remove_file(path).await?;
                    }
                }
                UpgradeAction::Download { name, url } => {
                    self.download_upgrade(name, url).await?;
                }
            }
            
            progress((i + 1) as f32 / total as f32);
        }
        
        Ok(())
    }
    
    /// Analyze config migration
    async fn analyze_config_migration(&self) -> Result<Vec<UpgradeStep>> {
        let mut steps = vec![];
        
        let config_dir = self.game_dir.join("config");
        if config_dir.exists() {
            steps.push(UpgradeStep {
                name: "备份配置文件".to_string(),
                description: "备份当前版本配置".to_string(),
                category: UpgradeCategory::Config,
                action: UpgradeAction::Copy {
                    from: config_dir.clone(),
                    to: self.backup_path("config"),
                },
                auto_fixable: true,
            });
            
            steps.push(UpgradeStep {
                name: "迁移配置".to_string(),
                description: "迁移兼容的配置文件".to_string(),
                category: UpgradeCategory::Config,
                action: UpgradeAction::Copy {
                    from: self.backup_path("config"),
                    to: config_dir,
                },
                auto_fixable: true,
            });
        }
        
        Ok(steps)
    }
    
    /// Analyze mod compatibility
    async fn analyze_mod_compatibility(&self) -> Result<Vec<UpgradeStep>> {
        let mut steps = vec![];
        let mod_manager = ModManager::new(self.game_dir.clone());
        
        let installed = mod_manager.get_installed();
        
        for mod_info in installed {
            let is_compatible = self.check_mod_compatibility(&mod_info.mc_version);
            
            if !is_compatible {
                steps.push(UpgradeStep {
                    name: format!("检查Mod: {}", mod_info.info.name),
                    description: format!("寻找 {} 的兼容版本", mod_info.info.name),
                    category: UpgradeCategory::Mod,
                    action: UpgradeAction::Download {
                        name: mod_info.info.name.clone(),
                        url: format!("curseforge/{}-{}", mod_info.info.name, self.to_version),
                    },
                    auto_fixable: false,
                });
            }
        }
        
        Ok(steps)
    }
    
    /// Analyze resource pack compatibility
    async fn analyze_resource_pack_compatibility(&self) -> Result<Vec<UpgradeStep>> {
        let mut steps = vec![];
        
        let rp_dir = self.game_dir.join("resourcepacks");
        if rp_dir.exists() {
            steps.push(UpgradeStep {
                name: "检查资源包兼容性".to_string(),
                description: "验证资源包是否兼容新版本".to_string(),
                category: UpgradeCategory::ResourcePack,
                action: UpgradeAction::Copy {
                    from: rp_dir.clone(),
                    to: self.backup_path("resourcepacks"),
                },
                auto_fixable: true,
            });
        }
        
        Ok(steps)
    }
    
    /// Analyze save compatibility
    async fn analyze_save_compatibility(&self) -> Result<Vec<UpgradeStep>> {
        let mut steps = vec![];
        
        let saves_dir = self.game_dir.join("saves");
        if saves_dir.exists() {
            steps.push(UpgradeStep {
                name: "备份存档".to_string(),
                description: "重要：备份所有世界存档".to_string(),
                category: UpgradeCategory::Save,
                action: UpgradeAction::Copy {
                    from: saves_dir.clone(),
                    to: self.backup_path("saves"),
                },
                auto_fixable: true,
            });
            
            if self.needs_world_conversion() {
                steps.push(UpgradeStep {
                    name: "转换存档格式".to_string(),
                    description: "升级存档到新版格式".to_string(),
                    category: UpgradeCategory::Save,
                    action: UpgradeAction::Convert {
                        path: saves_dir,
                        converter: "anvil".to_string(),
                    },
                    auto_fixable: true,
                });
            }
        }
        
        Ok(steps)
    }
    
    /// Check if major upgrade
    fn is_major_upgrade(&self) -> bool {
        let from: Vec<u32> = self.from_version.split('.').filter_map(|s| s.parse().ok()).collect();
        let to: Vec<u32> = self.to_version.split('.').filter_map(|s| s.parse().ok()).collect();
        
        if from.len() >= 2 && to.len() >= 2 {
            from[1] != to[1]
        } else {
            false
        }
    }
    
    /// Check mod compatibility
    fn check_mod_compatibility(&self, mod_mc_version: &str) -> bool {
        mod_mc_version == self.to_version
    }
    
    /// Check if needs world conversion
    fn needs_world_conversion(&self) -> bool {
        // Check if crossing versions that need Anvil conversion
        let from: Vec<u32> = self.from_version.split('.').filter_map(|s| s.parse().ok()).collect();
        
        if from.len() >= 2 {
            // 1.13+ has different world format
            from[0] == 1 && from[1] < 13
        } else {
            false
        }
    }
    
    /// Get mod count
    fn mod_count(&self) -> usize {
        // Simplified
        0
    }
    
    /// Backup path
    fn backup_path(&self, name: &str) -> PathBuf {
        self.game_dir.join(".nml_backup").join(&self.from_version).join(name)
    }
    
    /// Convert file
    async fn convert_file(&self, _path: &PathBuf, _converter: &str) -> Result<()> {
        // TODO: Implement file conversion
        Ok(())
    }
    
    /// Download upgrade
    async fn download_upgrade(&self, _name: &str, _url: &str) -> Result<()> {
        // TODO: Implement download
        Ok(())
    }
}

use std::time::Duration;
