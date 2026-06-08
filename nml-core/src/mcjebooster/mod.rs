//! MCJEBooster integration for performance optimization
//!
//! Integrates with StarsailsClover's MCJEBooster Java Agent

use std::path::{Path, PathBuf};
use std::process::Child;

use crate::error::{NMLError, Result};
use crate::launch::MinecraftProcess;

pub mod adapter;
pub mod injector;

use adapter::{AdapterLoader, VersionAdapter};
use injector::JavaAgentInjector;

/// MCJEBooster integration
pub struct MCJEBoosterIntegration {
    adapter_loader: AdapterLoader,
    injector: JavaAgentInjector,
    data_dir: PathBuf,
}

impl MCJEBoosterIntegration {
    /// Create new integration
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let adapter_dir = data_dir.join("mcjebooster").join("adapters");
        
        Ok(Self {
            adapter_loader: AdapterLoader::new(&adapter_dir)?,
            injector: JavaAgentInjector::new(),
            data_dir,
        })
    }
    
    /// Optimize Minecraft process
    pub async fn optimize(&self, process: &MinecraftProcess, mc_version: &str, modloader: &str) -> Result<()> {
        tracing::info!("Optimizing Minecraft {} with MCJEBooster", mc_version);
        
        // 1. Detect suitable adapter
        let adapter = self.adapter_loader.find_adapter(mc_version, modloader)?;
        
        // 2. Attach Java Agent
        self.injector.attach(process, &adapter).await?;
        
        // 3. Apply optimizations
        self.apply_optimizations(process, &adapter).await?;
        
        tracing::info!("MCJEBooster optimization applied");
        Ok(())
    }
    
    /// Apply optimizations based on adapter
    async fn apply_optimizations(&self, process: &MinecraftProcess, adapter: &VersionAdapter) -> Result<()> {
        // Apply JVM arguments from adapter
        for arg in &adapter.jvm_args {
            self.injector.add_jvm_arg(process, arg).await?;
        }
        
        // Apply tick optimizations
        if adapter.supports_tick_optimization {
            self.injector.enable_tick_optimization(process).await?;
        }
        
        // Apply region scheduling
        if adapter.supports_region_scheduling {
            self.injector.enable_region_scheduling(process).await?;
        }
        
        // Apply sync point management
        if adapter.supports_sync_points {
            self.injector.enable_sync_points(process).await?;
        }
        
        Ok(())
    }
    
    /// Get performance stats
    pub async fn get_stats(&self, process: &MinecraftProcess) -> Result<PerformanceStats> {
        self.injector.query_stats(process).await
    }
    
    /// Check if version is supported
    pub fn is_version_supported(&self, version: &str) -> bool {
        self.adapter_loader.has_adapter(version)
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub tps: f32,
    pub mspt: f32,
    pub chunk_load_time: f32,
    pub entity_count: u32,
    pub thread_count: u32,
    pub optimized_chunks: u32,
    pub active_schedulers: u32,
}
