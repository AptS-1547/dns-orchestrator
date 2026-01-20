//! 凭证存储服务
//!
//! 使用系统钥匙串安全存储 DNS 服务商凭证
//! 实现 dns-orchestrator-core 的 CredentialStore trait

use async_trait::async_trait;
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap};
use dns_orchestrator_core::{CoreError, CoreResult};
use dns_orchestrator_provider::ProviderCredentials;
use keyring::Entry;
use tokio::sync::Mutex;

const SERVICE_NAME: &str = "dns-orchestrator-tui";
const CREDENTIALS_KEY: &str = "__all_credentials__";

/// 基于系统钥匙串的凭证存储
///
/// 使用 keyring crate 将凭证安全存储到：
/// - Windows: Credential Manager
/// - macOS: Keychain
/// - Linux: Secret Service (GNOME Keyring / KWallet)
pub struct KeyringCredentialStore {
    /// 内存缓存，避免频繁访问钥匙串
    cache: Mutex<CredentialsMap>,
}

impl KeyringCredentialStore {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(CredentialsMap::new()),
        }
    }
}

impl Default for KeyringCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CredentialStore for KeyringCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        // 尝试从缓存返回
        {
            let cache = self.cache.lock().await;
            if !cache.is_empty() {
                return Ok(cache.clone());
            }
        }

        // 从钥匙串加载
        let json = self.load_raw_json().await?;
        if json.is_empty() {
            return Ok(CredentialsMap::new());
        }

        let credentials: CredentialsMap = serde_json::from_str(&json)
            .map_err(|e| CoreError::CredentialError(format!("Failed to deserialize: {e}")))?;

        // 更新缓存
        *self.cache.lock().await = credentials.clone();

        Ok(credentials)
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        let json = serde_json::to_string(credentials)
            .map_err(|e| CoreError::CredentialError(format!("Failed to serialize: {e}")))?;

        self.save_raw_json(&json).await?;

        // 更新缓存
        *self.cache.lock().await = credentials.clone();

        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        // 先检查缓存
        if let Some(creds) = self.cache.lock().await.get(account_id) {
            return Ok(Some(creds.clone()));
        }

        // 从存储加载全部并检查
        let all = self.load_all().await?;
        Ok(all.get(account_id).cloned())
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        let mut all = self.load_all().await?;
        all.insert(account_id.to_string(), credentials.clone());
        self.save_all(&all).await
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        let mut all = self.load_all().await?;
        all.remove(account_id);
        self.save_all(&all).await
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| CoreError::CredentialError(format!("Failed to create entry: {e}")))?;

        match entry.get_password() {
            Ok(json) => Ok(json),
            Err(keyring::Error::NoEntry) => Ok(String::new()),
            Err(e) => Err(CoreError::CredentialError(format!(
                "Failed to load: {e}"
            ))),
        }
    }

    async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
        let entry = Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| CoreError::CredentialError(format!("Failed to create entry: {e}")))?;

        entry
            .set_password(json)
            .map_err(|e| CoreError::CredentialError(format!("Failed to save: {e}")))?;

        Ok(())
    }
}
