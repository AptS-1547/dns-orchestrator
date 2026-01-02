# v2.0.0 清理计划

**目标**: 移除 v1.7.0 引入的数据迁移兼容代码
**影响**: v2.0.0 将不再支持从 v1.6.x 直接升级
**前提**: 用户必须先升级到 v1.7.x

## 数据兼容性说明

### 版本支持矩阵

| 源版本 | 目标版本 | 支持情况 | 说明 |
|--------|----------|----------|------|
| v1.6.x | v1.7.x | ✅ 自动迁移 | 启动时自动执行数据格式迁移 |
| v1.7.x | v2.0.x | ✅ 无需迁移 | 数据格式相同，直接升级 |
| v1.6.x | v2.0.x | ❌ **不支持** | 必须先升级到 v1.7.x |

### 升级路径

```
v1.6.x → v1.7.x → v2.0.x ✅

v1.6.x → v2.0.x ❌ (数据丢失)
```

## 需要删除的代码

### 1. 删除 `ProviderCredentials::from_map()` 和 `to_map()` 方法

**文件**: `dns-orchestrator-provider/src/types.rs` (L448-567)

**操作**:

```rust
// 删除整个 impl ProviderCredentials 块中的这两个方法
pub fn from_map(...) { ... }  // 删除
pub fn to_map(&self) { ... }  // 删除
```

**原因**: 这两个方法仅用于 v1.6.x 到 v1.7.0 的迁移，v2.0.0 不再需要。

### 2. 删除 `LegacyCredentialsMap` 类型别名

**文件 1**: `dns-orchestrator-core/src/traits/credential_store.rs` (L13)

```rust
pub type LegacyCredentialsMap = HashMap<String, HashMap<String, String>>;  // 删除整行
```

**文件 2**: `dns-orchestrator-core/src/traits/mod.rs` (L8)

```rust
pub use credential_store::{CredentialStore, CredentialsMap, LegacyCredentialsMap};
// 改为：
pub use credential_store::{CredentialStore, CredentialsMap};
```

### 3. 删除迁移服务

**文件 1**: `dns-orchestrator-core/src/services/migration_service.rs`

- **操作**: 删除整个文件

**文件 2**: `dns-orchestrator-core/src/services/mod.rs` (L10, L21)

```rust
mod migration_service;  // 删除
pub use migration_service::{MigrationResult, MigrationService};  // 删除
```

### 4. 删除启动流程中的迁移逻辑

**文件**: `src-tauri/src/lib.rs` (L156-191)

**操作**:

```rust
// 删除整个迁移代码块（第 156-191 行）
// 执行凭证迁移（v1.7.0 - 阻塞操作，确保迁移完成后再恢复账户）
tauri::async_runtime::block_on(async move {
    // ... 删除整个 block_on 块
});
```

**同时删除导入**:

```rust
use dns_orchestrator_core::services::{
    // ... 其他导入 ...
    MigrationResult, MigrationService,  // 删除这两个
};
```

### 5. 删除 Adapter 中的双格式支持

**文件**: `src-tauri/src/adapters/credential_store.rs`

#### 5.1 删除格式检测枚举（L14-19）

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum StorageFormat {
    V2(HashMap<String, ProviderCredentials>),
    V1(HashMap<String, HashMap<String, String>>),
}
// 删除整个枚举定义
```

#### 5.2 简化反序列化方法

**Desktop 版本** (`read_all_sync()` 方法，约 L75-85):

```rust
// 旧代码（需删除）：
fn read_all_sync() -> CoreResult<CredentialsMap> {
    let json = Self::read_raw_sync()?;
    match serde_json::from_str::<StorageFormat>(&json) {
        Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
        Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
        Err(_) if json.trim().is_empty() || json.trim() == "{}" => Ok(HashMap::new()),
        Err(e) => Err(CoreError::SerializationError(e.to_string())),
    }
}

// 新代码（简化版）：
fn read_all_sync() -> CoreResult<CredentialsMap> {
    let json = Self::read_raw_sync()?;
    // 直接反序列化，不需要格式检测
    serde_json::from_str(&json)
        .map_err(|e| CoreError::SerializationError(e.to_string()))
}
```

**Android 版本** (`load_from_store()` 方法，约 L254-264):

```rust
// 旧代码（需删除）：
fn load_from_store(&self) -> CoreResult<CredentialsMap> {
    let json = self.load_raw_from_store()?;
    match serde_json::from_str::<StorageFormat>(&json) {
        Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
        Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
        Err(_) if json.trim().is_empty() || json.trim() == "{}" => Ok(HashMap::new()),
        Err(e) => Err(CoreError::SerializationError(e.to_string())),
    }
}

// 新代码（简化版）：
fn load_from_store(&self) -> CoreResult<CredentialsMap> {
    let json = self.load_raw_from_store()?;
    // 直接反序列化，不需要格式检测
    serde_json::from_str(&json)
        .map_err(|e| CoreError::SerializationError(e.to_string()))
}
```

#### 5.3 删除类型导入（L11）

```rust
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap, LegacyCredentialsMap};
// 改为：
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap};
```

### 6. 删除迁移相关错误类型

**文件 1**: `dns-orchestrator-core/src/error.rs` (约 L73-79)

```rust
/// 需要迁移数据格式（v1.7.0 凭证格式升级）
#[error("Credential data migration required")]
MigrationRequired,

/// 迁移失败
#[error("Migration failed: {0}")]
MigrationFailed(String),
// 删除这两个错误变体
```

**文件 2**: `src-tauri/src/error.rs` (约 L89-93)

```rust
// v1.7.0 迁移相关错误
CoreError::MigrationRequired => {
    Self::CredentialError("Credential migration required".to_string())
}
CoreError::MigrationFailed(s) => Self::CredentialError(format!("Migration failed: {s}")),
// 删除这两个匹配分支
```

### 7. 可选：删除迁移辅助方法

**文件**: `dns-orchestrator-core/src/traits/credential_store.rs` (约 L68-79)

```rust
/// 加载原始 JSON（用于格式检测和迁移）
async fn load_raw_json(&self) -> CoreResult<String>;

/// 保存原始 JSON（用于迁移写入）
async fn save_raw_json(&self, json: &str) -> CoreResult<()>;
// 如果这两个方法没有其他用途，可以删除
```

**如果删除，同时删除 Adapter 中的实现**:

- Desktop: 约 L172-183
- Android: 约 L334-340

## 清理检查清单

在发布 v2.0.0 之前，确保完成以下步骤：

- [ ] 删除 `ProviderCredentials::from_map()` 和 `to_map()` 方法
- [ ] 删除 `LegacyCredentialsMap` 类型别名（2 处）
- [ ] 删除 `migration_service.rs` 文件
- [ ] 删除 `MigrationService` 导入和使用（2 处）
- [ ] 删除启动流程中的迁移代码块
- [ ] 删除 `StorageFormat` 枚举
- [ ] 简化 Adapter 的 `read_all_sync()` 和 `load_from_store()` 方法
- [ ] 删除迁移错误类型（2 个错误变体 + 2 处映射）
- [ ] 可选：删除 `load_raw_json()` 和 `save_raw_json()` 方法
- [ ] 运行全部测试确保删除后无遗留引用
- [ ] 更新 CHANGELOG.md 说明移除了迁移代码
- [ ] 更新文档说明不再支持 v1.6.x 及以下版本的自动迁移

## Release Notes 建议

### v1.7.0 Release Notes

```markdown
## 重要变更：凭证存储格式升级

v1.7.0 引入了类型安全的凭证存储格式。从 v1.6.x 升级时会自动迁移数据。

**升级建议**：
- ✅ 推荐所有 v1.6.x 用户升级到 v1.7.0
- ✅ v2.0.0 将不再支持从 v1.6.x 直接升级
- ✅ 迁移过程自动完成，无需手动操作

**如果迁移失败**：
- 失败的账户会被标记为 Error 状态
- 可以手动重新输入凭证来修复
```

### v2.0.0 Release Notes

```markdown
## 破坏性变更：移除 v1.6.x 迁移支持

v2.0.0 已移除 v1.7.0 中用于数据迁移的兼容代码。

**重要提示**：
- ❌ v2.0.0 不支持从 v1.6.x 直接升级
- ✅ v1.6.x 用户必须先升级到 v1.7.x，再升级到 v2.0.0
- ✅ v1.7.x 用户可以直接升级到 v2.0.0

**升级路径**：
1. v1.6.x → v1.7.x → v2.0.x ✅
2. v1.7.x → v2.0.x ✅
3. v1.6.x → v2.0.x ❌ **不支持**

**代码清理**：
- 移除了 MigrationService 和相关迁移逻辑
- 移除了 LegacyCredentialsMap 类型别名
- 简化了 CredentialStore 适配器实现
```

## 影响评估

### 正面影响

1. **代码简化**: 移除约 300 行迁移相关代码
2. **维护成本降低**: 不再需要维护双格式支持
3. **性能提升**: 启动时不再执行格式检测和迁移
4. **类型安全**: 完全移除旧的 HashMap 格式

### 潜在风险

1. **用户升级路径**: 需要在文档中明确说明升级路径
2. **数据丢失风险**: 从 v1.6.x 直接升级到 v2.0.0 会导致凭证丢失
3. **用户沟通**: 需要提前通知用户升级到 v1.7.x

### 风险缓解措施

1. **提前通知**: 在 v1.7.0 release notes 中强调升级重要性
2. **版本检查**: 考虑在 v2.0.0 启动时检测旧格式并给出明确错误提示
3. **文档更新**: 更新升级指南，明确标注升级路径

---

**最后更新**: 2026-01-01
**计划发布**: v2.0.0（待定）
