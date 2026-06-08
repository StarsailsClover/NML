//! Hot swap coordinator for identity switching without restart

use std::process::Child;
use crate::account::{Account, AccountType};
use crate::error::{NMLError, Result};
use crate::launch::MinecraftProcess;

pub mod agent;

/// Identity information for hot swap
#[derive(Debug, Clone)]
pub struct Identity {
    pub username: String,
    pub uuid: String,
    pub account_type: AccountType,
    pub access_token: Option<String>,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
}

/// Hot swap coordinator
pub struct HotSwapCoordinator {
    process_id: u32,
    current_identity: Option<Identity>,
    agent_attached: bool,
}

impl HotSwapCoordinator {
    pub fn for_process(process: &MinecraftProcess) -> Self {
        Self {
            process_id: process.pid,
            current_identity: None,
            agent_attached: false,
        }
    }
    
    /// Initialize agent in Minecraft JVM
    pub async fn initialize(&mut self, process: &MinecraftProcess) -> Result<()> {
        if self.agent_attached {
            return Ok(());
        }
        
        // Attach Java Agent via MCJEBooster integration
        agent::attach(process.pid).await?;
        
        self.agent_attached = true;
        self.process_id = process.pid;
        
        tracing::info!("Hot swap agent attached to PID {}", process.pid);
        Ok(())
    }
    
    /// Swap to new identity
    pub async fn swap_identity(&mut self, new_identity: Identity) -> Result<()> {
        if !self.agent_attached {
            return Err(NMLError::Other("Agent not attached".to_string()));
        }
        
        // Send swap command to agent
        agent::send_swap_command(self.process_id, &new_identity).await?;
        
        // Wait for confirmation
        agent::wait_for_confirmation(self.process_id, Duration::from_secs(5)).await?;
        
        self.current_identity = Some(new_identity);
        
        Ok(())
    }
    
    /// Swap to offline mode
    pub async fn swap_to_offline(&mut self, username: &str) -> Result<()> {
        let uuid = generate_offline_uuid(username);
        let identity = Identity {
            username: username.to_string(),
            uuid,
            account_type: AccountType::Offline,
            access_token: None,
            skin_url: None,
            cape_url: None,
        };
        
        self.swap_identity(identity).await
    }
    
    /// Swap to Microsoft account
    pub async fn swap_to_microsoft(&mut self, account: &Account) -> Result<()> {
        let identity = Identity {
            username: account.player_name.clone(),
            uuid: account.uuid.clone(),
            account_type: AccountType::Microsoft,
            access_token: account.access_token.clone(),
            skin_url: account.metadata.get("skin_url").cloned(),
            cape_url: account.metadata.get("cape_url").cloned(),
        };
        
        self.swap_identity(identity).await
    }
}

fn generate_offline_uuid(username: &str) -> String {
    use md5::{Md5, Digest};
    
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", username).as_bytes());
    let result = hasher.finalize();
    
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        result[0], result[1], result[2], result[3],
        result[4], result[5], result[6], result[7],
        result[8], result[9], result[10], result[11],
        result[12], result[13], result[14], result[15]
    )
}
