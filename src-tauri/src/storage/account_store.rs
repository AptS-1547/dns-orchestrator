use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::error::{DnsError, Result};
use crate::types::Account;

const STORE_FILE_NAME: &str = "accounts.json";
const ACCOUNTS_KEY: &str = "accounts";

/// 账户元数据存储
///
/// 负责账户元数据的持久化，使用 Tauri Store 插件。
/// 敏感凭证仍然由 `KeychainStore` 单独管理。
pub struct AccountStore;

impl AccountStore {
    /// 保存所有账户元数据到持久化存储
    pub fn save_accounts(app: &AppHandle, accounts: &[Account]) -> Result<()> {
        let store = app
            .store(STORE_FILE_NAME)
            .map_err(|e| DnsError::SerializationError(format!("Failed to access store: {e}")))?;

        // 将账户列表序列化为 JSON
        let accounts_json = serde_json::to_value(accounts)
            .map_err(|e| DnsError::SerializationError(e.to_string()))?;

        // 保存到 store
        store.set(ACCOUNTS_KEY.to_string(), accounts_json);

        // 立即持久化到磁盘
        store
            .save()
            .map_err(|e| DnsError::SerializationError(format!("Failed to save store: {e}")))?;

        log::info!("Saved {} accounts to store", accounts.len());
        Ok(())
    }

    /// 从持久化存储加载所有账户元数据
    pub fn load_accounts(app: &AppHandle) -> Result<Vec<Account>> {
        let store = app
            .store(STORE_FILE_NAME)
            .map_err(|e| DnsError::SerializationError(format!("Failed to access store: {e}")))?;

        // 从 store 获取账户数据
        let accounts_value = if let Some(value) = store.get(ACCOUNTS_KEY) { value } else {
            log::info!("No accounts found in store, returning empty list");
            return Ok(Vec::new());
        };

        // 反序列化
        let accounts: Vec<Account> = serde_json::from_value(accounts_value.clone())
            .map_err(|e| DnsError::SerializationError(e.to_string()))?;

        log::info!("Loaded {} accounts from store", accounts.len());
        Ok(accounts)
    }

    /// 删除单个账户的元数据
    ///
    /// 实际上是保存更新后的账户列表（已移除指定账户）
    pub fn delete_account(app: &AppHandle, account_id: &str, accounts: &[Account]) -> Result<()> {
        Self::save_accounts(app, accounts)?;
        log::info!("Deleted account {account_id} from store");
        Ok(())
    }

    /// 清空所有账户元数据（用于测试或重置）
    #[allow(dead_code)]
    pub fn clear_all(app: &AppHandle) -> Result<()> {
        Self::save_accounts(app, &[])?;
        log::info!("Cleared all accounts from store");
        Ok(())
    }
}
