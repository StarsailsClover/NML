//! Microsoft account provider
//!
//! Implements Xbox Live authentication flow

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::account::{Account, AccountProvider, AccountType, Credentials};
use crate::error::{NMLError, Result};

/// Microsoft OAuth configuration
const MICROSOFT_CLIENT_ID: &str = "00000000402b5328"; // Minecraft Launcher client ID
const MICROSOFT_AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const MICROSOFT_TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const XBOX_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XBOX_XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Microsoft OAuth tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MicrosoftTokens {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

/// Xbox Live tokens
#[derive(Debug, Clone)]
struct XboxTokens {
    token: String,
    user_hash: String,
}

/// XSTS tokens
#[derive(Debug, Clone)]
struct XSTSTokens {
    token: String,
    user_hash: String,
}

/// Minecraft auth response
#[derive(Debug, Deserialize)]
struct MinecraftAuthResponse {
    access_token: String,
    expires_in: i64,
}

/// Minecraft profile
#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<MinecraftSkin>,
    #[serde(default)]
    capes: Vec<MinecraftCape>,
}

#[derive(Debug, Deserialize)]
struct MinecraftSkin {
    id: String,
    state: String,
    url: String,
    #[serde(rename = "variant")]
    variant: String,
}

#[derive(Debug, Deserialize)]
struct MinecraftCape {
    id: String,
    state: String,
    url: String,
}

/// Microsoft account provider
pub struct MicrosoftProvider {
    client: reqwest::Client,
}

impl MicrosoftProvider {
    /// Create new Microsoft provider
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Get device code for OAuth flow
    pub async fn get_device_code(&self) -> Result<DeviceCodeResponse> {
        let params = [
            ("client_id", MICROSOFT_CLIENT_ID),
            ("scope", "XboxLive.signin offline_access"),
        ];

        let response = self.client
            .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("Failed to get device code".to_string()));
        }

        let device_code: DeviceCodeResponse = response.json().await?;
        Ok(device_code)
    }

    /// Poll for OAuth token
    async fn poll_for_token(&self, device_code: &str) -> Result<MicrosoftTokens> {
        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("client_id", MICROSOFT_CLIENT_ID),
            ("device_code", device_code),
        ];

        loop {
            let response = self.client
                .post(MICROSOFT_TOKEN_URL)
                .form(&params)
                .send()
                .await?;

            if response.status().is_success() {
                let token_response: MicrosoftTokenResponse = response.json().await?;
                return Ok(MicrosoftTokens {
                    access_token: token_response.access_token,
                    refresh_token: token_response.refresh_token,
                    expires_in: token_response.expires_in,
                });
            }

            // Check for specific error
            let error: OAuthErrorResponse = response.json().await?;
            match error.error.as_str() {
                "authorization_pending" => {
                    // User hasn't authorized yet, wait and retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }
                "authorization_declined" => {
                    return Err(NMLError::AccountError("Authorization declined by user".to_string()));
                }
                "expired_token" => {
                    return Err(NMLError::AccountError("Device code expired".to_string()));
                }
                _ => {
                    return Err(NMLError::AccountError(format!("OAuth error: {}", error.error)));
                }
            }
        }
    }

    /// Authenticate with Xbox Live
    async fn xbox_authenticate(&self, ms_access_token: &str) -> Result<XboxTokens> {
        let body = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", ms_access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let response = self.client
            .post(XBOX_AUTH_URL)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("Xbox authentication failed".to_string()));
        }

        let xbox_response: XboxAuthResponse = response.json().await?;
        
        Ok(XboxTokens {
            token: xbox_response.token,
            user_hash: xbox_response.display_claims.xui[0].uhs.clone(),
        })
    }

    /// Get XSTS token
    async fn get_xsts_token(&self, xbox_token: &str) -> Result<XSTSTokens> {
        let body = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbox_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let response = self.client
            .post(XBOX_XSTS_URL)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("XSTS authentication failed".to_string()));
        }

        let xsts_response: XSTSAuthResponse = response.json().await?;
        
        Ok(XSTSTokens {
            token: xsts_response.token,
            user_hash: xsts_response.display_claims.xui[0].uhs.clone(),
        })
    }

    /// Authenticate with Minecraft
    async fn minecraft_authenticate(&self, xsts_token: &str, user_hash: &str) -> Result<MinecraftAuthResponse> {
        let body = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
        });

        let response = self.client
            .post(MINECRAFT_AUTH_URL)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("Minecraft authentication failed".to_string()));
        }

        let mc_response: MinecraftAuthResponse = response.json().await?;
        Ok(mc_response)
    }

    /// Get Minecraft profile
    async fn get_minecraft_profile(&self, access_token: &str) -> Result<MinecraftProfile> {
        let response = self.client
            .get(MINECRAFT_PROFILE_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("Failed to get Minecraft profile".to_string()));
        }

        let profile: MinecraftProfile = response.json().await?;
        Ok(profile)
    }
}

#[async_trait]
impl AccountProvider for MicrosoftProvider {
    async fn authenticate(&self, _credentials: Credentials) -> Result<Account> {
        // For Microsoft auth, we use device code flow
        // This would be called after the user has authorized via browser
        Err(NMLError::AccountError("Use device code flow instead".to_string()))
    }

    async fn refresh(&self, account: &Account) -> Result<Account> {
        let refresh_token = account.refresh_token.as_ref()
            .ok_or_else(|| NMLError::AccountError("No refresh token".to_string()))?;

        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", MICROSOFT_CLIENT_ID),
            ("refresh_token", refresh_token.as_str()),
        ];

        let response = self.client
            .post(MICROSOFT_TOKEN_URL)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(NMLError::AccountError("Token refresh failed".to_string()));
        }

        let token_response: MicrosoftTokenResponse = response.json().await?;

        // Continue with Xbox/Minecraft auth...
        // (Simplified for brevity)

        let mut refreshed = account.clone();
        refreshed.access_token = Some(token_response.access_token);
        refreshed.refresh_token = Some(token_response.refresh_token);
        refreshed.expires_at = Some(Utc::now() + Duration::seconds(token_response.expires_in));

        Ok(refreshed)
    }

    async fn validate(&self, account: &Account) -> Result<bool> {
        if account.is_token_expired() {
            return Ok(false);
        }

        // Try to get profile to validate
        match &account.access_token {
            Some(token) => {
                let response = self.client
                    .get(MINECRAFT_PROFILE_URL)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await?;

                Ok(response.status().is_success())
            }
            None => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "Microsoft"
    }

    fn account_type(&self) -> AccountType {
        AccountType::Microsoft
    }
}

/// Device code response
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i64,
    pub interval: i64,
}

/// Microsoft token response
#[derive(Debug, Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

/// OAuth error response
#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: String,
    error_description: Option<String>,
}

/// Xbox auth response
#[derive(Debug, Deserialize)]
struct XboxAuthResponse {
    token: String,
    display_claims: XboxDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XboxDisplayClaims {
    xui: Vec<XboxXUI>,
}

#[derive(Debug, Deserialize)]
struct XboxXUI {
    uhs: String,
}

/// XSTS auth response
#[derive(Debug, Deserialize)]
struct XSTSAuthResponse {
    token: String,
    display_claims: XSTSDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XSTSDisplayClaims {
    xui: Vec<XSTSXUI>,
}

#[derive(Debug, Deserialize)]
struct XSTSXUI {
    uhs: String,
}

impl MicrosoftProvider {
    /// Complete authentication flow (for use from UI)
    pub async fn authenticate_with_device_code(&self) -> Result<Account> {
        // 1. Get device code
        let device_code = self.get_device_code().await?;

        // 2. Show user code to user (in UI)
        println!("Please go to: {}", device_code.verification_uri);
        println!("Enter code: {}", device_code.user_code);

        // 3. Poll for token
        let ms_tokens = self.poll_for_token(&device_code.device_code).await?;

        // 4. Xbox auth
        let xbox_tokens = self.xbox_authenticate(&ms_tokens.access_token).await?;

        // 5. XSTS auth
        let xsts_tokens = self.get_xsts_token(&xbox_tokens.token).await?;

        // 6. Minecraft auth
        let mc_auth = self.minecraft_authenticate(&xsts_tokens.token, &xsts_tokens.user_hash).await?;

        // 7. Get profile
        let profile = self.get_minecraft_profile(&mc_auth.access_token).await?;

        // 8. Create account
        let mut account = Account::new(
            AccountType::Microsoft,
            profile.name,
            profile.id,
        );

        account.access_token = Some(mc_auth.access_token);
        account.refresh_token = Some(ms_tokens.refresh_token);
        account.expires_at = Some(Utc::now() + Duration::seconds(mc_auth.expires_in));

        // Add skin URL if available
        if let Some(skin) = profile.skins.iter().find(|s| s.state == "ACTIVE") {
            account.metadata.insert("skin_url".to_string(), skin.url.clone());
        }

        // Add cape URL if available
        if let Some(cape) = profile.capes.iter().find(|c| c.state == "ACTIVE") {
            account.metadata.insert("cape_url".to_string(), cape.url.clone());
        }

        Ok(account)
    }
}
