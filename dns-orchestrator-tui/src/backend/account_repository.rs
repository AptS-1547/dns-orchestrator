//! 账号仓库
//!
//! 使用 JSON 文件存储账号元数据
//! 实现 dns-orchestrator-core 的 AccountRepository trait

use async_trait::async_trait;
use dns_orchestrator_core::traits::AccountRepository;
use dns_orchestrator_core::types::{Account, AccountStatus};
use dns_orchestrator_core::{CoreError, CoreResult};
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::Mutex;

/// 获取配置目录路径
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dns-orchestrator-tui")
}

/// 获取账号数据文件路径
fn get_accounts_file() -> PathBuf {
    get_config_dir().join("accounts.json")
}

/// 基于 JSON 文件的账号仓库
pub struct JsonAccountRepository {
    /// 内存缓存
    cache: Mutex<Vec<Account>>,
}

impl JsonAccountRepository {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(Vec::new()),
        }
    }

    /// 确保配置目录存在
    async fn ensure_config_dir() -> CoreResult<()> {
        let dir = get_config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .await
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
        }
        Ok(())
    }

    /// 从文件加载账号列表
    async fn load_from_file(&self) -> CoreResult<Vec<Account>> {
        let path = get_accounts_file();

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let accounts: Vec<Account> = serde_json::from_str(&content)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        Ok(accounts)
    }

    /// 保存账号列表到文件
    async fn save_to_file(&self, accounts: &[Account]) -> CoreResult<()> {
        Self::ensure_config_dir().await?;

        let path = get_accounts_file();
        let content = serde_json::to_string_pretty(accounts)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        fs::write(&path, content)
            .await
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        Ok(())
    }
}

impl Default for JsonAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        // 尝试从缓存获取
        {
            let cache = self.cache.lock().await;
            if !cache.is_empty() {
                return Ok(cache.clone());
            }
        }

        // 从文件加载
        let accounts = self.load_from_file().await?;

        // 更新缓存
        *self.cache.lock().await = accounts.clone();

        Ok(accounts)
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        let accounts = self.find_all().await?;
        Ok(accounts.into_iter().find(|a| a.id == id))
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        let mut accounts = self.load_from_file().await?;

        // 查找是否已存在
        if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
            accounts[pos] = account.clone();
        } else {
            accounts.push(account.clone());
        }

        self.save_to_file(&accounts).await?;

        // 更新缓存
        *self.cache.lock().await = accounts;

        Ok(())
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        let mut accounts = self.load_from_file().await?;

        let original_len = accounts.len();
        accounts.retain(|a| a.id != id);

        if accounts.len() == original_len {
            return Err(CoreError::AccountNotFound(id.to_string()));
        }

        self.save_to_file(&accounts).await?;

        // 更新缓存
        *self.cache.lock().await = accounts;

        Ok(())
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        let mut existing = self.load_from_file().await?;

        for account in accounts {
            if let Some(pos) = existing.iter().position(|a| a.id == account.id) {
                existing[pos] = account.clone();
            } else {
                existing.push(account.clone());
            }
        }

        self.save_to_file(&existing).await?;

        // 更新缓存
        *self.cache.lock().await = existing;

        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let mut accounts = self.load_from_file().await?;

        if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
            account.status = Some(status);
            account.error = error;
            account.updated_at = chrono::Utc::now();

            self.save_to_file(&accounts).await?;

            // 更新缓存
            *self.cache.lock().await = accounts;

            Ok(())
        } else {
            Err(CoreError::AccountNotFound(id.to_string()))
        }
    }
}
