//! 账户持久化抽象 Trait

use async_trait::async_trait;

use crate::error::CoreResult;
use crate::types::{Account, AccountStatus};

/// 账户元数据仓库 Trait
///
/// 平台实现:
/// - Tauri: `TauriAccountRepository` (tauri-plugin-store)
/// - Actix-Web: `DatabaseAccountRepository` (`SeaORM`)
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// 获取所有账户
    async fn find_all(&self) -> CoreResult<Vec<Account>>;

    /// 根据 ID 获取账户
    ///
    /// # Arguments
    /// * `id` - 账户 ID
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;

    /// 保存账户 (新增或更新)
    ///
    /// # Arguments
    /// * `account` - 账户数据
    async fn save(&self, account: &Account) -> CoreResult<()>;

    /// 删除账户
    ///
    /// # Arguments
    /// * `id` - 账户 ID
    async fn delete(&self, id: &str) -> CoreResult<()>;

    /// 批量保存账户 (用于导入)
    ///
    /// # Arguments
    /// * `accounts` - 账户列表
    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()>;

    /// 更新账户状态
    ///
    /// # Arguments
    /// * `id` - 账户 ID
    /// * `status` - 新状态
    /// * `error` - 错误信息（如果状态为 Error）
    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()>;
}
