//! 凭证存储抽象 Trait

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::CoreResult;

/// 凭证映射类型：`account_id` -> credentials
pub type CredentialsMap = HashMap<String, HashMap<String, String>>;

/// 凭证存储 Trait
///
/// 平台实现:
/// - Tauri Desktop: `KeychainStore` (keyring crate)
/// - Tauri Android: `StrongholdStore`
/// - Actix-Web: `DatabaseCredentialStore` (`SeaORM` + AES 加密)
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// 一次性加载所有凭证
    ///
    /// 启动时使用，减少存储访问次数
    async fn load_all(&self) -> CoreResult<CredentialsMap>;

    /// 保存凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    /// * `credentials` - 凭证键值对
    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>)
        -> CoreResult<()>;

    /// 加载单个账户凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>>;

    /// 删除凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn delete(&self, account_id: &str) -> CoreResult<()>;

    /// 检查凭证是否存在
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn exists(&self, account_id: &str) -> bool;
}
