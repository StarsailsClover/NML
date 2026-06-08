//! Offline account provider
//!
//! For local play without authentication

use async_trait::async_trait;

use crate::account::{Account, AccountProvider, AccountType, Credentials};
use crate::error::{NMLError, Result};

/// Offline account provider
pub struct OfflineProvider;

impl OfflineProvider {
    /// Create new offline provider
    pub fn new() -> Self {
        Self
    }

    /// Generate offline UUID from username
    fn generate_uuid(&self, username: &str) -> String {
        // Offline UUID v3 (MD5 based)
        use md5::{Digest, Md5};
        
        let mut hasher = Md5::new();
        hasher.update(format!("OfflinePlayer:{}", username).as_bytes());
        let result = hasher.finalize();
        
        // Format as UUID v3
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            result[0], result[1], result[2], result[3],
            result[4], result[5],
            result[6], result[7],
            result[8], result[9],
            result[10], result[11], result[12], result[13], result[14], result[15]
        )
    }
}

#[async_trait]
impl AccountProvider for OfflineProvider {
    async fn authenticate(&self, credentials: Credentials) -> Result<Account> {
        let username = credentials.username;
        
        if username.is_empty() {
            return Err(NMLError::AccountError("Username cannot be empty".to_string()));
        }
        
        // Validate username
        if !self.is_valid_username(&username) {
            return Err(NMLError::AccountError(
                "Invalid username. Must be 3-16 alphanumeric characters".to_string()
            ));
        }
        
        let uuid = self.generate_uuid(&username);
        let account = Account::new(AccountType::Offline, username, uuid);
        
        Ok(account)
    }

    async fn refresh(&self, account: &Account) -> Result<Account> {
        // Offline accounts don't need refresh
        Ok(account.clone())
    }

    async fn validate(&self, _account: &Account) -> Result<bool> {
        // Offline accounts are always valid
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "Offline"
    }

    fn account_type(&self) -> AccountType {
        AccountType::Offline
    }
}

impl OfflineProvider {
    /// Check if username is valid
    fn is_valid_username(&self, username: &str) -> bool {
        if username.len() < 3 || username.len() > 16 {
            return false;
        }
        
        username.chars().all(|c| {
            c.is_alphanumeric() || c == '_'
        })
    }
}
