//! Third-party account provider (Authlib-Injector)

use async_trait::async_trait;
use crate::account::{Account, AccountProvider, AccountType, Credentials};
use crate::error::{NMLError, Result};

pub struct ThirdPartyProvider;

impl ThirdPartyProvider {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl AccountProvider for ThirdPartyProvider {
    async fn authenticate(&self, _credentials: Credentials) -> Result<Account> {
        Err(NMLError::AccountError("Third-party not yet implemented".to_string()))
    }
    async fn refresh(&self, account: &Account) -> Result<Account> {
        Ok(account.clone())
    }
    async fn validate(&self, _account: &Account) -> Result<bool> {
        Ok(false)
    }
    fn name(&self) -> &'static str { "ThirdParty" }
    fn account_type(&self) -> AccountType { AccountType::ThirdParty }
}
