//! 凭证存储抽象 Trait

use async_trait::async_trait;
use dns_orchestrator_provider::ProviderCredentials;
use std::collections::HashMap;

use crate::error::CoreResult;

/// 凭证映射类型：`account_id` -> ProviderCredentials（类型安全）
pub type CredentialsMap = HashMap<String, ProviderCredentials>;

/// 旧格式凭证映射（迁移期间使用）
pub type LegacyCredentialsMap = HashMap<String, HashMap<String, String>>;

/// 凭证存储 Trait
///
/// 平台实现:
/// - Tauri Desktop: `TauriCredentialStore` (keyring crate)
/// - Tauri Android: `TauriCredentialStore` (tauri-plugin-store)
/// - Actix-Web: `DatabaseCredentialStore` (`SeaORM` + AES 加密)
///
/// # v1.7.0 变更
///
/// - 使用类型安全的 `ProviderCredentials` 替代 `HashMap<String, String>`
/// - 方法重命名以符合 Rust 惯用法：`load()` → `get()`, `save()` → `set()`, `delete()` → `remove()`
/// - 新增 `save_all()` 用于批量保存（迁移场景）
/// - 新增 `load_raw_json()` 和 `save_raw_json()` 用于迁移检测
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// 加载所有凭证（新格式）
    ///
    /// 启动时使用，减少存储访问次数。返回类型安全的 `CredentialsMap`。
    async fn load_all(&self) -> CoreResult<CredentialsMap>;

    /// 批量保存凭证（新格式）
    ///
    /// 用于迁移场景，一次性写入所有凭证。
    ///
    /// # Arguments
    /// * `credentials` - 凭证映射
    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()>;

    /// 获取单个账户凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    ///
    /// # Returns
    /// * `Ok(Some(credentials))` - 凭证存在
    /// * `Ok(None)` - 凭证不存在
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>>;

    /// 设置单个账户凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    /// * `credentials` - 类型安全的凭证
    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()>;

    /// 删除凭证
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn remove(&self, account_id: &str) -> CoreResult<()>;

    // === 迁移辅助方法 ===

    /// 加载原始 JSON（用于格式检测和迁移）
    ///
    /// 返回存储中的原始 JSON 字符串，不进行反序列化。
    async fn load_raw_json(&self) -> CoreResult<String>;

    /// 保存原始 JSON（用于迁移写入）
    ///
    /// 直接写入 JSON 字符串到存储，不进行序列化。
    ///
    /// # Arguments
    /// * `json` - JSON 字符串
    async fn save_raw_json(&self, json: &str) -> CoreResult<()>;
}
