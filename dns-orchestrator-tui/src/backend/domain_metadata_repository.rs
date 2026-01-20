//! 域名元数据仓库
//!
//! 实现 dns-orchestrator-core 的 DomainMetadataRepository trait
//! TUI 使用 JSON 文件存储域名元数据

use async_trait::async_trait;
use dns_orchestrator_core::traits::DomainMetadataRepository;
use dns_orchestrator_core::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};
use dns_orchestrator_core::CoreResult;
use std::collections::HashMap;
use tokio::sync::Mutex;

/// 内存域名元数据仓库
///
/// 简单实现，数据存储在内存中（重启后丢失）
/// TODO: 实现 JSON 文件持久化
pub struct InMemoryDomainMetadataRepository {
    store: Mutex<HashMap<DomainMetadataKey, DomainMetadata>>,
}

impl InMemoryDomainMetadataRepository {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryDomainMetadataRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainMetadataRepository for InMemoryDomainMetadataRepository {
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        let store = self.store.lock().await;
        Ok(store.get(key).cloned())
    }

    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        let store = self.store.lock().await;
        let mut result = HashMap::new();
        for key in keys {
            if let Some(metadata) = store.get(key) {
                result.insert(key.clone(), metadata.clone());
            }
        }
        Ok(result)
    }

    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()> {
        let mut store = self.store.lock().await;
        if metadata.is_empty() {
            store.remove(key);
        } else {
            store.insert(key.clone(), metadata.clone());
        }
        Ok(())
    }

    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()> {
        let mut store = self.store.lock().await;
        for (key, metadata) in entries {
            if metadata.is_empty() {
                store.remove(key);
            } else {
                store.insert(key.clone(), metadata.clone());
            }
        }
        Ok(())
    }

    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        let mut store = self.store.lock().await;
        let metadata = store.entry(key.clone()).or_insert_with(DomainMetadata::default);

        if let Some(is_favorite) = update.is_favorite {
            metadata.is_favorite = is_favorite;
        }
        if let Some(ref color) = update.color {
            metadata.color = color.clone();
        }
        if let Some(ref tags) = update.tags {
            metadata.tags = tags.clone();
        }
        if let Some(ref note) = update.note {
            metadata.note = note.clone();
        }

        Ok(())
    }

    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()> {
        let mut store = self.store.lock().await;
        store.remove(key);
        Ok(())
    }

    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()> {
        let mut store = self.store.lock().await;
        store.retain(|k, _| k.account_id != account_id);
        Ok(())
    }

    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        let store = self.store.lock().await;
        Ok(store
            .iter()
            .filter(|(k, v)| k.account_id == account_id && v.is_favorite)
            .map(|(k, _)| k.clone())
            .collect())
    }

    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        let store = self.store.lock().await;
        Ok(store
            .iter()
            .filter(|(_, v)| v.tags.contains(&tag.to_string()))
            .map(|(k, _)| k.clone())
            .collect())
    }

    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        let store = self.store.lock().await;
        let mut tags: Vec<String> = store
            .values()
            .flat_map(|m| m.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        Ok(tags)
    }
}
