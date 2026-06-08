//! Network Hook for 1.16-1.16.5 offline multiplayer fix
//! 
//! This module provides elegant API hooking to enable multiplayer
//! in offline mode without disconnecting internet

use std::process::Child;
use crate::error::{NMLError, Result};

pub mod windows;
pub mod linux;
pub mod macos;

/// Hook installer for network detection APIs
pub struct NetworkHook {
    process_id: u32,
    installed: bool,
}

impl NetworkHook {
    /// Create new hook for Minecraft process
    pub fn for_process(process: &Child) -> Self {
        Self {
            process_id: process.id(),
            installed: false,
        }
    }
    
    /// Install hook (disable network detection)
    pub async fn install(&mut self) -> Result<()> {
        if self.installed {
            return Ok(());
        }
        
        #[cfg(target_os = "windows")]
        windows::install_hook(self.process_id).await?;
        
        #[cfg(target_os = "linux")]
        linux::install_hook(self.process_id).await?;
        
        #[cfg(target_os = "macos")]
        macos::install_hook(self.process_id).await?;
        
        self.installed = true;
        tracing::info!("Network hook installed for PID {}", self.process_id);
        Ok(())
    }
    
    /// Uninstall hook (restore network detection)
    pub async fn uninstall(&mut self) -> Result<()> {
        if !self.installed {
            return Ok(());
        }
        
        #[cfg(target_os = "windows")]
        windows::uninstall_hook(self.process_id).await?;
        
        #[cfg(target_os = "linux")]
        linux::uninstall_hook(self.process_id).await?;
        
        #[cfg(target_os = "macos")]
        macos::uninstall_hook(self.process_id).await?;
        
        self.installed = false;
        tracing::info!("Network hook uninstalled for PID {}", self.process_id);
        Ok(())
    }
    
    /// Check if hook is installed
    pub fn is_installed(&self) -> bool {
        self.installed
    }
}

/// Get hook DLL path for current platform
pub fn get_hook_dll_path() -> std::path::PathBuf {
    let base_dir = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    
    #[cfg(target_os = "windows")]
    return base_dir.join("nml_hook.dll");
    
    #[cfg(target_os = "linux")]
    return base_dir.join("libnml_hook.so");
    
    #[cfg(target_os = "macos")]
    return base_dir.join("libnml_hook.dylib");
}
