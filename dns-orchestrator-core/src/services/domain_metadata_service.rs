//! 域名元数据管理服务

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::CoreResult;
use crate::traits::DomainMetadataRepository;
use crate::types::{
    BatchTagFailure, BatchTagRequest, BatchTagResult, DomainMetadata, DomainMetadataKey,
    DomainMetadataUpdate,
};

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
        use crate::error::CoreError;

        // 颜色验证（"none" 表示无颜色）
        const VALID_COLORS: &[&str] = &[
            "red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "brown", "gray",
            "none",
        ];

        if let Some(ref color) = update.color {
            if !VALID_COLORS.contains(&color.as_str()) {
                return Err(CoreError::ValidationError(format!(
                    "Invalid color key: '{}'. Must be one of: {}",
                    color,
                    VALID_COLORS.join(", ")
                )));
            }
        }

        // 备注长度验证（仅验证非空值）
        if let Some(Some(ref note)) = update.note {
            if note.len() > 500 {
                return Err(CoreError::ValidationError(
                    "Note length cannot exceed 500 characters".to_string(),
                ));
            }
        }

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

        // 首次收藏时记录时间，之后永不修改
        if metadata.is_favorite && metadata.favorited_at.is_none() {
            metadata.favorited_at = Some(chrono::Utc::now());
        }
        // 注意：取消收藏时不清空 favorited_at

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

    /// 添加标签（返回更新后的标签列表）
    pub async fn add_tag(
        &self,
        account_id: &str,
        domain_id: &str,
        tag: String,
    ) -> CoreResult<Vec<String>> {
        use crate::error::CoreError;

        // 标签验证
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            return Err(CoreError::ValidationError(
                "Tag cannot be empty".to_string(),
            ));
        }
        if tag.len() > 50 {
            return Err(CoreError::ValidationError(
                "Tag length cannot exceed 50 characters".to_string(),
            ));
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 去重：检查标签是否已存在
        if metadata.tags.contains(&tag) {
            return Ok(metadata.tags);
        }

        // 限制标签数量
        if metadata.tags.len() >= 10 {
            return Err(CoreError::ValidationError(
                "Cannot add more than 10 tags".to_string(),
            ));
        }

        metadata.tags.push(tag);
        metadata.tags.sort();
        metadata.touch();

        let tags = metadata.tags.clone();
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(tags)
    }

    /// 移除标签（返回更新后的标签列表）
    pub async fn remove_tag(
        &self,
        account_id: &str,
        domain_id: &str,
        tag: &str,
    ) -> CoreResult<Vec<String>> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 移除标签（不存在也不报错，静默处理）
        metadata.tags.retain(|t| t != tag);
        metadata.touch();

        let tags = metadata.tags.clone();
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(tags)
    }

    /// 批量设置标签（替换所有标签）
    pub async fn set_tags(
        &self,
        account_id: &str,
        domain_id: &str,
        tags: Vec<String>,
    ) -> CoreResult<Vec<String>> {
        use crate::error::CoreError;

        // 验证每个标签
        for tag in &tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                return Err(CoreError::ValidationError(
                    "Tag cannot be empty".to_string(),
                ));
            }
            if trimmed.len() > 50 {
                return Err(CoreError::ValidationError(
                    "Tag length cannot exceed 50 characters".to_string(),
                ));
            }
        }

        if tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot have more than 10 tags".to_string(),
            ));
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 清理、去重、排序
        let mut cleaned_tags: Vec<String> = tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        cleaned_tags.sort();
        cleaned_tags.dedup();

        metadata.tags = cleaned_tags.clone();
        metadata.touch();

        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(cleaned_tags)
    }

    /// 按标签查询域名（跨账户）
    pub async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        self.repository.find_by_tag(tag).await
    }

    /// 获取所有使用过的标签（用于自动补全，可选功能）
    pub async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        self.repository.list_all_tags().await
    }

    // ===== 批量标签操作方法 =====

    /// 批量添加标签（为多个域名添加相同标签）
    pub async fn batch_add_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // 第一阶段：在内存中处理所有修改
        for req in requests {
            match self
                .add_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // 第二阶段：一次性批量保存
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    /// 批量移除标签（从多个域名移除相同标签）
    pub async fn batch_remove_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // 第一阶段：在内存中处理所有修改
        for req in requests {
            match self
                .remove_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // 第二阶段：一次性批量保存
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    /// 批量替换标签（清空原有标签后设置新标签）
    pub async fn batch_set_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // 第一阶段：在内存中处理所有修改
        for req in requests {
            match self
                .set_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // 第二阶段：一次性批量保存
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    // ===== 内部辅助方法（用于批量操作优化） =====

    /// 内部方法：为单个域名添加标签（不保存，用于批量操作）
    async fn add_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags_to_add: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        use crate::error::CoreError;

        // 验证每个标签
        for tag in &tags_to_add {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                return Err(CoreError::ValidationError(
                    "Tag cannot be empty".to_string(),
                ));
            }
            if trimmed.len() > 50 {
                return Err(CoreError::ValidationError(
                    "Tag length cannot exceed 50 characters".to_string(),
                ));
            }
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 合并标签并去重
        let mut all_tags: Vec<String> = metadata.tags.clone();
        for tag in tags_to_add {
            let trimmed = tag.trim().to_string();
            if !trimmed.is_empty() && !all_tags.contains(&trimmed) {
                all_tags.push(trimmed);
            }
        }

        // 检查标签数量限制
        if all_tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot exceed 10 tags".to_string(),
            ));
        }

        all_tags.sort();
        all_tags.dedup();

        metadata.tags = all_tags;
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }

    /// 内部方法：为单个域名移除标签（不保存，用于批量操作）
    async fn remove_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags_to_remove: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 移除指定的标签
        let tags_to_remove_set: std::collections::HashSet<String> = tags_to_remove
            .into_iter()
            .map(|t| t.trim().to_string())
            .collect();

        metadata.tags.retain(|t| !tags_to_remove_set.contains(t));
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }

    /// 内部方法：为单个域名替换标签（不保存，用于批量操作）
    async fn set_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        use crate::error::CoreError;

        // 验证每个标签
        for tag in &tags {
            let trimmed = tag.trim();
            if trimmed.is_empty() {
                return Err(CoreError::ValidationError(
                    "Tag cannot be empty".to_string(),
                ));
            }
            if trimmed.len() > 50 {
                return Err(CoreError::ValidationError(
                    "Tag length cannot exceed 50 characters".to_string(),
                ));
            }
        }

        if tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot have more than 10 tags".to_string(),
            ));
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // 清理、去重、排序
        let mut cleaned_tags: Vec<String> = tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        cleaned_tags.sort();
        cleaned_tags.dedup();

        metadata.tags = cleaned_tags;
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }
}
