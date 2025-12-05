use keyring::Entry;
use std::collections::HashMap;

use super::{CredentialStore, CredentialsMap};
use crate::error::{DnsError, Result};

const SERVICE_NAME: &str = "dns-orchestrator";
const CREDENTIALS_KEY: &str = "all-credentials";

/// 系统 Keychain 凭证存储实现
///
/// 使用单个 Keychain 条目存储所有账户凭证，避免多次 Keychain 访问
pub struct KeychainStore;

impl KeychainStore {
    pub fn new() -> Self {
        Self
    }

    /// 获取 Keychain Entry
    fn get_entry() -> Result<Entry> {
        Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| DnsError::CredentialError(e.to_string()))
    }

    /// 从 Keychain 读取整个凭证存储
    fn read_all_internal(&self) -> Result<CredentialsMap> {
        let entry = Self::get_entry()?;

        match entry.get_password() {
            Ok(json) => {
                serde_json::from_str(&json).map_err(|e| DnsError::SerializationError(e.to_string()))
            }
            Err(keyring::Error::NoEntry) => {
                // 没有存储的凭证，返回空 map
                Ok(HashMap::new())
            }
            Err(e) => Err(DnsError::CredentialError(e.to_string())),
        }
    }

    /// 将整个凭证存储写入 Keychain
    fn write_all_internal(&self, credentials: &CredentialsMap) -> Result<()> {
        let entry = Self::get_entry()?;

        let json = serde_json::to_string(credentials)
            .map_err(|e| DnsError::SerializationError(e.to_string()))?;

        entry
            .set_password(&json)
            .map_err(|e| DnsError::CredentialError(e.to_string()))?;

        Ok(())
    }
}

impl Default for KeychainStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for KeychainStore {
    fn load_all(&self) -> Result<CredentialsMap> {
        log::debug!("Loading all credentials from Keychain");
        let credentials = self.read_all_internal()?;
        log::info!("Loaded {} accounts from Keychain", credentials.len());
        Ok(credentials)
    }

    fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> Result<()> {
        log::debug!("Saving credentials for account: {}", account_id);

        // 读取现有凭证
        let mut all_credentials = self.read_all_internal()?;

        // 更新指定账户的凭证
        all_credentials.insert(account_id.to_string(), credentials.clone());

        // 写回 Keychain
        self.write_all_internal(&all_credentials)?;

        log::info!("Credentials saved for account: {}", account_id);
        Ok(())
    }

    fn load(&self, account_id: &str) -> Result<HashMap<String, String>> {
        let all_credentials = self.read_all_internal()?;

        all_credentials.get(account_id).cloned().ok_or_else(|| {
            DnsError::CredentialError(format!("No credentials found for account: {}", account_id))
        })
    }

    fn delete(&self, account_id: &str) -> Result<()> {
        log::debug!("Deleting credentials for account: {}", account_id);

        // 读取现有凭证
        let mut all_credentials = self.read_all_internal()?;

        // 删除指定账户的凭证
        all_credentials.remove(account_id);

        // 写回 Keychain
        self.write_all_internal(&all_credentials)?;

        log::info!("Credentials deleted for account: {}", account_id);
        Ok(())
    }

    fn exists(&self, account_id: &str) -> bool {
        self.read_all_internal()
            .map(|creds| creds.contains_key(account_id))
            .unwrap_or(false)
    }
}
