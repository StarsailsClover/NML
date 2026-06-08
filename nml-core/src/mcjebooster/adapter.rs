//! MCJEBooster adapter management

use std::collections::HashMap;
use std::path::Path;

use crate::error::{NMLError, Result};

/// Version adapter definition
#[derive(Debug, Clone)]
pub struct VersionAdapter {
    pub mc_version: String,
    pub modloader: String,
    pub jvm_args: Vec<String>,
    pub supports_tick_optimization: bool,
    pub supports_region_scheduling: bool,
    pub supports_sync_points: bool,
    pub config: AdapterConfig,
}

/// Adapter configuration
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    pub worker_threads: u32,
    pub scheduler_tick_rate: u32,
    pub sync_point_interval: u32,
    pub chunk_load_optimization: bool,
    pub entity_optimization: bool,
}

/// Adapter loader
pub struct AdapterLoader {
    adapters: HashMap<String, VersionAdapter>,
    adapter_dir: PathBuf,
}

impl AdapterLoader {
    /// Create new loader
    pub fn new(adapter_dir: &Path) -> Result<Self> {
        let mut loader = Self {
            adapters: HashMap::new(),
            adapter_dir: adapter_dir.to_path_buf(),
        };
        
        // Load all adapters
        loader.load_adapters()?;
        
        Ok(loader)
    }
    
    /// Load all adapters from directory
    fn load_adapters(&mut self) -> Result<()> {
        if !self.adapter_dir.exists() {
            std::fs::create_dir_all(&self.adapter_dir)?;
            return Ok(());
        }
        
        for entry in std::fs::read_dir(&self.adapter_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("mcjeb") {
                if let Some(adapter) = self.parse_adapter(&path)? {
                    let key = format!("{}-{}", adapter.mc_version, adapter.modloader);
                    self.adapters.insert(key, adapter);
                }
            }
        }
        
        tracing::info!("Loaded {} MCJEBooster adapters", self.adapters.len());
        Ok(())
    }
    
    /// Parse adapter file (.mcjeb)
    fn parse_adapter(&self, path: &Path) -> Result<Option<VersionAdapter>> {
        // Parse MCJEBooster adapter format
        // This is a simplified version - actual format may vary
        
        let content = std::fs::read_to_string(path)?;
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| NMLError::Other("Invalid filename".to_string()))?;
        
        // Parse filename like "1.16.5-Forge.mcjeb"
        let parts: Vec<&str> = filename.split('-').collect();
        if parts.len() < 2 {
            return Ok(None);
        }
        
        let mc_version = parts[0].to_string();
        let modloader = parts[1..].join("-");
        
        // Parse JSON content
        let config: serde_json::Value = serde_json::from_str(&content)?;
        
        let adapter = VersionAdapter {
            mc_version,
            modloader,
            jvm_args: config["jvm_args"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            supports_tick_optimization: config["tick_optimization"].as_bool().unwrap_or(false),
            supports_region_scheduling: config["region_scheduling"].as_bool().unwrap_or(false),
            supports_sync_points: config["sync_points"].as_bool().unwrap_or(false),
            config: AdapterConfig {
                worker_threads: config["worker_threads"].as_u64().unwrap_or(4) as u32,
                scheduler_tick_rate: config["scheduler_tick_rate"].as_u64().unwrap_or(20) as u32,
                sync_point_interval: config["sync_point_interval"].as_u64().unwrap_or(100) as u32,
                chunk_load_optimization: config["chunk_load_optimization"].as_bool().unwrap_or(true),
                entity_optimization: config["entity_optimization"].as_bool().unwrap_or(true),
            },
        };
        
        Ok(Some(adapter))
    }
    
    /// Find suitable adapter for version
    pub fn find_adapter(&self, mc_version: &str, modloader: &str) -> Result<VersionAdapter> {
        let key = format!("{}-{}", mc_version, modloader);
        
        if let Some(adapter) = self.adapters.get(&key) {
            return Ok(adapter.clone());
        }
        
        // Try without modloader (Vanilla)
        let vanilla_key = format!("{}-Vanilla", mc_version);
        if let Some(adapter) = self.adapters.get(&vanilla_key) {
            return Ok(adapter.clone());
        }
        
        Err(NMLError::Other(format!(
            "No MCJEBooster adapter found for {}-{}",
            mc_version, modloader
        )))
    }
    
    /// Check if adapter exists
    pub fn has_adapter(&self, version: &str) -> bool {
        self.adapters.keys().any(|k| k.starts_with(version))
    }
}
