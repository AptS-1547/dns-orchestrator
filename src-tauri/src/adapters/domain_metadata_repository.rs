//! Tauri 域名元数据仓库适配器
//!
//! 使用 tauri-plugin-store 实现元数据持久化

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio::sync::RwLock;

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::DomainMetadataRepository;
use dns_orchestrator_core::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

const STORE_FILE_NAME: &str = "domain_metadata.json";
const METADATA_KEY: &str = "metadata";

/// Tauri 域名元数据仓库实现
pub struct TauriDomainMetadataRepository {
    app_handle: AppHandle,
    /// 内存缓存（key: `storage_key`, value: metadata）
    cache: Arc<RwLock<Option<HashMap<String, DomainMetadata>>>>,
}

impl TauriDomainMetadataRepository {
    /// 创建新的元数据仓库实例
    #[must_use]
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 从 Store 加载所有元数据
    fn load_from_store(&self) -> CoreResult<HashMap<String, DomainMetadata>> {
        let store = self
            .app_handle
            .store(STORE_FILE_NAME)
            .map_err(|e| CoreError::StorageError(format!("Failed to access store: {e}")))?;

        let Some(value) = store.get(METADATA_KEY) else {
            return Ok(HashMap::new());
        };

        serde_json::from_value(value.clone())
            .map_err(|e| CoreError::SerializationError(e.to_string()))
    }

    /// 保存所有元数据到 Store
    fn save_to_store(&self, metadata_map: &HashMap<String, DomainMetadata>) -> CoreResult<()> {
        let store = self
            .app_handle
            .store(STORE_FILE_NAME)
            .map_err(|e| CoreError::StorageError(format!("Failed to access store: {e}")))?;

        let value = serde_json::to_value(metadata_map)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        store.set(METADATA_KEY.to_string(), value);
        store
            .save()
            .map_err(|e| CoreError::StorageError(format!("Failed to save store: {e}")))?;

        log::debug!(
            "Saved {} domain metadata entries to store",
            metadata_map.len()
        );
        Ok(())
    }

    /// 加载或初始化缓存（延迟加载）
    async fn ensure_cache(&self) -> CoreResult<()> {
        let cache = self.cache.read().await;
        if cache.is_none() {
            drop(cache);
            let data = self.load_from_store()?;
            let mut cache = self.cache.write().await;
            *cache = Some(data);
        }
        Ok(())
    }
}

#[async_trait]
impl DomainMetadataRepository for TauriDomainMetadataRepository {
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        self.ensure_cache().await?;
        let cache = self.cache.read().await;
        let storage_key = key.to_storage_key();
        Ok(cache.as_ref().and_then(|c| c.get(&storage_key).cloned()))
    }

    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        self.ensure_cache().await?;
        let cache = self.cache.read().await;
        let mut result = HashMap::new();

        if let Some(ref cache_data) = *cache {
            for key in keys {
                let storage_key = key.to_storage_key();
                if let Some(metadata) = cache_data.get(&storage_key) {
                    result.insert(key.clone(), metadata.clone());
                }
            }
        }

        Ok(result)
    }

    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()> {
        self.ensure_cache().await?;
        let storage_key = key.to_storage_key();

        let mut cache = self.cache.write().await;
        let cache_data = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Cache not initialized".to_string()))?;

        // 如果元数据为空，删除条目（节省空间）
        if metadata.is_empty() {
            cache_data.remove(&storage_key);
        } else {
            cache_data.insert(storage_key, metadata.clone());
        }

        self.save_to_store(cache_data)?;
        Ok(())
    }

    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        self.ensure_cache().await?;
        let mut cache = self.cache.write().await;
        let cache_data = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Cache not initialized".to_string()))?;

        // 批量修改内存缓存
        for (key, metadata) in entries {
            let storage_key = key.to_storage_key();
            if metadata.is_empty() {
                cache_data.remove(&storage_key);
            } else {
                cache_data.insert(storage_key, metadata.clone());
            }
        }

        // 只写入一次文件
        self.save_to_store(cache_data)?;
        log::info!(
            "Batch saved {} domain metadata entries to store",
            entries.len()
        );
        Ok(())
    }

    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        self.ensure_cache().await?;
        let storage_key = key.to_storage_key();

        let mut cache = self.cache.write().await;
        let cache_data = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Cache not initialized".to_string()))?;

        let mut metadata = cache_data.get(&storage_key).cloned().unwrap_or_default();

        update.apply_to(&mut metadata);

        // 如果更新后为空，删除条目
        if metadata.is_empty() {
            cache_data.remove(&storage_key);
        } else {
            cache_data.insert(storage_key, metadata);
        }

        self.save_to_store(cache_data)?;
        Ok(())
    }

    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()> {
        self.ensure_cache().await?;
        let storage_key = key.to_storage_key();

        let mut cache = self.cache.write().await;
        let cache_data = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Cache not initialized".to_string()))?;
        cache_data.remove(&storage_key);

        self.save_to_store(cache_data)?;
        Ok(())
    }

    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()> {
        self.ensure_cache().await?;

        let mut cache = self.cache.write().await;
        let cache_data = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Cache not initialized".to_string()))?;

        cache_data.retain(|storage_key, _| {
            DomainMetadataKey::from_storage_key(storage_key)
                .is_some_and(|key| key.account_id != account_id)
        });

        self.save_to_store(cache_data)?;
        Ok(())
    }

    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        self.ensure_cache().await?;
        let cache = self.cache.read().await;
        let mut result = Vec::new();

        if let Some(ref cache_data) = *cache {
            for (storage_key, metadata) in cache_data {
                if metadata.is_favorite {
                    if let Some(key) = DomainMetadataKey::from_storage_key(storage_key) {
                        if key.account_id == account_id {
                            result.push(key);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        self.ensure_cache().await?;
        let cache = self.cache.read().await;
        let mut result = Vec::new();

        if let Some(ref cache_data) = *cache {
            for (storage_key, metadata) in cache_data {
                if metadata.tags.contains(&tag.to_string()) {
                    if let Some(key) = DomainMetadataKey::from_storage_key(storage_key) {
                        result.push(key);
                    }
                }
            }
        }

        Ok(result)
    }

    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        self.ensure_cache().await?;
        let cache = self.cache.read().await;
        let mut tags = std::collections::HashSet::new();

        if let Some(ref cache_data) = *cache {
            for metadata in cache_data.values() {
                tags.extend(metadata.tags.iter().cloned());
            }
        }

        let mut result: Vec<String> = tags.into_iter().collect();
        result.sort();
        Ok(result)
    }
}
