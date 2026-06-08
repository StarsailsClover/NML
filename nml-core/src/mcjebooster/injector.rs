//! Java Agent injector for MCJEBooster
//!
//! Handles attaching Java Agent to running Minecraft JVM

use std::process::Command;
use std::path::Path;

use crate::error::{NMLError, Result};
use crate::launch::MinecraftProcess;
use super::adapter::VersionAdapter;

/// Java Agent injector
pub struct JavaAgentInjector;

impl JavaAgentInjector {
    /// Create new injector
    pub fn new() -> Self {
        Self
    }
    
    /// Attach agent to running process
    pub async fn attach(&self, process: &MinecraftProcess, adapter: &VersionAdapter) -> Result<()> {
        tracing::info!("Attaching MCJEBooster agent to PID {}", process.pid);
        
        // Get MCJEBooster agent JAR path
        let agent_jar = self.get_agent_jar_path()?;
        
        if !agent_jar.exists() {
            return Err(NMLError::Other(
                format!("MCJEBooster agent not found: {}", agent_jar.display())
            ));
        }
        
        // Attach via JVM Attach API
        #[cfg(target_os = "windows")]
        self.attach_windows(process, &agent_jar, adapter).await?;
        
        #[cfg(target_os = "linux")]
        self.attach_linux(process, &agent_jar, adapter).await?;
        
        #[cfg(target_os = "macos")]
        self.attach_macos(process, &agent_jar, adapter).await?;
        
        tracing::info!("MCJEBooster agent attached successfully");
        Ok(())
    }
    
    /// Attach on Windows
    #[cfg(target_os = "windows")]
    async fn attach_windows(&self, process: &MinecraftProcess, agent_jar: &Path, adapter: &VersionAdapter) -> Result<()> {
        // Use jattach.exe or direct JNI
        // For now, use jattach if available
        
        let jattach = self.find_jattach()?;
        
        let output = Command::new(&jattach)
            .arg(process.pid.to_string())
            .arg("load")
            .arg("instrument")
            .arg(format!("{}={}", agent_jar.display(), adapter.config.worker_threads))
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(NMLError::Other(format!("Agent attach failed: {}", error)));
        }
        
        Ok(())
    }
    
    /// Attach on Linux
    #[cfg(target_os = "linux")]
    async fn attach_linux(&self, process: &MinecraftProcess, agent_jar: &Path, adapter: &VersionAdapter) -> Result<()> {
        // Use jattach
        let jattach = self.find_jattach()?;
        
        let output = Command::new(&jattach)
            .arg(process.pid.to_string())
            .arg("load")
            .arg("instrument")
            .arg(format!("{}={}", agent_jar.display(), adapter.config.worker_threads))
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(NMLError::Other(format!("Agent attach failed: {}", error)));
        }
        
        Ok(())
    }
    
    /// Attach on macOS
    #[cfg(target_os = "macos")]
    async fn attach_macos(&self, process: &MinecraftProcess, agent_jar: &Path, adapter: &VersionAdapter) -> Result<()> {
        // Similar to Linux
        self.attach_linux(process, agent_jar, adapter).await
    }
    
    /// Add JVM argument
    pub async fn add_jvm_arg(&self, _process: &MinecraftProcess, arg: &str) -> Result<()> {
        tracing::debug!("Adding JVM arg: {}", arg);
        // Note: JVM args can only be added at startup, not runtime
        // This is a no-op for runtime, but useful for documentation
        Ok(())
    }
    
    /// Enable tick optimization
    pub async fn enable_tick_optimization(&self, process: &MinecraftProcess) -> Result<()> {
        self.send_command(process, "enable_tick_opt").await
    }
    
    /// Enable region scheduling
    pub async fn enable_region_scheduling(&self, process: &MinecraftProcess) -> Result<()> {
        self.send_command(process, "enable_region_sched").await
    }
    
    /// Enable sync points
    pub async fn enable_sync_points(&self, process: &MinecraftProcess) -> Result<()> {
        self.send_command(process, "enable_sync_points").await
    }
    
    /// Query performance stats
    pub async fn query_stats(&self, process: &MinecraftProcess) -> Result<super::PerformanceStats> {
        // Send query command to agent
        let response = self.send_query(process, "stats").await?;
        
        // Parse response
        let stats: super::PerformanceStats = serde_json::from_str(&response)?;
        
        Ok(stats)
    }
    
    /// Send command to agent
    async fn send_command(&self, process: &MinecraftProcess, command: &str) -> Result<()> {
        // Use agent communication channel (socket, file, etc.)
        tracing::debug!("Sending command to agent: {}", command);
        
        // TODO: Implement actual communication
        // For now, just log
        
        Ok(())
    }
    
    /// Send query to agent
    async fn send_query(&self, process: &MinecraftProcess, query: &str) -> Result<String> {
        tracing::debug!("Sending query to agent: {}", query);
        
        // TODO: Implement actual communication
        
        // Return dummy data for now
        Ok(r#"{"tps":20.0,"mspt":50.0,"chunk_load_time":100.0,"entity_count":1000,"thread_count":4,"optimized_chunks":50,"active_schedulers":2}"#.to_string())
    }
    
    /// Get MCJEBooster agent JAR path
    fn get_agent_jar_path(&self) -> Result<std::path::PathBuf> {
        let base_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| NMLError::Other("Cannot find executable directory".to_string()))?
            .to_path_buf();
        
        Ok(base_dir.join("MCJEBooster.jar"))
    }
    
    /// Find jattach tool
    fn find_jattach(&self) -> Result<std::path::PathBuf> {
        // Try bundled jattach first
        let bundled = std::env::current_exe()?
            .parent()
            .ok_or_else(|| NMLError::Other("Cannot find executable directory".to_string()))?
            .join("jattach");
        
        if bundled.exists() {
            return Ok(bundled);
        }
        
        // Try system jattach
        if cfg!(target_os = "windows") {
            let system = std::path::PathBuf::from("jattach.exe");
            if system.exists() {
                return Ok(system);
            }
        } else {
            let system = std::path::PathBuf::from("jattach");
            if system.exists() {
                return Ok(system);
            }
        }
        
        Err(NMLError::Other("jattach not found".to_string()))
    }
}

impl Default for JavaAgentInjector {
    fn default() -> Self {
        Self::new()
    }
}
