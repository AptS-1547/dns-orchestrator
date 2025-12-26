# TODO

## CredentialStore Trait 重构计划

### 当前实现

```rust
pub type CredentialsMap = HashMap<String, HashMap<String, String>>;

pub trait CredentialStore: Send + Sync {
    async fn load_all(&self) -> CoreResult<CredentialsMap>;
    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()>;
    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>>;
    async fn delete(&self, account_id: &str) -> CoreResult<()>;
    async fn exists(&self, account_id: &str) -> CoreResult<bool>;
}
```

### 计划重构为

```rust
pub type CredentialsMap = HashMap<String, ProviderCredentials>;

pub trait CredentialStore: Send + Sync {
    async fn load_all(&self) -> CoreResult<CredentialsMap>;
    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()>;
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>>;
    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()>;
    async fn remove(&self, account_id: &str) -> CoreResult<()>;
}
```

### 重构原因

1. **类型安全**: 使用 `ProviderCredentials` 枚举代替 `HashMap<String, String>` 可以在编译时捕获凭证类型错误
2. **性能优化**: 减少运行时的字符串查找和解析开销
3. **API 一致性**: 方法命名 `get/set/remove` 更符合 Rust 惯用法

### 影响范围

- `dns-orchestrator-core/src/traits/credential_store.rs`
- 所有 `CredentialStore` 的实现：
  - `KeychainStore` (macOS/Windows/Linux)
  - `StrongholdStore` (Android)
  - `DatabaseCredentialStore` (Web backend)
- 依赖 `CredentialStore` 的服务层代码
