//! Account management for NML
//!
//! Supports multiple authentication methods:
//! - Microsoft OAuth (Online)
//! - Offline (Local mode)
//! - Third-party (Authlib-Injector)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{NMLError, Result};

pub mod microsoft;
pub mod offline;
pub mod third_party;

use microsoft::MicrosoftProvider;
use offline::OfflineProvider;
use third_party::ThirdPartyProvider;

/// Account type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// Microsoft account (Xbox Live)
    Microsoft,
    /// Offline/local account
    Offline,
    /// Third-party authentication server
    ThirdParty,
}

impl AccountType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AccountType::Microsoft => "微软账户",
            AccountType::Offline => "离线账户",
            AccountType::ThirdParty => "第三方账户",
        }
    }
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account ID
    pub id: String,
    /// Account type
    pub account_type: AccountType,
    /// Player name
    pub player_name: String,
    /// Player UUID
    pub uuid: String,
    /// Access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Refresh token (for Microsoft)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token expiration time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Account metadata
    pub metadata: HashMap<String, String>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last used
    pub last_used: DateTime<Utc>,
    /// Is selected as default
    pub is_selected: bool,
}

impl Account {
    /// Create new account
    pub fn new(
        account_type: AccountType,
        player_name: String,
        uuid: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            account_type,
            player_name,
            uuid,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            last_used: Utc::now(),
            is_selected: false,
        }
    }

    /// Check if token is expired
    pub fn is_token_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() >= expires,
            None => true,
        }
    }

    /// Check if account is valid
    pub fn is_valid(&self) -> bool {
        !self.player_name.is_empty() && !self.uuid.is_empty()
    }

    /// Update last used time
    pub fn touch(&mut self) {
        self.last_used = Utc::now();
    }

    /// Get skin URL
    pub fn skin_url(&self) -> Option<String> {
        self.metadata.get("skin_url").cloned()
    }

    /// Get cape URL
    pub fn cape_url(&self) -> Option<String> {
        self.metadata.get("cape_url").cloned()
    }
}

/// Credentials for authentication
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Username or email
    pub username: String,
    /// Password (if applicable)
    pub password: Option<String>,
    /// Authorization code (OAuth)
    pub auth_code: Option<String>,
    /// Third-party server URL
    pub server_url: Option<String>,
}

impl Credentials {
    /// Create Microsoft credentials
    pub fn microsoft(auth_code: String) -> Self {
        Self {
            username: String::new(),
            password: None,
            auth_code: Some(auth_code),
            server_url: None,
        }
    }

    /// Create offline credentials
    pub fn offline(username: String) -> Self {
        Self {
            username,
            password: None,
            auth_code: None,
            server_url: None,
        }
    }

    /// Create third-party credentials
    pub fn third_party(server_url: String, username: String, password: String) -> Self {
        Self {
            username,
            password: Some(password),
            auth_code: None,
            server_url: Some(server_url),
        }
    }
}

/// Account provider trait
#[async_trait]
pub trait AccountProvider: Send + Sync {
    /// Authenticate and create account
    async fn authenticate(&self, credentials: Credentials) -> Result<Account>;
    
    /// Refresh account token
    async fn refresh(&self, account: &Account) -> Result<Account>;
    
    /// Validate account
    async fn validate(&self, account: &Account) -> Result<bool>;
    
    /// Get provider name
    fn name(&self) -> &'static str;
    
    /// Get provider type
    fn account_type(&self) -> AccountType;
}

/// Account manager
pub struct AccountManager {
    providers: HashMap<AccountType, Box<dyn AccountProvider>>,
    accounts: Vec<Account>,
    data_dir: std::path::PathBuf,
}

impl AccountManager {
    /// Create new account manager
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let mut providers: HashMap<AccountType, Box<dyn AccountProvider>> = HashMap::new();
        
        // Register providers
        providers.insert(AccountType::Microsoft, Box::new(MicrosoftProvider::new()));
        providers.insert(AccountType::Offline, Box::new(OfflineProvider::new()));
        providers.insert(AccountType::ThirdParty, Box::new(ThirdPartyProvider::new()));
        
        Self {
            providers,
            accounts: Vec::new(),
            data_dir,
        }
    }

    /// Load accounts from storage
    pub async fn load(&mut self) -> Result<()> {
        let accounts_file = self.data_dir.join("accounts.json");
        
        if accounts_file.exists() {
            let content = tokio::fs::read_to_string(&accounts_file).await?;
            self.accounts = serde_json::from_str(&content)?;
        }
        
        Ok(())
    }

    /// Save accounts to storage
    pub async fn save(&self) -> Result<()> {
        let accounts_file = self.data_dir.join("accounts.json");
        
        std::fs::create_dir_all(&self.data_dir)?;
        let content = serde_json::to_string_pretty(&self.accounts)?;
        tokio::fs::write(&accounts_file, content).await?;
        
        Ok(())
    }

    /// Add new account
    pub async fn add_account(&mut self, account: Account) -> Result<()> {
        // Check if account already exists
        if self.accounts.iter().any(|a| a.uuid == account.uuid) {
            return Err(NMLError::AccountError("Account already exists".to_string()));
        }
        
        self.accounts.push(account);
        self.save().await?;
        
        Ok(())
    }

    /// Remove account
    pub async fn remove_account(&mut self, account_id: &str) -> Result<()> {
        let index = self.accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| NMLError::AccountError("Account not found".to_string()))?;
        
        self.accounts.remove(index);
        self.save().await?;
        
        Ok(())
    }

    /// Get all accounts
    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// Get account by ID
    pub fn get_account(&self, account_id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    /// Get selected account
    pub fn get_selected_account(&self) -> Option<&Account> {
        self.accounts.iter().find(|a| a.is_selected)
    }

    /// Set selected account
    pub async fn set_selected_account(&mut self, account_id: &str) -> Result<()> {
        // Clear previous selection
        for account in &mut self.accounts {
            account.is_selected = false;
        }
        
        // Set new selection
        let account = self.accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| NMLError::AccountError("Account not found".to_string()))?;
        
        account.is_selected = true;
        account.touch();
        
        self.save().await?;
        Ok(())
    }

    /// Authenticate with credentials
    pub async fn authenticate(&self, account_type: AccountType, credentials: Credentials) -> Result<Account> {
        let provider = self.providers
            .get(&account_type)
            .ok_or_else(|| NMLError::AccountError("Provider not found".to_string()))?;
        
        let account = provider.authenticate(credentials).await?;
        Ok(account)
    }

    /// Refresh account
    pub async fn refresh_account(&self, account: &Account) -> Result<Account> {
        let provider = self.providers
            .get(&account.account_type)
            .ok_or_else(|| NMLError::AccountError("Provider not found".to_string()))?;
        
        let refreshed = provider.refresh(account).await?;
        Ok(refreshed)
    }

    /// Validate account
    pub async fn validate_account(&self, account: &Account) -> Result<bool> {
        let provider = self.providers
            .get(&account.account_type)
            .ok_or_else(|| NMLError::AccountError("Provider not found".to_string()))?;
        
        provider.validate(account).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new(
            AccountType::Offline,
            "TestPlayer".to_string(),
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        );
        
        assert_eq!(account.account_type, AccountType::Offline);
        assert_eq!(account.player_name, "TestPlayer");
        assert!(!account.is_token_expired());
    }

    #[test]
    fn test_offline_credentials() {
        let creds = Credentials::offline("Player".to_string());
        assert_eq!(creds.username, "Player");
        assert!(creds.password.is_none());
    }
}
