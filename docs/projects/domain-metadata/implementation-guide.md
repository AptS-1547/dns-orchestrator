# 域名元数据系统 - 实施指南

本文档提供域名元数据系统的详细实施步骤，包含完整的代码示例和注意事项。

## 目录

- [Phase 1: 基础收藏功能](#phase-1-基础收藏功能)
  - [步骤 1: Core 层类型定义](#步骤-1-core-层类型定义)
  - [步骤 2: Repository Trait](#步骤-2-repository-trait)
  - [步骤 3: Adapter 实现](#步骤-3-adapter-实现)
  - [步骤 4: Service 层](#步骤-4-service-层)
  - [步骤 5: ServiceContext 修改](#步骤-5-servicecontext-修改)
  - [步骤 6: DomainService 修改](#步骤-6-domainservice-修改)
  - [步骤 7: Tauri 命令层](#步骤-7-tauri-命令层)
  - [步骤 8: AppState 初始化](#步骤-8-appstate-初始化)
  - [步骤 9: 前端类型定义](#步骤-9-前端类型定义)
  - [步骤 10: 前端 Service](#步骤-10-前端-service)
  - [步骤 11: DomainStore 扩展](#步骤-11-domainstore-扩展)
  - [步骤 12: UI 组件](#步骤-12-ui-组件)
  - [步骤 13: 验证](#步骤-13-验证)
- [Phase 2: 标签系统](#phase-2-标签系统)
- [Phase 3: 完整元数据](#phase-3-完整元数据)

---

## Phase 1: 基础收藏功能

### 目标

实现完整的域名收藏功能链路，验证架构正确性。

**功能范围**：
- ✅ 收藏/取消收藏域名
- ✅ 后端持久化到 `domain_metadata.json`
- ✅ 自动合并元数据到域名列表
- ✅ 前端星标按钮 UI

**不包含**：标签、颜色、备注（Phase 2/3）

---

### 步骤 1: Core 层类型定义

**文件**: `dns-orchestrator-core/src/types/domain_metadata.rs`

**创建新文件**，定义域名元数据的核心类型：

```rust
//! 域名元数据类型定义

use serde::{Deserialize, Serialize};

/// 域名元数据键（复合主键）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataKey {
    pub account_id: String,
    pub domain_id: String,
}

impl DomainMetadataKey {
    /// 创建新的元数据键
    #[must_use]
    pub fn new(account_id: String, domain_id: String) -> Self {
        Self {
            account_id,
            domain_id,
        }
    }

    /// 生成存储用的字符串键（格式: account_id::domain_id）
    #[must_use]
    pub fn to_storage_key(&self) -> String {
        format!("{}::{}", self.account_id, self.domain_id)
    }

    /// 从存储键解析
    #[must_use]
    pub fn from_storage_key(key: &str) -> Option<Self> {
        let parts: Vec<&str> = key.split("::").collect();
        if parts.len() != 2 {
            return None;
        }
        Some(Self {
            account_id: parts[0].to_string(),
            domain_id: parts[1].to_string(),
        })
    }
}

/// 域名元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    /// 是否收藏
    #[serde(default)]
    pub is_favorite: bool,

    /// 标签列表（Phase 2 实现）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// 颜色标记（Phase 3 实现）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// 备注（Phase 3 实现）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// 最后修改时间（Unix 时间戳，毫秒）
    pub updated_at: i64,
}

impl Default for DomainMetadata {
    fn default() -> Self {
        Self {
            is_favorite: false,
            tags: Vec::new(),
            color: None,
            note: None,
            updated_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl DomainMetadata {
    /// 刷新更新时间
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    /// 是否为空元数据（所有字段都是默认值）
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_favorite
            && self.tags.is_empty()
            && self.color.is_none()
            && self.note.is_none()
    }
}

/// 域名元数据更新请求（支持部分更新，Phase 2/3 使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Option<String>>,
}

impl DomainMetadataUpdate {
    /// 应用更新到现有元数据
    pub fn apply_to(&self, metadata: &mut DomainMetadata) {
        if let Some(is_favorite) = self.is_favorite {
            metadata.is_favorite = is_favorite;
        }
        if let Some(ref tags) = self.tags {
            metadata.tags = tags.clone();
        }
        if let Some(ref color) = self.color {
            metadata.color = color.clone();
        }
        if let Some(ref note) = self.note {
            metadata.note = note.clone();
        }
        metadata.touch();
    }
}
```

**修改** `dns-orchestrator-core/src/types/mod.rs`，导出新类型：

```rust
// 在 mod 声明中添加
mod domain_metadata;

// 在 pub use 中添加
pub use domain_metadata::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};
```

**修改** `dns-orchestrator-core/src/types/domain.rs`，扩展 `AppDomain`：

```rust
use super::domain_metadata::DomainMetadata;  // 新增导入

pub struct AppDomain {
    // ... 现有字段 ...

    /// 用户自定义元数据（新增）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DomainMetadata>,
}

impl AppDomain {
    // ... 现有方法 ...

    /// 附加元数据（新增辅助方法）
    #[must_use]
    pub fn with_metadata(mut self, metadata: Option<DomainMetadata>) -> Self {
        self.metadata = metadata;
        self
    }
}
```

**关键点**：
- `DomainMetadataKey` 使用 `account_id::domain_id` 格式作为存储键
- `is_empty()` 方法用于判断是否删除存储条目（节省空间）
- Phase 2/3 字段预留但暂不实现逻辑

---

### 步骤 2: Repository Trait

**文件**: `dns-orchestrator-core/src/traits/domain_metadata_repository.rs`

**创建新文件**，定义元数据持久化抽象接口：

```rust
//! 域名元数据持久化抽象 Trait

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::CoreResult;
use crate::types::{DomainMetadata, DomainMetadataKey, DomainMetadataUpdate};

/// 域名元数据仓库 Trait
///
/// 平台实现:
/// - Tauri: `TauriDomainMetadataRepository` (tauri-plugin-store)
/// - Actix-Web: `DatabaseDomainMetadataRepository` (SeaORM)
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
}
```

**修改** `dns-orchestrator-core/src/traits/mod.rs`，导出新 trait：

```rust
// 在 mod 声明中添加
mod domain_metadata_repository;

// 在 pub use 中添加
pub use domain_metadata_repository::DomainMetadataRepository;
```

**关键点**：
- `find_by_keys()` 批量读取优化，避免逐个查询
- `save()` 内部应检查 `is_empty()` 并删除空条目
- 参考 `AccountRepository` 的设计模式

---

### 步骤 3: Adapter 实现

**文件**: `src-tauri/src/adapters/domain_metadata_repository.rs`

**创建新文件**，实现 Tauri 平台的元数据存储：

```rust
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
    /// 内存缓存（key: storage_key, value: metadata）
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

        log::debug!("Saved {} domain metadata entries to store", metadata_map.len());
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
        let cache_data = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Cache not initialized".to_string())
        })?;

        // 如果元数据为空，删除条目（节省空间）
        if metadata.is_empty() {
            cache_data.remove(&storage_key);
        } else {
            cache_data.insert(storage_key, metadata.clone());
        }

        self.save_to_store(cache_data)?;
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
        let cache_data = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Cache not initialized".to_string())
        })?;

        let mut metadata = cache_data
            .get(&storage_key)
            .cloned()
            .unwrap_or_default();

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
        let cache_data = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Cache not initialized".to_string())
        })?;
        cache_data.remove(&storage_key);

        self.save_to_store(cache_data)?;
        Ok(())
    }

    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()> {
        self.ensure_cache().await?;

        let mut cache = self.cache.write().await;
        let cache_data = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Cache not initialized".to_string())
        })?;

        cache_data.retain(|storage_key, _| {
            DomainMetadataKey::from_storage_key(storage_key)
                .map_or(false, |key| key.account_id != account_id)
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
}
```

**修改** `src-tauri/src/adapters/mod.rs`，导出新 adapter：

```rust
// 在 mod 声明中添加
mod domain_metadata_repository;

// 在 pub use 中添加
pub use domain_metadata_repository::TauriDomainMetadataRepository;
```

**关键点**：
- 使用 `RwLock` 保证线程安全
- 延迟加载：首次访问时才从文件读取
- 每次写入都保存到文件（确保数据不丢失）
- 空元数据自动删除，节省存储空间

---

### 步骤 4: Service 层

**文件**: `dns-orchestrator-core/src/services/domain_metadata_service.rs`

**创建新文件**，实现元数据业务逻辑：

```rust
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
        Ok(self
            .repository
            .find_by_key(&key)
            .await?
            .unwrap_or_default())
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
    pub async fn list_favorites(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        self.repository.find_favorites_by_account(account_id).await
    }

    /// 删除账户下的所有元数据（账户删除时调用）
    pub async fn delete_account_metadata(&self, account_id: &str) -> CoreResult<()> {
        self.repository.delete_by_account(account_id).await
    }
}
```

**修改** `dns-orchestrator-core/src/services/mod.rs`，导出新 service：

```rust
// 在 mod 声明中添加
mod domain_metadata_service;

// 在 pub use 中添加
pub use domain_metadata_service::DomainMetadataService;
```

**关键点**：
- `toggle_favorite()` 返回新状态，方便前端更新 UI
- `get_metadata_batch()` 供 `DomainService` 调用
- Service 层不直接操作存储，全部通过 Repository trait

---

### 步骤 5: ServiceContext 修改

**文件**: `dns-orchestrator-core/src/services/mod.rs`

**修改 `ServiceContext`** 结构，注入元数据仓库：

```rust
use crate::traits::DomainMetadataRepository;  // 新增导入

/// 服务上下文 - 持有所有依赖
pub struct ServiceContext {
    pub credential_store: Arc<dyn CredentialStore>,
    pub account_repository: Arc<dyn AccountRepository>,
    pub provider_registry: Arc<dyn ProviderRegistry>,
    pub domain_metadata_repository: Arc<dyn DomainMetadataRepository>,  // 新增字段
}

impl ServiceContext {
    /// 创建服务上下文
    #[must_use]
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
        provider_registry: Arc<dyn ProviderRegistry>,
        domain_metadata_repository: Arc<dyn DomainMetadataRepository>,  // 新增参数
    ) -> Self {
        Self {
            credential_store,
            account_repository,
            provider_registry,
            domain_metadata_repository,  // 新增字段初始化
        }
    }

    // ... 其他方法保持不变 ...
}
```

**关键点**：
- 这是**破坏性修改**，所有 `ServiceContext::new()` 调用点都需要更新
- 目前只有 1 处调用点：`src-tauri/src/lib.rs` 的 `AppState::new()`

---

### 步骤 6: DomainService 修改

**文件**: `dns-orchestrator-core/src/services/domain_service.rs`

**修改 `list_domains()` 方法**，自动合并元数据：

找到 `list_domains` 方法，在返回前添加元数据合并逻辑：

```rust
use crate::types::DomainMetadataKey;  // 新增导入
use super::DomainMetadataService;  // 新增导入

pub async fn list_domains(
    &self,
    account_id: &str,
    page: Option<u32>,
    page_size: Option<u32>,
) -> CoreResult<PaginatedResponse<AppDomain>> {
    let provider = self.ctx.get_provider(account_id).await?;

    let params = PaginationParams {
        page: page.unwrap_or(1),
        page_size: page_size.unwrap_or(20),
    };

    match provider.list_domains(&params).await {
        Ok(lib_response) => {
            let mut domains: Vec<AppDomain> = lib_response
                .items
                .into_iter()
                .map(|d| AppDomain::from_provider(d, account_id.to_string()))
                .collect();

            // 批量加载元数据并合并（新增逻辑）
            let keys: Vec<(String, String)> = domains
                .iter()
                .map(|d| (d.account_id.clone(), d.id.clone()))
                .collect();

            let metadata_service = DomainMetadataService::new(
                Arc::clone(&self.ctx.domain_metadata_repository)
            );

            if let Ok(metadata_map) = metadata_service.get_metadata_batch(keys).await {
                for domain in &mut domains {
                    let key = DomainMetadataKey::new(
                        domain.account_id.clone(),
                        domain.id.clone(),
                    );
                    if let Some(metadata) = metadata_map.get(&key) {
                        domain.metadata = Some(metadata.clone());
                    }
                }
            }

            Ok(PaginatedResponse::new(
                domains,
                lib_response.page,
                lib_response.page_size,
                lib_response.total_count,
            ))
        }
        Err(e) => Err(self.handle_provider_error(account_id, e).await),
    }
}
```

**关键点**：
- 使用 `get_metadata_batch()` 批量读取，避免 N+1 查询
- 错误静默处理：如果元数据加载失败，域名列表仍正常返回
- 只附加非空元数据（`metadata_map.get()` 仅返回存在的条目）

---

### 步骤 7: Tauri 命令层

**文件**: `src-tauri/src/commands/domain_metadata.rs`

**创建新文件**，定义 Tauri 命令：

```rust
//! 域名元数据相关命令

use tauri::State;

use crate::error::DnsError;
use crate::types::ApiResponse;
use crate::AppState;

use serde::{Deserialize, Serialize};

// 本地类型定义（与前端对应）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub note: Option<String>,
    pub updated_at: i64,
}

// 类型转换
impl From<dns_orchestrator_core::types::DomainMetadata> for DomainMetadata {
    fn from(core: dns_orchestrator_core::types::DomainMetadata) -> Self {
        Self {
            is_favorite: core.is_favorite,
            tags: core.tags,
            color: core.color,
            note: core.note,
            updated_at: core.updated_at,
        }
    }
}

/// 获取域名元数据
#[tauri::command]
pub async fn get_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<DomainMetadata>, DnsError> {
    let metadata = state
        .domain_metadata_service
        .get_metadata(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(metadata.into()))
}

/// 切换收藏状态
#[tauri::command]
pub async fn toggle_domain_favorite(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<bool>, DnsError> {
    let new_state = state
        .domain_metadata_service
        .toggle_favorite(&account_id, &domain_id)
        .await?;

    Ok(ApiResponse::success(new_state))
}

/// 获取账户下的收藏域名 ID 列表
#[tauri::command]
pub async fn list_account_favorite_domain_keys(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<Vec<String>>, DnsError> {
    let keys = state
        .domain_metadata_service
        .list_favorites(&account_id)
        .await?;

    let result = keys.into_iter().map(|k| k.domain_id).collect();

    Ok(ApiResponse::success(result))
}
```

**修改** `src-tauri/src/commands/mod.rs`，导出新模块：

```rust
pub mod domain_metadata;  // 新增
```

**关键点**：
- Phase 1 仅实现 3 个核心命令（get、toggle、list）
- 使用本地 `DomainMetadata` 类型（避免导出 core 内部类型）
- 返回 `ApiResponse<T>` 统一响应格式

---

### 步骤 8: AppState 初始化

**文件**: `src-tauri/src/lib.rs`

**修改 AppState 结构**，添加元数据服务字段：

```rust
use adapters::TauriDomainMetadataRepository;  // 新增导入
use dns_orchestrator_core::services::DomainMetadataService;  // 新增导入

pub struct AppState {
    pub ctx: Arc<ServiceContext>,
    // ... 现有字段 ...
    pub domain_metadata_service: Arc<DomainMetadataService>,  // 新增字段
}
```

**修改 `AppState::new()` 方法**，初始化元数据服务：

```rust
impl AppState {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        // ... 现有适配器创建 ...

        // 创建元数据仓库（新增）
        let domain_metadata_repository = Arc::new(
            TauriDomainMetadataRepository::new(app_handle.clone())
        );

        // 创建服务上下文（添加新参数）
        let ctx = Arc::new(ServiceContext::new(
            credential_store.clone(),
            account_repository.clone(),
            provider_registry.clone(),
            domain_metadata_repository.clone(),  // 新增参数
        ));

        // 创建元数据服务（新增）
        let domain_metadata_service = Arc::new(
            DomainMetadataService::new(domain_metadata_repository)
        );

        // ... 其他服务创建 ...

        Self {
            ctx,
            // ... 现有字段 ...
            domain_metadata_service,  // 新增字段
            // ...
        }
    }
}
```

**修改 `run()` 函数**，注册 Tauri 命令：

找到 `.invoke_handler()` 调用，添加新命令：

```rust
.invoke_handler(tauri::generate_handler![
    // ... 现有命令 ...

    // 域名元数据命令（新增）
    domain_metadata::get_domain_metadata,
    domain_metadata::toggle_domain_favorite,
    domain_metadata::list_account_favorite_domain_keys,
])
```

**关键点**：
- 确保在 `ServiceContext::new()` 之前创建 `domain_metadata_repository`
- 命令注册顺序无关紧要

---

### 步骤 9: 前端类型定义

**文件**: `src/types/domain-metadata.ts`

**创建新文件**，定义前端元数据类型：

```typescript
/**
 * 域名元数据
 */
export interface DomainMetadata {
  /** 是否收藏 */
  isFavorite: boolean
  /** 标签列表（Phase 2） */
  tags: string[]
  /** 颜色标记（Phase 3） */
  color?: string
  /** 备注（Phase 3） */
  note?: string
  /** 最后修改时间（Unix 时间戳，毫秒） */
  updatedAt: number
}

/**
 * 域名元数据更新请求（部分更新，Phase 2/3 使用）
 */
export interface DomainMetadataUpdate {
  isFavorite?: boolean
  tags?: string[]
  /** null 表示清空字段 */
  color?: string | null
  /** null 表示清空字段 */
  note?: string | null
}
```

**修改** `src/types/domain.ts`，扩展 `Domain` 接口：

```typescript
import type { DomainMetadata } from "./domain-metadata"

export interface Domain {
  id: string
  name: string
  accountId: string
  provider: string
  status: DomainStatus
  recordCount?: number
  createdAt?: string
  metadata?: DomainMetadata  // 新增字段
}
```

**修改** `src/types/index.ts`（如果存在），导出新类型：

```typescript
export type { DomainMetadata, DomainMetadataUpdate } from "./domain-metadata"
```

**关键点**：
- 与后端类型完全对应（camelCase 命名）
- Phase 2/3 字段预留

---

### 步骤 10: 前端 Service

**文件**: `src/services/domainMetadata.service.ts`

**创建新文件**，实现前端 API 调用：

```typescript
import { transport } from "./transport"
import type { DomainMetadata, DomainMetadataUpdate } from "@/types"

class DomainMetadataService {
  /**
   * 获取域名元数据
   */
  async getMetadata(accountId: string, domainId: string) {
    return transport.invoke("get_domain_metadata", { accountId, domainId })
  }

  /**
   * 切换收藏状态
   * @returns 新的收藏状态
   */
  async toggleFavorite(accountId: string, domainId: string) {
    return transport.invoke("toggle_domain_favorite", { accountId, domainId })
  }

  /**
   * 获取账户下的收藏域名 ID 列表
   */
  async listAccountFavorites(accountId: string) {
    return transport.invoke("list_account_favorite_domain_keys", { accountId })
  }
}

export const domainMetadataService = new DomainMetadataService()
```

**修改** `src/services/index.ts`（如果存在），导出新 service：

```typescript
export { domainMetadataService } from "./domainMetadata.service"
```

**修改** `src/services/transport/types.ts`，添加命令类型映射：

```typescript
export interface CommandMap {
  // ... 现有命令 ...

  // 域名元数据命令（新增）
  get_domain_metadata: {
    params: { accountId: string; domainId: string }
    result: DomainMetadata
  }
  toggle_domain_favorite: {
    params: { accountId: string; domainId: string }
    result: boolean
  }
  list_account_favorite_domain_keys: {
    params: { accountId: string }
    result: string[]
  }
}
```

**关键点**：
- 类型安全：`transport.invoke()` 会检查参数和返回值类型
- Phase 1 仅实现 3 个方法

---

### 步骤 11: DomainStore 扩展

**文件**: `src/stores/domainStore.ts`

**添加 `toggleFavorite` 方法**：

```typescript
import { domainMetadataService } from "@/services"  // 新增导入

interface DomainState {
  // ... 现有状态 ...

  // 元数据操作（新增）
  toggleFavorite: (accountId: string, domainId: string) => Promise<void>
}

export const useDomainStore = create<DomainState>((set, get) => ({
  // ... 现有实现 ...

  // 切换收藏（新增方法）
  toggleFavorite: async (accountId, domainId) => {
    const response = await domainMetadataService.toggleFavorite(accountId, domainId)

    if (!response.success || response.data === undefined) {
      console.error("Failed to toggle favorite:", response.error)
      return
    }

    const newFavoriteState = response.data

    // 更新本地缓存
    set((state) => {
      const cache = state.domainsByAccount[accountId]
      if (!cache) return {}

      const domains = cache.domains.map((d) => {
        if (d.id === domainId) {
          return {
            ...d,
            metadata: {
              isFavorite: newFavoriteState,
              tags: d.metadata?.tags ?? [],
              updatedAt: Date.now(),
            },
          }
        }
        return d
      })

      return {
        domainsByAccount: {
          ...state.domainsByAccount,
          [accountId]: { ...cache, domains },
        },
      }
    })

    // 保存到 localStorage
    get().saveToStorage()
  },
}))
```

**关键点**：
- 乐观更新：立即更新 UI，不等待后端响应
- 错误处理：失败时打印错误但不阻塞 UI
- localStorage 同步：调用 `saveToStorage()` 保存缓存

---

### 步骤 12: UI 组件

**文件**: `src/components/domain/DomainFavoriteButton.tsx`

**创建新文件**，实现星标按钮组件：

```tsx
import { Star } from "lucide-react"
import { Button } from "@/components/ui/button"
import { useDomainStore } from "@/stores/domainStore"
import { cn } from "@/lib/utils"

interface DomainFavoriteButtonProps {
  accountId: string
  domainId: string
  isFavorite: boolean
}

export function DomainFavoriteButton({
  accountId,
  domainId,
  isFavorite,
}: DomainFavoriteButtonProps) {
  const toggleFavorite = useDomainStore((state) => state.toggleFavorite)

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation() // 阻止事件冒泡（避免触发域名选择）
    toggleFavorite(accountId, domainId)
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={handleClick}
      className="h-8 w-8"
      title={isFavorite ? "取消收藏" : "收藏"}
    >
      <Star
        className={cn(
          "h-4 w-4 transition-colors",
          isFavorite ? "fill-yellow-400 text-yellow-400" : "text-muted-foreground"
        )}
      />
    </Button>
  )
}
```

**修改域名列表组件**，集成星标按钮。

假设域名列表在 `src/components/domains/DomainSelectorPage.tsx` 或 `src/components/domain/DomainList.tsx`，找到渲染域名项的位置，添加星标按钮：

```tsx
import { DomainFavoriteButton } from "@/components/domain/DomainFavoriteButton"

// 在域名项渲染中添加：
<div className="flex items-center justify-between gap-2">
  <DomainFavoriteButton
    accountId={accountId}
    domainId={domain.id}
    isFavorite={domain.metadata?.isFavorite ?? false}
  />
  <div className="flex-1">{domain.name}</div>
  {/* ... 其他元素（状态徽章、记录数等） */}
</div>
```

**关键点**：
- `e.stopPropagation()` 防止触发父元素的点击事件
- 使用 `cn()` 工具函数动态应用样式
- 默认值 `false`：`domain.metadata?.isFavorite ?? false`

---

### 步骤 13: 验证

运行以下命令验证实施：

**后端验证**：
```bash
# 检查 Rust 代码编译
cargo check -p dns-orchestrator-core
cargo check -p dns-orchestrator

# 运行 clippy 检查
pnpm lint:rust

# 格式化检查
pnpm format:rust:check
```

**前端验证**：
```bash
# 检查 TypeScript 类型
pnpm tsc --noEmit

# 运行 lint
pnpm lint

# 格式化检查
pnpm format:check
```

**功能测试**：
```bash
# 启动开发模式
pnpm tauri dev

# 测试步骤：
# 1. 打开域名列表
# 2. 点击星标按钮，图标变为黄色填充
# 3. 刷新页面，收藏状态保持
# 4. 再次点击，取消收藏
# 5. 检查 domain_metadata.json 文件（macOS: ~/Library/Application Support/com.tauri.dns-orchestrator/）
```

**验收标准**：
- [ ] ✅ 所有 Rust 代码通过 clippy
- [ ] ✅ 所有 TypeScript 代码通过 lint
- [ ] ✅ 点击星标按钮，UI 立即响应
- [ ] ✅ 刷新页面后，收藏状态保持
- [ ] ✅ `domain_metadata.json` 文件正确保存数据
- [ ] ✅ 取消收藏后，空元数据从文件中删除

---

## Phase 2: 标签系统

> 待 Phase 1 完成后补充

**新增功能**：
- `addTag(accountId, domainId, tag)` - 添加标签
- `removeTag(accountId, domainId, tag)` - 移除标签
- `findByTag(tag)` - 按标签查询
- 标签徽章组件
- 标签筛选器

---

## Phase 3: 完整元数据

> 待 Phase 2 完成后补充

**新增功能**：
- 颜色标记（color picker）
- 备注编辑（textarea）
- 元数据编辑面板（Dialog）
- 部分更新 API（`update_domain_metadata`）

---

## 常见问题

### Q1: 为什么使用 `account_id::domain_id` 作为存储键？

**A**: 因为 `domain_id` 在单个 provider 内唯一，但跨 provider 可能重复。使用复合键确保全局唯一性。

### Q2: 为什么元数据要自动合并到 `list_domains()` 返回值？

**A**: 避免前端额外调用 API。批量读取优化后，性能损耗可忽略。

### Q3: 如果元数据文件损坏怎么办？

**A**: `TauriDomainMetadataRepository::load_from_store()` 会返回空 `HashMap`，降级为默认值，不会崩溃。

### Q4: 删除账户后，元数据会清理吗？

**A**: 需要在 `AccountLifecycleService::delete_account()` 中手动调用 `delete_account_metadata()`（Phase 1 暂未实现，Phase 2 补充）。

---

## 下一步

Phase 1 完成后：
1. ✅ 验证所有功能正常
2. 📝 更新用户文档（`docs/user-guide/`）
3. 🎨 UI/UX 优化（动画、交互反馈）
4. 🚀 开始 Phase 2 实施

---

**最后更新**: 2026-01-01
**作者**: AptS:1548 (Claude Sonnet 4.5)
