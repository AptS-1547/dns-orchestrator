//! 域名元数据管理服务

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::CoreResult;
use crate::traits::DomainMetadataRepository;
use crate::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

/// 域名元数据管理服务
pub struct DomainMetadataService {
    repository: Arc<dyn DomainMetadataRepository>,
}

impl DomainMetadataService {
    /// 创建元数据服务实例
    #[must_use]
    pub fn new(repository: Arc<dyn DomainMetadataRepository>) -> Self {
        Self { repository }
    }

    /// 获取元数据（不存在则返回默认值）
    pub async fn get_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
    ) -> CoreResult<DomainMetadata> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok(self.repository.find_by_key(&key).await?.unwrap_or_default())
    }

    /// 批量获取元数据（用于域名列表，性能优化）
    pub async fn get_metadata_batch(
        &self,
        keys: Vec<(String, String)>, // (account_id, domain_id) 对
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        let keys: Vec<DomainMetadataKey> = keys
            .into_iter()
            .map(|(acc, dom)| DomainMetadataKey::new(acc, dom))
            .collect();
        self.repository.find_by_keys(&keys).await
    }

    /// 更新元数据（全量）
    pub async fn save_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
        metadata: DomainMetadata,
    ) -> CoreResult<()> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.save(&key, &metadata).await
    }

    /// 更新元数据（部分，Phase 2/3 使用）
    pub async fn update_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
        update: DomainMetadataUpdate,
    ) -> CoreResult<()> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.update(&key, &update).await
    }

    /// 删除元数据
    pub async fn delete_metadata(&self, account_id: &str, domain_id: &str) -> CoreResult<()> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.delete(&key).await
    }

    /// 切换收藏状态
    pub async fn toggle_favorite(&self, account_id: &str, domain_id: &str) -> CoreResult<bool> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;
        metadata.is_favorite = !metadata.is_favorite;
        metadata.touch();

        let new_state = metadata.is_favorite;
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(new_state)
    }

    /// 获取账户下的收藏域名键
    pub async fn list_favorites(&self, account_id: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        self.repository.find_favorites_by_account(account_id).await
    }

    /// 删除账户下的所有元数据（账户删除时调用）
    pub async fn delete_account_metadata(&self, account_id: &str) -> CoreResult<()> {
        self.repository.delete_by_account(account_id).await
    }
}
