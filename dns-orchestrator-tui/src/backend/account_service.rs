//! 账号服务

use anyhow::Result;
use async_trait::async_trait;

use crate::model::domain::{Account, AccountStatus, ProviderType};

/// 账号服务 trait
#[async_trait]
pub trait AccountService: Send + Sync {
    /// 获取所有账号
    async fn list_accounts(&self) -> Result<Vec<Account>>;

    /// 创建账号
    async fn create_account(&self, name: String, provider: ProviderType) -> Result<Account>;

    /// 删除账号
    async fn delete_account(&self, id: &str) -> Result<()>;

    /// 验证账号凭证
    async fn verify_account(&self, id: &str) -> Result<bool>;
}

/// Mock 账号服务（用于开发测试）
pub struct MockAccountService {
    accounts: Vec<Account>,
}

impl MockAccountService {
    pub fn new() -> Self {
        Self {
            accounts: vec![
                Account {
                    id: "1".to_string(),
                    name: "Production CF".to_string(),
                    provider: ProviderType::Cloudflare,
                    status: Some(AccountStatus::Active),
                    error: None,
                    created_at: "2024-01-01".to_string(),
                    updated_at: "2024-01-01".to_string(),
                },
                Account {
                    id: "2".to_string(),
                    name: "Aliyun DNS".to_string(),
                    provider: ProviderType::Aliyun,
                    status: Some(AccountStatus::Active),
                    error: None,
                    created_at: "2024-02-01".to_string(),
                    updated_at: "2024-02-01".to_string(),
                },
            ],
        }
    }
}

impl Default for MockAccountService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountService for MockAccountService {
    async fn list_accounts(&self) -> Result<Vec<Account>> {
        Ok(self.accounts.clone())
    }

    async fn create_account(&self, _name: String, _provider: ProviderType) -> Result<Account> {
        // Mock 实现
        todo!("Mock create_account")
    }

    async fn delete_account(&self, _id: &str) -> Result<()> {
        // Mock 实现
        todo!("Mock delete_account")
    }

    async fn verify_account(&self, _id: &str) -> Result<bool> {
        Ok(true)
    }
}