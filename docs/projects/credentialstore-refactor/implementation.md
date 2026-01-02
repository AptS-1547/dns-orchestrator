# CredentialStore Trait 重构 - 实施细节

**Issue**: [#28](https://github.com/AptS-1547/dns-orchestrator/issues/28)
**版本**: v1.7.0
**状态**: ✅ 已完成
**日期**: 2026-01-01

## 实施内容

### 1. Trait 定义重构 ✅

**文件**: `dns-orchestrator-core/src/traits/credential_store.rs`

**变更**:

- 类型安全升级：`HashMap<String, HashMap<String, String>>` → `HashMap<String, ProviderCredentials>`
- 方法重命名：`load()` → `get()`, `save()` → `set()`, `delete()` → `remove()`
- 新增方法：`save_all()`, `load_raw_json()`, `save_raw_json()`

**新类型定义**:

```rust
pub type CredentialsMap = HashMap<String, ProviderCredentials>;
pub type LegacyCredentialsMap = HashMap<String, HashMap<String, String>>;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn load_all(&self) -> CoreResult<CredentialsMap>;
    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()>;
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>>;
    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()>;
    async fn remove(&self, account_id: &str) -> CoreResult<()>;
    async fn load_raw_json(&self) -> CoreResult<String>;
    async fn save_raw_json(&self, json: &str) -> CoreResult<()>;
}
```

### 2. Adapter 实现（双格式支持）✅

**文件**: `src-tauri/src/adapters/credential_store.rs`

**变更**:

- Desktop (Keyring) 和 Android (Store) 支持 V1/V2 格式检测
- 使用 `#[serde(untagged)]` 枚举自动识别格式
- 旧格式返回 `CoreError::MigrationRequired` 触发迁移

**格式检测枚举**:

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum StorageFormat {
    V2(HashMap<String, ProviderCredentials>),  // 新格式
    V1(HashMap<String, HashMap<String, String>>),  // 旧格式
}
```

**实现逻辑**:

```rust
fn read_all_sync() -> CoreResult<CredentialsMap> {
    let json = Self::read_raw_sync()?;
    match serde_json::from_str::<StorageFormat>(&json) {
        Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
        Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
        Err(_) if json.trim().is_empty() => Ok(HashMap::new()),
        Err(e) => Err(CoreError::SerializationError(e.to_string())),
    }
}
```

### 3. 迁移服务 ✅

**文件**: `dns-orchestrator-core/src/services/migration_service.rs` (新建)

**功能**:

- 自动检测旧格式并触发迁移
- 从 `AccountRepository` 获取 provider 类型用于转换
- 记录迁移失败的账户，不中断启动流程

**核心逻辑**:

```rust
pub async fn migrate_if_needed(&self) -> CoreResult<MigrationResult> {
    match self.credential_store.load_all().await {
        Ok(_) => Ok(MigrationResult::NotNeeded),
        Err(CoreError::MigrationRequired) => self.perform_migration().await,
        Err(e) => Err(e),
    }
}
```

### 4. 启动流程集成 ✅

**文件**: `src-tauri/src/lib.rs`

**变更**:

- 在账户恢复前执行迁移（使用 `block_on` 确保完成）
- 详细的日志记录（成功/失败账户数）

**集成代码**:

```rust
// 执行凭证迁移（v1.7.0 - 阻塞操作）
tauri::async_runtime::block_on(async move {
    let state = app_handle.state::<AppState>();
    let migration_service = MigrationService::new(/* ... */);

    match migration_service.migrate_if_needed().await {
        Ok(MigrationResult::NotNeeded) => {
            log::info!("凭证格式检查：无需迁移");
        }
        Ok(MigrationResult::Success { migrated_count, failed_accounts }) => {
            log::info!("凭证迁移成功：{} 个账户已迁移", migrated_count);
            if !failed_accounts.is_empty() {
                log::warn!("部分账户迁移失败: {:?}", failed_accounts);
            }
        }
        Err(e) => log::error!("凭证迁移失败: {}", e),
    }
});
```

### 5. Service 层更新（4 个文件）✅

#### CredentialManagementService

**变更**: 直接使用 `ProviderCredentials`

```rust
pub async fn validate_and_create_provider(
    &self,
    credentials: &ProviderCredentials,  // 不再需要 from_map 转换
) -> CoreResult<Arc<dyn DnsProvider>>
```

#### AccountLifecycleService

**变更**: 移除 `from_map()` 调用，直接使用 `request.credentials`

```rust
pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
    // 旧代码：
    // let credentials = ProviderCredentials::from_map(&request.provider, &request.credentials)?;

    // 新代码：直接使用
    let provider = self.credential_service
        .validate_and_create_provider(&request.credentials)
        .await?;
}
```

#### AccountBootstrapService

**变更**: 使用新的 `CredentialsMap` 类型

#### ImportExportService

**变更**: 使用 `get()/set()/remove()` 方法

```rust
// 导出
let credentials = self.ctx.credential_store.get(&account.id).await?;

// 导入
self.ctx.credential_store.set(&account_id, &credentials).await?;

// 清理
self.ctx.credential_store.remove(&account_id).await;
```

### 6. 错误处理 ✅

**文件**: `dns-orchestrator-core/src/error.rs`, `src-tauri/src/error.rs`

**新增错误类型**:

```rust
pub enum CoreError {
    /// 需要迁移数据格式（v1.7.0 凭证格式升级）
    #[error("Credential data migration required")]
    MigrationRequired,

    /// 迁移失败
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}
```

### 7. 前端类型安全重构 ✅

#### TypeScript 类型定义

**文件**: `src/types/account.ts`

```typescript
export type ProviderCredentials =
  | { provider: "cloudflare"; credentials: { api_token: string } }
  | { provider: "aliyun"; credentials: { access_key_id: string; access_key_secret: string } }
  | { provider: "dnspod"; credentials: { secret_id: string; secret_key: string } }
  | { provider: "huaweicloud"; credentials: { access_key_id: string; secret_access_key: string } }

export interface CreateAccountRequest {
  name: string
  provider: string
  credentials: ProviderCredentials  // 从 Record<string, string> 改为结构化类型
}
```

#### 表单提交逻辑

**文件**: `src/components/account/AccountForm.tsx`

**新增辅助函数**:

```typescript
function buildProviderCredentials(
  provider: string,
  credentialsMap: Record<string, string>
): ProviderCredentials {
  switch (provider) {
    case "cloudflare":
      return {
        provider: "cloudflare",
        credentials: { api_token: credentialsMap.apiToken }
      }
    // ... 其他 providers
  }
}
```

**优化提交逻辑**: 提取 `handleCreate()` 和 `handleUpdate()` 函数降低复杂度

### 8. 请求类型更新 ✅

**文件**: `dns-orchestrator-core/src/types/account.rs`, `src-tauri/src/types.rs`

```rust
pub struct CreateAccountRequest {
    pub name: String,
    pub provider: ProviderType,
    pub credentials: ProviderCredentials,  // 从 HashMap<String, String> 改为 ProviderCredentials
}

pub struct UpdateAccountRequest {
    pub id: String,
    pub name: Option<String>,
    pub credentials: Option<ProviderCredentials>,  // 同上
}
```

## 修改的文件清单（17 个文件）

### Core 库（10 个）

1. `dns-orchestrator-core/src/traits/credential_store.rs` - Trait 定义
2. `dns-orchestrator-core/src/traits/mod.rs` - 导出 `LegacyCredentialsMap`
3. `dns-orchestrator-core/src/services/migration_service.rs` - 迁移服务（新建）
4. `dns-orchestrator-core/src/services/mod.rs` - 导出 `MigrationService`
5. `dns-orchestrator-core/src/services/credential_management_service.rs`
6. `dns-orchestrator-core/src/services/account_lifecycle_service.rs` - 移除 `from_map()` 调用
7. `dns-orchestrator-core/src/services/account_bootstrap_service.rs`
8. `dns-orchestrator-core/src/services/import_export_service.rs`
9. `dns-orchestrator-core/src/error.rs` - 新增迁移错误类型
10. `dns-orchestrator-core/src/types/account.rs` - 请求类型更新为 `ProviderCredentials`

### Tauri 后端（4 个）

11. `src-tauri/src/adapters/credential_store.rs` - Adapter 实现
12. `src-tauri/src/error.rs` - 错误映射
13. `src-tauri/src/lib.rs` - 启动流程集成
14. `src-tauri/src/types.rs` - Tauri IPC 层类型更新为 `ProviderCredentials`

### 前端（3 个）

15. `src/types/account.ts` - TypeScript 类型定义
16. `src/components/account/AccountForm.tsx` - 表单提交逻辑（新增 `buildProviderCredentials` 函数）
17. `docs/TODO.md` - 文档更新

---

**测试状态**:

- ✅ Rust 编译通过
- ✅ TypeScript 类型检查通过
- ✅ 创建账户功能测试通过
- ✅ 更新账户功能测试通过

**最后更新**: 2026-01-01
