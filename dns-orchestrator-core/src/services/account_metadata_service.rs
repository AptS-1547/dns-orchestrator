//! 账户元数据服务
//!
//! 负责账户元数据的纯 CRUD 操作，不涉及凭证和 Provider 管理

use std::sync::Arc;

use crate::error::CoreResult;
use crate::traits::AccountRepository;
use crate::types::{Account, AccountStatus};

/// 账户元数据服务
pub struct AccountMetadataService {
    account_repository: Arc<dyn AccountRepository>,
}

impl AccountMetadataService {
    /// 创建账户元数据服务实例
    #[must_use]
    pub fn new(account_repository: Arc<dyn AccountRepository>) -> Self {
        Self { account_repository }
    }

    /// 列出所有账户
    pub async fn list_accounts(&self) -> CoreResult<Vec<Account>> {
        self.account_repository.find_all().await
    }

    /// 根据 ID 获取账户
    pub async fn get_account(&self, account_id: &str) -> CoreResult<Option<Account>> {
        self.account_repository.find_by_id(account_id).await
    }

    /// 保存账户（新增或更新）
    pub async fn save_account(&self, account: &Account) -> CoreResult<()> {
        self.account_repository.save(account).await
    }

    /// 删除账户元数据
    pub async fn delete_account(&self, account_id: &str) -> CoreResult<()> {
        self.account_repository.delete(account_id).await
    }

    /// 更新账户状态
    pub async fn update_status(
        &self,
        account_id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        self.account_repository
            .update_status(account_id, status, error)
            .await
    }
}
