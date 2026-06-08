//! Sandbox launcher for isolated testing
//!
//! Launches Minecraft in isolated environment, captures errors,
//! and auto-releases to normal process on success

use std::process::Command;
use std::path::PathBuf;
use std::time::Duration;

use crate::error::{NMLError, Result};
use crate::launch::{MinecraftProcess, LaunchConfig};
use crate::error_analyzer::{ErrorAnalyzer, AnalysisResult};

pub mod isolation;
pub mod monitor;

use isolation::IsolationEnvironment;
use monitor::SandboxMonitor;

/// Sandbox launcher
pub struct SandboxLauncher {
    config: LaunchConfig,
    isolation: IsolationEnvironment,
    analyzer: ErrorAnalyzer,
    monitor: SandboxMonitor,
}

/// Sandbox result
#[derive(Debug, Clone)]
pub enum SandboxResult {
    Success {
        process: MinecraftProcess,
        duration: Duration,
    },
    Failed {
        analysis: AnalysisResult,
        auto_fixed: bool,
        can_retry: bool,
    },
    NeedsRetry {
        fixes: Vec<String>,
    },
}

impl SandboxLauncher {
    /// Create new sandbox launcher
    pub fn new(config: LaunchConfig) -> Self {
        Self {
            config,
            isolation: IsolationEnvironment::new(),
            analyzer: ErrorAnalyzer::new(),
            monitor: SandboxMonitor::new(),
        }
    }
    
    /// Launch in sandbox
    pub async fn launch(&self) -> Result<SandboxResult> {
        tracing::info!("Launching Minecraft in sandbox");
        
        // 1. Create isolated environment
        let isolated_env = self.isolation.create().await?;
        
        // 2. Launch process in sandbox
        let start_time = std::time::Instant::now();
        let mut process = self.launch_isolated(&isolated_env).await?;
        
        // 3. Monitor for errors
        let monitor_result = self.monitor.monitor(&mut process, Duration::from_secs(30)).await?;
        
        match monitor_result {
            MonitorResult::Success => {
                // Sandbox succeeded, release to normal
                let duration = start_time.elapsed();
                tracing::info!("Sandbox success, releasing to normal process");
                
                let normal_process = self.release_to_normal(process).await?;
                
                Ok(SandboxResult::Success {
                    process: normal_process,
                    duration,
                })
            }
            MonitorResult::Failed { log_path } => {
                // Analyze error
                let analysis = self.analyzer.analyze(&log_path).await?;
                
                // Try auto-fix
                let auto_fixed = self.analyzer.auto_fix(&analysis).await?;
                
                // Determine if can retry
                let can_retry = analysis.fixes.iter().any(|f| f.auto_fixable);
                
                // Kill sandbox process
                let _ = process.kill().await;
                
                if can_retry && auto_fixed {
                    Ok(SandboxResult::NeedsRetry {
                        fixes: analysis.fixes.iter()
                            .filter(|f| f.auto_fixable)
                            .map(|f| f.description.clone())
                            .collect(),
                    })
                } else {
                    Ok(SandboxResult::Failed {
                        analysis,
                        auto_fixed,
                        can_retry,
                    })
                }
            }
            MonitorResult::Timeout => {
                // Timeout, assume success and release
                let normal_process = self.release_to_normal(process).await?;
                
                Ok(SandboxResult::Success {
                    process: normal_process,
                    duration: start_time.elapsed(),
                })
            }
        }
    }
    
    /// Launch in isolated environment
    async fn launch_isolated(&self, env: &IsolationEnvironment) -> Result<MinecraftProcess> {
        // Build isolated command
        let mut cmd = Command::new(&self.config.java_path);
        
        // Apply isolation
        env.apply_to_command(&mut cmd);
        
        // Apply JVM args
        for arg in &self.config.jvm_args {
            cmd.arg(arg);
        }
        
        // Launch
        let child = cmd.spawn()?;
        
        Ok(MinecraftProcess {
            pid: child.id(),
            child: std::sync::Arc::new(tokio::sync::Mutex::new(child)),
            version_id: self.config.version_id.clone(),
        })
    }
    
    /// Release to normal process
    async fn release_to_normal(&self, _sandbox_process: MinecraftProcess) -> Result<MinecraftProcess> {
        // For now, just return the same process
        // In real implementation, this would:
        // 1. Remove isolation constraints
        // 2. Restore network access
        // 3. etc.
        
        tracing::info!("Released to normal process");
        
        // Actually launch a new normal process
        let mut cmd = Command::new(&self.config.java_path);
        
        for arg in &self.config.jvm_args {
            cmd.arg(arg);
        }
        
        let child = cmd.spawn()?;
        
        Ok(MinecraftProcess {
            pid: child.id(),
            child: std::sync::Arc::new(tokio::sync::Mutex::new(child)),
            version_id: self.config.version_id.clone(),
        })
    }
}

/// Monitor result
#[derive(Debug, Clone)]
pub enum MonitorResult {
    Success,
    Failed { log_path: PathBuf },
    Timeout,
}
