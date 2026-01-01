# TODO

## Toolbox 功能扩展

### 已计划实现（优先级高）
- [x] DNS 查询
- [x] WHOIS 查询
- [x] SSL 证书检查
- [x] IP 地理位置查询
- [x] **HTTP 头检查器** - 显示响应头 + 安全头分析 + 自定义请求头
- [x] **DNS 传播检查** - 从全球多个 DNS 查询，检测传播状态
- [x] **DNSSEC 验证** - 检查 DNSSEC 启用状态和密钥信息

### 待评估功能（优先级中）

#### 1. 端口扫描器 (Port Scanner)
**用途**: 检测指定端口是否开放
**场景**: 确认服务是否运行、防火墙配置
**实现**: TCP 连接测试（复用 SSL 检查的 TCP 逻辑）
**难度**: ⭐⭐ 简单

#### 2. Ping / 连通性测试
**用途**: 测试主机可达性和延迟
**场景**: 检查服务器在线状态
**实现**: ICMP ping 或 TCP ping
**难度**: ⭐⭐⭐ 中等偏高（需要原生实现或系统调用）

#### 4. 子域名发现 (Subdomain Finder)
**用途**: 发现域名的子域名
**场景**: 安全审计、资产盘点
**实现**: 字典爆破 + 证书透明度日志（crt.sh API）
**难度**: ⭐⭐⭐ 中等偏高（涉及并发查询）

#### 5. Traceroute
**用途**: 路由追踪，查看数据包路径
**场景**: 网络诊断、延迟分析
**实现**: 系统 traceroute 或原生实现（TTL 递增）
**难度**: ⭐⭐⭐⭐ 较高（需要原始套接字）

#### 6. 批量 DNS 查询
**用途**: 一次查询多个域名
**场景**: 批量检查域名解析
**实现**: 复用现有 DNS 查询
**难度**: ⭐⭐ 简单

### 待评估功能（优先级低）

#### 7. Nslookup 模拟器
**用途**: 命令行风格的 DNS 查询
**难度**: ⭐⭐ 简单

#### 8. 网络工具集
- Base64 编解码
- 哈希计算 (MD5/SHA256)
- URL 编解码
**难度**: ⭐ 非常简单

---

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
