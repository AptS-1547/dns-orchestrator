//! 域名元数据持久化抽象 Trait

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::CoreResult;
use crate::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

/// 域名元数据仓库 Trait
///
/// 平台实现:
/// - Tauri: `TauriDomainMetadataRepository` (tauri-plugin-store)
/// - Actix-Web: `DatabaseDomainMetadataRepository` (`SeaORM`)
#[async_trait]
pub trait DomainMetadataRepository: Send + Sync {
    /// 获取单个域名的元数据
    ///
    /// # Returns
    /// * `Some(metadata)` - 找到元数据
    /// * `None` - 未找到（使用默认值）
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>>;

    /// 批量获取多个域名的元数据（性能优化）
    ///
    /// # Arguments
    /// * `keys` - 域名元数据键列表
    ///
    /// # Returns
    /// * 键值对映射（仅包含存在的元数据）
    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>;

    /// 保存或更新元数据
    ///
    /// # Arguments
    /// * `key` - 域名元数据键
    /// * `metadata` - 元数据
    ///
    /// # Note
    /// 如果 `metadata.is_empty()` 为 true，应删除存储条目
    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()>;

    /// 批量保存多个域名的元数据（性能优化，只写入一次文件）
    ///
    /// # Arguments
    /// * `entries` - 键值对列表
    ///
    /// # Note
    /// - 此方法用于批量操作性能优化，避免多次文件写入
    /// - 如果某个 `metadata.is_empty()` 为 true，应删除对应存储条目
    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()>;

    /// 更新元数据（部分更新，Phase 2/3 使用）
    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()>;

    /// 删除元数据
    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()>;

    /// 删除账户下的所有元数据（账户删除时调用）
    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()>;

    /// 获取账户下所有收藏的域名键
    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>>;

    /// 按标签查询域名（返回所有包含该标签的域名键）
    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>>;

    /// 获取所有使用过的标签（去重、排序）
    async fn list_all_tags(&self) -> CoreResult<Vec<String>>;
}
