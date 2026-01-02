# 域名元数据系统 - API 参考

本文档提供域名元数据系统的完整 API 参考，包括后端 Rust API 和前端 TypeScript API。

---

## 后端 API (Rust)

### DomainMetadataRepository Trait

**位置**: `dns-orchestrator-core/src/traits/domain_metadata_repository.rs`

#### find_by_key

```rust
async fn find_by_key(&self, key: &DomainMetadataKey)
    -> CoreResult<Option<DomainMetadata>>
```

查询单个域名的元数据。

**参数**:
- `key` - 域名元数据键

**返回**:
- `Some(metadata)` - 找到元数据
- `None` - 未找到（应使用默认值）

**示例**:
```rust
let key = DomainMetadataKey::new("account-123".to_string(), "domain-456".to_string());
let metadata = repository.find_by_key(&key).await?;
```

#### find_by_keys

```rust
async fn find_by_keys(&self, keys: &[DomainMetadataKey])
    -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>
```

批量查询多个域名的元数据（性能优化）。

**参数**:
- `keys` - 域名元数据键列表

**返回**:
- `HashMap` - 键值对映射（仅包含存在的元数据）

**示例**:
```rust
let keys = vec![
    DomainMetadataKey::new("acc1".to_string(), "dom1".to_string()),
    DomainMetadataKey::new("acc1".to_string(), "dom2".to_string()),
];
let metadata_map = repository.find_by_keys(&keys).await?;
```

**注意**: 返回的 HashMap 仅包含存在的元数据，不存在的键不会出现在结果中。

#### save

```rust
async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata)
    -> CoreResult<()>
```

保存或更新元数据。

**参数**:
- `key` - 域名元数据键
- `metadata` - 元数据对象

**行为**:
- 如果 `metadata.is_empty()` 为 `true`，删除存储条目
- 否则，保存或更新

**示例**:
```rust
let key = DomainMetadataKey::new("acc1".to_string(), "dom1".to_string());
let mut metadata = DomainMetadata::default();
metadata.is_favorite = true;
repository.save(&key, &metadata).await?;
```

#### delete

```rust
async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()>
```

删除元数据。

**参数**:
- `key` - 域名元数据键

**示例**:
```rust
let key = DomainMetadataKey::new("acc1".to_string(), "dom1".to_string());
repository.delete(&key).await?;
```

#### delete_by_account

```rust
async fn delete_by_account(&self, account_id: &str) -> CoreResult<()>
```

删除账户下的所有元数据（账户删除时调用）。

**参数**:
- `account_id` - 账户 ID

**示例**:
```rust
repository.delete_by_account("account-123").await?;
```

#### find_favorites_by_account

```rust
async fn find_favorites_by_account(&self, account_id: &str)
    -> CoreResult<Vec<DomainMetadataKey>>
```

获取账户下所有收藏的域名键。

**参数**:
- `account_id` - 账户 ID

**返回**:
- `Vec<DomainMetadataKey>` - 收藏的域名键列表

**示例**:
```rust
let favorites = repository.find_favorites_by_account("account-123").await?;
for key in favorites {
    println!("Favorite: {}/{}", key.account_id, key.domain_id);
}
```

---

### DomainMetadataService

**位置**: `dns-orchestrator-core/src/services/domain_metadata_service.rs`

#### get_metadata

```rust
pub async fn get_metadata(&self, account_id: &str, domain_id: &str)
    -> CoreResult<DomainMetadata>
```

获取域名元数据（不存在则返回默认值）。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID

**返回**:
- `DomainMetadata` - 元数据对象（保证非空）

**示例**:
```rust
let metadata = service.get_metadata("acc1", "dom1").await?;
println!("Is favorite: {}", metadata.is_favorite);
```

#### get_metadata_batch

```rust
pub async fn get_metadata_batch(&self, keys: Vec<(String, String)>)
    -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>>
```

批量获取元数据（用于域名列表，性能优化）。

**参数**:
- `keys` - `(account_id, domain_id)` 对的列表

**返回**:
- `HashMap<DomainMetadataKey, DomainMetadata>` - 键值对映射

**示例**:
```rust
let keys = vec![
    ("acc1".to_string(), "dom1".to_string()),
    ("acc1".to_string(), "dom2".to_string()),
];
let metadata_map = service.get_metadata_batch(keys).await?;
```

**用途**: 主要供 `DomainService::list_domains()` 调用。

#### save_metadata

```rust
pub async fn save_metadata(&self, account_id: &str, domain_id: &str,
                           metadata: DomainMetadata) -> CoreResult<()>
```

保存元数据（全量更新）。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID
- `metadata` - 元数据对象

**示例**:
```rust
let metadata = DomainMetadata {
    is_favorite: true,
    tags: vec!["production".to_string()],
    ..Default::default()
};
service.save_metadata("acc1", "dom1", metadata).await?;
```

#### update_metadata

```rust
pub async fn update_metadata(&self, account_id: &str, domain_id: &str,
                             update: DomainMetadataUpdate) -> CoreResult<()>
```

更新元数据（部分更新，Phase 2/3 使用）。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID
- `update` - 更新请求对象

**示例**:
```rust
let update = DomainMetadataUpdate {
    is_favorite: Some(true),  // 仅更新收藏状态
    ..Default::default()
};
service.update_metadata("acc1", "dom1", update).await?;
```

#### toggle_favorite

```rust
pub async fn toggle_favorite(&self, account_id: &str, domain_id: &str)
    -> CoreResult<bool>
```

切换收藏状态。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID

**返回**:
- `bool` - 新的收藏状态（`true` = 已收藏，`false` = 未收藏）

**示例**:
```rust
let is_favorite = service.toggle_favorite("acc1", "dom1").await?;
if is_favorite {
    println!("Domain is now favorited");
} else {
    println!("Domain is now unfavorited");
}
```

#### list_favorites

```rust
pub async fn list_favorites(&self, account_id: &str)
    -> CoreResult<Vec<DomainMetadataKey>>
```

获取账户下的收藏域名键。

**参数**:
- `account_id` - 账户 ID

**返回**:
- `Vec<DomainMetadataKey>` - 收藏的域名键列表

**示例**:
```rust
let favorites = service.list_favorites("account-123").await?;
println!("Total favorites: {}", favorites.len());
```

#### delete_account_metadata

```rust
pub async fn delete_account_metadata(&self, account_id: &str)
    -> CoreResult<()>
```

删除账户下的所有元数据（账户删除时调用）。

**参数**:
- `account_id` - 账户 ID

**示例**:
```rust
// 在 AccountLifecycleService::delete_account() 中调用
service.delete_account_metadata("account-123").await?;
```

---

### Tauri 命令

**位置**: `src-tauri/src/commands/domain_metadata.rs`

#### get_domain_metadata

```rust
#[tauri::command]
pub async fn get_domain_metadata(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<DomainMetadata>, DnsError>
```

获取域名元数据。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID

**返回**:
```json
{
  "success": true,
  "data": {
    "isFavorite": true,
    "tags": ["production"],
    "updatedAt": 1704067200000
  }
}
```

**前端调用**:
```typescript
const response = await transport.invoke("get_domain_metadata", {
  accountId: "acc1",
  domainId: "dom1",
})
```

#### toggle_domain_favorite

```rust
#[tauri::command]
pub async fn toggle_domain_favorite(
    state: State<'_, AppState>,
    account_id: String,
    domain_id: String,
) -> Result<ApiResponse<bool>, DnsError>
```

切换收藏状态。

**参数**:
- `account_id` - 账户 ID
- `domain_id` - 域名 ID

**返回**:
```json
{
  "success": true,
  "data": true  // 新的收藏状态
}
```

**前端调用**:
```typescript
const response = await transport.invoke("toggle_domain_favorite", {
  accountId: "acc1",
  domainId: "dom1",
})
console.log("Is favorite:", response.data)
```

#### list_account_favorite_domain_keys

```rust
#[tauri::command]
pub async fn list_account_favorite_domain_keys(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<ApiResponse<Vec<String>>, DnsError>
```

获取账户下的收藏域名 ID 列表。

**参数**:
- `account_id` - 账户 ID

**返回**:
```json
{
  "success": true,
  "data": ["domain-1", "domain-2", "domain-3"]
}
```

**前端调用**:
```typescript
const response = await transport.invoke("list_account_favorite_domain_keys", {
  accountId: "acc1",
})
console.log("Favorite domain IDs:", response.data)
```

---

## 前端 API (TypeScript)

### DomainMetadataService

**位置**: `src/services/domainMetadata.service.ts`

#### getMetadata

```typescript
async getMetadata(accountId: string, domainId: string):
    Promise<ApiResponse<DomainMetadata>>
```

获取域名元数据。

**参数**:
- `accountId` - 账户 ID
- `domainId` - 域名 ID

**返回**:
```typescript
{
  success: true,
  data: {
    isFavorite: true,
    tags: ["production"],
    color: "#FF5733",
    note: "重要域名",
    updatedAt: 1704067200000
  }
}
```

**示例**:
```typescript
const response = await domainMetadataService.getMetadata("acc1", "dom1")
if (response.success) {
  console.log("Is favorite:", response.data.isFavorite)
}
```

#### toggleFavorite

```typescript
async toggleFavorite(accountId: string, domainId: string):
    Promise<ApiResponse<boolean>>
```

切换收藏状态。

**参数**:
- `accountId` - 账户 ID
- `domainId` - 域名 ID

**返回**:
```typescript
{
  success: true,
  data: true  // 新的收藏状态
}
```

**示例**:
```typescript
const response = await domainMetadataService.toggleFavorite("acc1", "dom1")
if (response.success) {
  console.log("New favorite state:", response.data)
}
```

#### listAccountFavorites

```typescript
async listAccountFavorites(accountId: string):
    Promise<ApiResponse<string[]>>
```

获取账户下的收藏域名 ID 列表。

**参数**:
- `accountId` - 账户 ID

**返回**:
```typescript
{
  success: true,
  data: ["domain-1", "domain-2", "domain-3"]
}
```

**示例**:
```typescript
const response = await domainMetadataService.listAccountFavorites("acc1")
if (response.success) {
  console.log("Total favorites:", response.data.length)
}
```

---

### DomainStore

**位置**: `src/stores/domainStore.ts`

#### toggleFavorite

```typescript
toggleFavorite: (accountId: string, domainId: string) => Promise<void>
```

切换收藏状态并更新本地缓存。

**参数**:
- `accountId` - 账户 ID
- `domainId` - 域名 ID

**行为**:
1. 调用后端 API
2. 更新 Zustand store
3. 保存到 localStorage
4. 触发 React 组件重渲染

**示例**:
```typescript
// 在组件中使用
const toggleFavorite = useDomainStore((state) => state.toggleFavorite)

const handleClick = () => {
  toggleFavorite(accountId, domainId)
}
```

**错误处理**:
- 失败时打印错误到控制台
- 不阻塞 UI（静默失败）

---

## 类型定义

### DomainMetadata (TypeScript)

**位置**: `src/types/domain-metadata.ts`

```typescript
export interface DomainMetadata {
  /** 是否收藏 */
  isFavorite: boolean

  /** 标签列表（Phase 2） */
  tags: string[]

  /** 颜色标记（Phase 3，HEX 格式如 "#FF5733"） */
  color?: string

  /** 备注（Phase 3） */
  note?: string

  /** 最后修改时间（Unix 时间戳，毫秒） */
  updatedAt: number
}
```

**默认值**:
```typescript
{
  isFavorite: false,
  tags: [],
  color: undefined,
  note: undefined,
  updatedAt: Date.now()
}
```

### DomainMetadataUpdate (TypeScript)

**位置**: `src/types/domain-metadata.ts`

```typescript
export interface DomainMetadataUpdate {
  isFavorite?: boolean
  tags?: string[]
  /** null 表示清空字段 */
  color?: string | null
  /** null 表示清空字段 */
  note?: string | null
}
```

**示例**:
```typescript
// 仅更新收藏状态
const update: DomainMetadataUpdate = {
  isFavorite: true
}

// 清空颜色
const update: DomainMetadataUpdate = {
  color: null
}

// 更新多个字段
const update: DomainMetadataUpdate = {
  isFavorite: true,
  tags: ["production", "important"],
  color: "#FF5733"
}
```

### DomainMetadata (Rust)

**位置**: `dns-orchestrator-core/src/types/domain_metadata.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    #[serde(default)]
    pub is_favorite: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    pub updated_at: i64,
}
```

**方法**:
```rust
impl DomainMetadata {
    // 刷新更新时间
    pub fn touch(&mut self);

    // 是否为空元数据
    pub fn is_empty(&self) -> bool;

    // 创建新实例
    pub fn new(is_favorite: bool, tags: Vec<String>,
               color: Option<String>, note: Option<String>) -> Self;
}
```

### DomainMetadataKey (Rust)

**位置**: `dns-orchestrator-core/src/types/domain_metadata.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataKey {
    pub account_id: String,
    pub domain_id: String,
}
```

**方法**:
```rust
impl DomainMetadataKey {
    // 创建新键
    pub fn new(account_id: String, domain_id: String) -> Self;

    // 生成存储键（"account_id::domain_id"）
    pub fn to_storage_key(&self) -> String;

    // 从存储键解析
    pub fn from_storage_key(key: &str) -> Option<Self>;
}
```

---

## 错误处理

### CoreError 变体

**位置**: `dns-orchestrator-core/src/error.rs`

```rust
pub enum CoreError {
    StorageError(String),       // 存储操作失败
    SerializationError(String), // JSON 序列化/反序列化失败
    // ... 其他变体
}
```

**错误示例**:
```rust
// 文件访问失败
CoreError::StorageError("Failed to access store: ...")

// JSON 解析失败
CoreError::SerializationError("Failed to deserialize metadata: ...")
```

### 前端错误处理

**ApiResponse 结构**:
```typescript
{
  success: boolean
  data?: T
  error?: string
}
```

**错误示例**:
```typescript
const response = await domainMetadataService.toggleFavorite("acc1", "dom1")
if (!response.success) {
  console.error("Failed to toggle favorite:", response.error)
  // 显示错误提示
  toast.error(response.error || "操作失败")
}
```

---

## 性能建议

### 批量操作

❌ **不推荐**（逐个查询）:
```rust
for domain in &domains {
    let metadata = service.get_metadata(&domain.account_id, &domain.id).await?;
    domain.metadata = Some(metadata);
}
```

✅ **推荐**（批量查询）:
```rust
let keys: Vec<(String, String)> = domains
    .iter()
    .map(|d| (d.account_id.clone(), d.id.clone()))
    .collect();

let metadata_map = service.get_metadata_batch(keys).await?;

for domain in &mut domains {
    let key = DomainMetadataKey::new(domain.account_id.clone(), domain.id.clone());
    if let Some(metadata) = metadata_map.get(&key) {
        domain.metadata = Some(metadata.clone());
    }
}
```

### 前端缓存

✅ **推荐**: 使用 Zustand store 缓存元数据，避免重复调用 API。

```typescript
// 第一次加载：调用 API
const response = await domainService.listDomains(accountId, 1, 20)
// response.data.items 已包含 metadata

// 后续访问：从 domainStore 读取缓存
const domains = useDomainStore(state => state.domainsByAccount[accountId]?.domains)
```

---

## 迁移指南（未来）

### Phase 1 → Phase 2

**新增 API**:
```typescript
// 添加标签
await domainMetadataService.addTag(accountId, domainId, "production")

// 移除标签
await domainMetadataService.removeTag(accountId, domainId, "production")

// 按标签查询
await domainMetadataService.findByTag("production")
```

**Rust 实现**:
```rust
impl DomainMetadataService {
    pub async fn add_tag(&self, account_id: &str, domain_id: &str, tag: String)
        -> CoreResult<()>;

    pub async fn remove_tag(&self, account_id: &str, domain_id: &str, tag: &str)
        -> CoreResult<()>;

    pub async fn find_by_tag(&self, tag: &str)
        -> CoreResult<Vec<DomainMetadataKey>>;
}
```

### Phase 2 → Phase 3

**新增 API**:
```typescript
// 更新元数据（部分更新）
await domainMetadataService.updateMetadata(accountId, domainId, {
  color: "#FF5733",
  note: "重要域名",
})
```

**Rust 实现**:
```rust
// 已在 Phase 1 预留
pub async fn update_metadata(&self, account_id: &str, domain_id: &str,
                             update: DomainMetadataUpdate) -> CoreResult<()>
```

---

## 常见问题

### Q1: 如何判断域名是否收藏？

```typescript
const domain: Domain = ...
const isFavorite = domain.metadata?.isFavorite ?? false
```

### Q2: 如何获取所有收藏域名的完整信息？

```typescript
// 1. 获取收藏的 domainId 列表
const response = await domainMetadataService.listAccountFavorites(accountId)

// 2. 从 domainStore 中查找完整 Domain 对象
const allDomains = useDomainStore(state => state.domainsByAccount[accountId]?.domains)
const favoriteDomains = allDomains.filter(d => response.data.includes(d.id))
```

### Q3: 如何清空可选字段？

```rust
// Rust
let update = DomainMetadataUpdate {
    color: Some(None),  // 清空颜色
    note: Some(None),   // 清空备注
    ..Default::default()
};
service.update_metadata("acc1", "dom1", update).await?;
```

```typescript
// TypeScript
await domainMetadataService.updateMetadata(accountId, domainId, {
  color: null,  // 清空颜色
  note: null,   // 清空备注
})
```

---

**最后更新**: 2026-01-01
**作者**: AptS:1548 (Claude Sonnet 4.5)
