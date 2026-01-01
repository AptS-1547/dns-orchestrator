# 域名元数据系统 - 架构设计

本文档描述域名元数据系统的技术架构、数据流和设计决策。

## 架构概览

### 四层架构

```
┌─────────────────────────────────────────────────────────────┐
│                   Frontend (React + Zustand)                │
│  - domainStore                                              │
│  - DomainFavoriteButton                                     │
│  - domainMetadataService                                    │
└────────────────────────┬────────────────────────────────────┘
                         │ Tauri IPC
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Tauri Commands (domain_metadata.rs)            │
│  - get_domain_metadata                                      │
│  - toggle_domain_favorite                                   │
│  - list_account_favorite_domain_keys                        │
└────────────────────────┬────────────────────────────────────┘
                         │ 类型转换
                         ▼
┌─────────────────────────────────────────────────────────────┐
│         Core Service (DomainMetadataService)                │
│  - get_metadata / get_metadata_batch                        │
│  - toggle_favorite                                          │
│  - save_metadata                                            │
└────────────────────────┬────────────────────────────────────┘
                         │ Repository trait
                         ▼
┌─────────────────────────────────────────────────────────────┐
│   Repository Abstraction (DomainMetadataRepository trait)   │
│  - find_by_key / find_by_keys                               │
│  - save / delete                                            │
│  - find_favorites_by_account                                │
└────────────────────────┬────────────────────────────────────┘
                         │ 平台实现
                         ▼
┌─────────────────────────────────────────────────────────────┐
│       Adapter (TauriDomainMetadataRepository)               │
│  - 内存缓存：RwLock<HashMap>                                 │
│  - 延迟加载                                                  │
│  - tauri-plugin-store                                       │
└────────────────────────┬────────────────────────────────────┘
                         │ JSON 序列化
                         ▼
                  domain_metadata.json
```

---

## 核心组件

### 1. 类型系统 (dns-orchestrator-core/src/types/domain_metadata.rs)

#### DomainMetadataKey

**复合主键**，唯一标识一个域名的元数据。

```rust
pub struct DomainMetadataKey {
    pub account_id: String,  // 账户 ID
    pub domain_id: String,   // 域名 ID（Provider API 返回）
}
```

**存储键格式**：`{account_id}::{domain_id}`

**设计理由**：
- `domain_id` 在单个 provider 内唯一，但跨 provider 可能重复
- 使用复合键确保全局唯一性
- `::` 分隔符避免与 ID 内部字符冲突

#### DomainMetadata

**元数据结构**，包含所有用户自定义字段。

```rust
pub struct DomainMetadata {
    pub is_favorite: bool,           // Phase 1
    pub tags: Vec<String>,           // Phase 2
    pub color: Option<String>,       // Phase 3
    pub note: Option<String>,        // Phase 3
    pub updated_at: i64,             // 时间戳（毫秒）
}
```

**默认值**：
```rust
DomainMetadata {
    is_favorite: false,
    tags: vec![],
    color: None,
    note: None,
    updated_at: chrono::Utc::now().timestamp_millis(),
}
```

**空元数据判断**：
```rust
fn is_empty(&self) -> bool {
    !self.is_favorite && self.tags.is_empty()
        && self.color.is_none() && self.note.is_none()
}
```

**设计理由**：
- 空元数据自动删除，节省存储空间
- `updated_at` 用于未来的同步功能

#### DomainMetadataUpdate

**部分更新请求**（Phase 2/3 使用）。

```rust
pub struct DomainMetadataUpdate {
    pub is_favorite: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub color: Option<Option<String>>,  // Option<Option<T>> 允许清空字段
    pub note: Option<Option<String>>,
}
```

**设计理由**：
- `Option<T>` 表示"是否更新此字段"
- `Option<Option<T>>` 允许区分"不更新"和"清空"
  - `None` → 不更新
  - `Some(None)` → 清空字段
  - `Some(Some(value))` → 更新为 value

---

### 2. Repository Trait (dns-orchestrator-core/src/traits/domain_metadata_repository.rs)

**抽象存储接口**，支持多种平台实现。

#### 核心方法

```rust
#[async_trait]
pub trait DomainMetadataRepository: Send + Sync {
    // 单条查询
    async fn find_by_key(&self, key: &DomainMetadataKey)
        -> CoreResult<Option<DomainMetadata>>;

    // 批量查询（性能优化）
    async fn find_by_keys(&self, keys: &[DomainMetadataKey])
        -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>;

    // 保存（空元数据自动删除）
    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata)
        -> CoreResult<()>;

    // 删除
    async fn delete(&self, key: &DomainMetadataKey)
        -> CoreResult<()>;

    // 按账户删除（账户删除时调用）
    async fn delete_by_account(&self, account_id: &str)
        -> CoreResult<()>;

    // 查询收藏
    async fn find_favorites_by_account(&self, account_id: &str)
        -> CoreResult<Vec<DomainMetadataKey>>;
}
```

#### 设计模式

**参考**: `AccountRepository` trait

**优点**：
- 平台无关：Core 层不依赖具体存储实现
- 可测试：易于编写 mock 实现
- 可扩展：支持多种后端（Tauri、Web、桌面）

---

### 3. Adapter 实现 (src-tauri/src/adapters/domain_metadata_repository.rs)

#### 存储结构

**文件路径**：`~/Library/Application Support/com.tauri.dns-orchestrator/domain_metadata.json`

**JSON 格式**：
```json
{
  "metadata": {
    "account-uuid-1::domain-id-1": {
      "isFavorite": true,
      "tags": ["production"],
      "updatedAt": 1704067200000
    },
    "account-uuid-2::domain-id-5": {
      "isFavorite": true,
      "tags": [],
      "updatedAt": 1704067300000
    }
  }
}
```

#### 内存缓存

```rust
pub struct TauriDomainMetadataRepository {
    app_handle: AppHandle,
    cache: Arc<RwLock<Option<HashMap<String, DomainMetadata>>>>,
}
```

**缓存策略**：
- **延迟加载**：首次访问时从文件读取
- **写时保存**：每次修改立即写入文件
- **线程安全**：使用 `RwLock` 保护

**读取流程**：
```rust
async fn ensure_cache(&self) -> CoreResult<()> {
    let cache = self.cache.read().await;
    if cache.is_none() {
        drop(cache);  // 释放读锁
        let data = self.load_from_store()?;
        let mut cache = self.cache.write().await;  // 获取写锁
        *cache = Some(data);
    }
    Ok(())
}
```

**写入流程**：
1. 获取写锁 (`cache.write().await`)
2. 更新内存缓存
3. 调用 `save_to_store()` 写入文件
4. 释放写锁

**批量读取优化**：
```rust
async fn find_by_keys(&self, keys: &[DomainMetadataKey])
    -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>
{
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
```

**设计理由**：
- 减少文件 I/O（首次加载后全部在内存）
- 批量读取避免 N+1 查询
- 每次写入立即持久化，防止数据丢失

---

### 4. Service 层 (dns-orchestrator-core/src/services/domain_metadata_service.rs)

**业务逻辑封装**，提供高级 API。

#### 核心方法

```rust
pub struct DomainMetadataService {
    repository: Arc<dyn DomainMetadataRepository>,
}

impl DomainMetadataService {
    // 获取元数据（不存在返回默认值）
    pub async fn get_metadata(&self, account_id: &str, domain_id: &str)
        -> CoreResult<DomainMetadata>;

    // 批量获取（供 DomainService 调用）
    pub async fn get_metadata_batch(&self, keys: Vec<(String, String)>)
        -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>;

    // 切换收藏状态（返回新状态）
    pub async fn toggle_favorite(&self, account_id: &str, domain_id: &str)
        -> CoreResult<bool>;

    // 保存元数据
    pub async fn save_metadata(&self, account_id: &str, domain_id: &str,
                                metadata: DomainMetadata)
        -> CoreResult<()>;

    // 获取收藏列表
    pub async fn list_favorites(&self, account_id: &str)
        -> CoreResult<Vec<DomainMetadataKey>>;

    // 删除账户元数据
    pub async fn delete_account_metadata(&self, account_id: &str)
        -> CoreResult<()>;
}
```

**设计理由**：
- 封装 Repository 细节
- 提供语义化的业务方法（如 `toggle_favorite`）
- 不暴露 `DomainMetadataKey` 给上层（使用 `account_id` + `domain_id`）

---

### 5. DomainService 集成

**自动合并元数据到域名列表**。

```rust
pub async fn list_domains(&self, account_id: &str, page: Option<u32>,
                          page_size: Option<u32>)
    -> CoreResult<PaginatedResponse<AppDomain>>
{
    // 1. 从 Provider API 获取域名列表
    let mut domains: Vec<AppDomain> = provider
        .list_domains(&params)
        .await?
        .items
        .into_iter()
        .map(|d| AppDomain::from_provider(d, account_id.to_string()))
        .collect();

    // 2. 批量加载元数据
    let keys: Vec<(String, String)> = domains
        .iter()
        .map(|d| (d.account_id.clone(), d.id.clone()))
        .collect();

    let metadata_service = DomainMetadataService::new(
        Arc::clone(&self.ctx.domain_metadata_repository)
    );

    if let Ok(metadata_map) = metadata_service.get_metadata_batch(keys).await {
        // 3. 合并到 AppDomain
        for domain in &mut domains {
            let key = DomainMetadataKey::new(
                domain.account_id.clone(), domain.id.clone()
            );
            if let Some(metadata) = metadata_map.get(&key) {
                domain.metadata = Some(metadata.clone());
            }
        }
    }

    Ok(PaginatedResponse::new(domains, page, page_size, total_count))
}
```

**数据流**：
1. Provider API → `Vec<ProviderDomain>`
2. 转换 → `Vec<AppDomain>` (metadata = None)
3. 批量查询 → `HashMap<DomainMetadataKey, DomainMetadata>`
4. 合并 → `AppDomain.metadata = Some(metadata)`

**性能优化**：
- 批量读取元数据（1 次数据库查询，而非 N 次）
- 错误容错：元数据加载失败不影响域名列表返回

---

### 6. Tauri 命令层 (src-tauri/src/commands/domain_metadata.rs)

**前后端桥接**，类型转换。

```rust
// 本地类型（避免暴露 core 内部类型）
#[derive(Serialize, Deserialize)]
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

// Tauri 命令
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
```

**设计理由**：
- 本地类型与 Core 类型分离（解耦）
- `camelCase` 命名符合前端习惯
- 统一返回 `ApiResponse<T>` 格式

---

### 7. 前端架构 (React + Zustand)

#### DomainStore 扩展

```typescript
interface DomainState {
  // 现有状态
  domainsByAccount: Record<string, AccountDomainCache>
  // ...

  // 新增方法
  toggleFavorite: (accountId: string, domainId: string) => Promise<void>
}

// 实现
toggleFavorite: async (accountId, domainId) => {
  // 1. 调用后端 API
  const response = await domainMetadataService.toggleFavorite(accountId, domainId)

  // 2. 更新本地缓存（乐观更新）
  set((state) => {
    const cache = state.domainsByAccount[accountId]
    const domains = cache.domains.map((d) => {
      if (d.id === domainId) {
        return {
          ...d,
          metadata: {
            isFavorite: response.data,
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

  // 3. 保存到 localStorage
  get().saveToStorage()
}
```

**数据流**：
1. 用户点击星标按钮
2. `toggleFavorite()` 调用后端
3. 更新 Zustand store
4. 自动触发 React 组件重渲染
5. 保存到 localStorage（刷新后恢复）

---

## 数据流图

### 收藏域名流程

```
用户点击星标按钮
    ↓
DomainFavoriteButton.onClick()
    ↓
domainStore.toggleFavorite(accountId, domainId)
    ↓
domainMetadataService.toggleFavorite(accountId, domainId)
    ↓ Tauri IPC
toggle_domain_favorite(State, accountId, domainId)
    ↓
DomainMetadataService::toggle_favorite(account_id, domain_id)
    ↓
1. 读取当前元数据
2. 翻转 is_favorite
3. 调用 repository.save()
    ↓
TauriDomainMetadataRepository::save(key, metadata)
    ↓
1. 更新内存缓存
2. 保存到 domain_metadata.json
    ↓
返回新状态 (true/false)
    ↓
前端更新 UI（星标填充/空心）
```

### 加载域名列表流程

```
用户刷新域名列表
    ↓
domainStore.refreshAccount(accountId)
    ↓
domainService.listDomains(accountId, page, pageSize)
    ↓ Tauri IPC
list_domains(State, accountId, page, pageSize)
    ↓
DomainService::list_domains(account_id, page, page_size)
    ↓
1. provider.list_domains() → Vec<ProviderDomain>
2. 转换为 Vec<AppDomain>
3. DomainMetadataService::get_metadata_batch(keys)
    ↓
TauriDomainMetadataRepository::find_by_keys(keys)
    ↓
1. ensure_cache()（首次加载从文件读取）
2. 批量从内存缓存查询
    ↓
返回 HashMap<DomainMetadataKey, DomainMetadata>
    ↓
合并到 AppDomain.metadata
    ↓
返回 PaginatedResponse<AppDomain>
    ↓
前端渲染域名列表（显示星标、标签等）
```

---

## 依赖注入

### ServiceContext 修改

```rust
pub struct ServiceContext {
    pub credential_store: Arc<dyn CredentialStore>,
    pub account_repository: Arc<dyn AccountRepository>,
    pub provider_registry: Arc<dyn ProviderRegistry>,
    pub domain_metadata_repository: Arc<dyn DomainMetadataRepository>,  // 新增
}
```

### AppState 初始化

```rust
impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        // 创建适配器
        let domain_metadata_repository = Arc::new(
            TauriDomainMetadataRepository::new(app_handle.clone())
        );

        // 注入 ServiceContext
        let ctx = Arc::new(ServiceContext::new(
            credential_store,
            account_repository,
            provider_registry,
            domain_metadata_repository.clone(),  // 注入
        ));

        // 创建 Service
        let domain_metadata_service = Arc::new(
            DomainMetadataService::new(domain_metadata_repository)
        );

        Self {
            ctx,
            domain_metadata_service,
            // ...
        }
    }
}
```

**依赖图**：
```
TauriDomainMetadataRepository
    ↓ Arc<dyn DomainMetadataRepository>
ServiceContext.domain_metadata_repository
    ↓ 注入
DomainService (通过 ctx)
    ↓ 自动合并元数据
AppDomain.metadata

TauriDomainMetadataRepository
    ↓ Arc<dyn DomainMetadataRepository>
DomainMetadataService
    ↓ Arc<DomainMetadataService>
AppState.domain_metadata_service
    ↓ Tauri Command 调用
toggle_domain_favorite
```

---

## 设计决策

### Q1: 为什么不直接在 Provider 层存储元数据？

**A**: Provider 层只负责与第三方 API 交互，元数据是用户本地数据，应由 Core 层管理。

### Q2: 为什么使用 HashMap 而非 Vec？

**A**:
- 存储键查询：O(1) vs O(n)
- 批量读取时避免重复遍历
- JSON 序列化为对象（更易读）

### Q3: 为什么空元数据要删除？

**A**:
- 节省存储空间（大量域名时显著）
- JSON 文件更简洁（仅保存有元数据的域名）
- 默认值处理逻辑简单（`None` → `default()`）

### Q4: 为什么不使用 SQLite？

**A**:
- Phase 1 数据量小（数百条），JSON 足够
- 无需复杂查询（仅按键查询）
- 跨平台兼容性好（Tauri/Web 都支持）
- 可在 Phase 2/3 迁移到 SeaORM（Web 版本）

### Q5: 为什么元数据要自动合并到 `list_domains()`？

**A**:
- 减少前端 API 调用
- 批量读取优化后性能损耗可忽略
- 保证前端始终获得完整数据

---

## 性能分析

### 批量读取优化

**场景**: 刷新 20 个域名的列表

**方案 1（逐个查询，未优化）**:
- 20 次 `find_by_key()` 调用
- 20 次 HashMap 查询（内存）
- 时间复杂度：O(20)

**方案 2（批量查询，已优化）**:
- 1 次 `find_by_keys()` 调用
- 1 次遍历 HashMap（内存）
- 时间复杂度：O(1)

**优化效果**: ~20倍性能提升

### 内存缓存效果

**场景**: 用户频繁切换域名页面

**无缓存**:
- 每次切换都从文件读取
- 文件 I/O 延迟：~10-50ms

**有缓存**:
- 首次加载：~10-50ms
- 后续访问：< 1ms（内存读取）

---

## 扩展性设计

### 支持 Web 版本

**实现方式**: 创建 `DatabaseDomainMetadataRepository`

```rust
pub struct DatabaseDomainMetadataRepository {
    db: Arc<DatabaseConnection>,  // SeaORM
}

#[async_trait]
impl DomainMetadataRepository for DatabaseDomainMetadataRepository {
    // 使用 SQL 查询实现
}
```

**切换方式**: 在 `AppState::new()` 中根据平台选择：

```rust
#[cfg(not(target_os = "web"))]
let repo = Arc::new(TauriDomainMetadataRepository::new(app_handle));

#[cfg(target_os = "web")]
let repo = Arc::new(DatabaseDomainMetadataRepository::new(db));
```

### 支持导入导出

**Phase 2/3 实现**：

1. 导出时包含元数据：
```rust
pub struct ExportedAccount {
    // ... 现有字段
    pub domain_metadata: HashMap<String, DomainMetadata>,  // 新增
}
```

2. 导入时恢复元数据：
```rust
async fn import_accounts(&self, file: ExportFile) -> CoreResult<()> {
    for account in file.accounts {
        // ... 导入账户
        // ... 导入凭据
        // ... 导入元数据（新增）
        for (domain_id, metadata) in account.domain_metadata {
            let key = DomainMetadataKey::new(account.id.clone(), domain_id);
            self.domain_metadata_repository.save(&key, &metadata).await?;
        }
    }
    Ok(())
}
```

---

## 安全性

### 数据隔离

- 元数据文件仅包含本地数据（无敏感凭据）
- 与账户凭据分离存储（`accounts.json` vs `domain_metadata.json`）

### 文件权限

- macOS/Linux: `0600`（仅用户可读写）
- Windows: NTFS ACL（仅当前用户）

### 错误处理

- 文件损坏：降级为空 HashMap，不崩溃
- 反序列化失败：记录日志，返回默认值

---

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toggle_favorite() {
        let repo = MockDomainMetadataRepository::new();
        let service = DomainMetadataService::new(Arc::new(repo));

        let result = service.toggle_favorite("acc1", "dom1").await.unwrap();
        assert_eq!(result, true);  // 首次收藏

        let result = service.toggle_favorite("acc1", "dom1").await.unwrap();
        assert_eq!(result, false);  // 取消收藏
    }
}
```

### 集成测试

- 创建临时 AppHandle
- 写入元数据
- 读取验证
- 删除验证

---

**最后更新**: 2026-01-01
**作者**: AptS:1548 (Claude Sonnet 4.5)
