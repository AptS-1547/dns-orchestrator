# dns-orchestrator-core 架构文档

本文档描述 `dns-orchestrator-core` 库的架构设计，这是一个平台无关的 DNS 管理业务逻辑层。

## 目录

- [概述](#概述)
- [目录结构](#目录结构)
- [核心架构](#核心架构)
  - [设计理念](#设计理念)
  - [ServiceContext](#servicecontext)
  - [核心 Trait](#核心-trait)
- [类型系统](#类型系统)
  - [账户类型](#账户类型)
  - [域名类型](#域名类型)
  - [导入导出类型](#导入导出类型)
  - [工具箱类型](#工具箱类型)
- [服务层](#服务层)
  - [AccountBootstrapService](#accountbootstrapservice)
  - [AccountLifecycleService](#accountlifecycleservice)
  - [AccountMetadataService](#accountmetadataservice)
  - [CredentialManagementService](#credentialmanagementservice)
  - [DnsService](#dnsservice)
  - [DomainService](#domainservice)
  - [ImportExportService](#importexportservice)
  - [ProviderMetadataService](#providermetadataservice)
  - [ToolboxService](#toolboxservice)
- [错误处理](#错误处理)
- [加密模块](#加密模块)
- [数据流图](#数据流图)
- [平台适配指南](#平台适配指南)

---

## 概述

`dns-orchestrator-core` 是 DNS Orchestrator 的核心业务逻辑库，提供：

- **平台无关设计**：支持 Tauri（桌面/移动端）和 Actix-Web（服务器）后端
- **基于 Trait 的存储抽象**：存储实现在运行时注入
- **账户和凭证管理**：安全处理 DNS 服务商账户
- **多服务商支持**：统一接口管理 Cloudflare、阿里云、DNSPod 和华为云

### 与 dns-orchestrator-provider 的关系

```
┌─────────────────────────────────────┐
│           平台层                     │
│   (Tauri / Actix-Web 后端)          │
├─────────────────────────────────────┤
│       dns-orchestrator-core         │  ← 本库
│      (业务逻辑与服务层)              │
├─────────────────────────────────────┤
│     dns-orchestrator-provider       │
│     (DNS 服务商实现)                 │
└─────────────────────────────────────┘
```

- **dns-orchestrator-provider**：定义 `DnsProvider` trait，实现各厂商 DNS API 调用
- **dns-orchestrator-core**：编排业务逻辑，管理账户/凭证，调用 Provider API

---

## 目录结构

```
src/
├── lib.rs                    # 库入口，re-export 公共 API
├── error.rs                  # 统一错误类型 (CoreError, CoreResult)
│
├── types/                    # 数据类型定义
│   ├── mod.rs
│   ├── account.rs           # Account, AccountStatus, CreateAccountRequest
│   ├── domain.rs            # AppDomain（扩展 ProviderDomain，添加 account_id）
│   ├── export.rs            # ExportFile, ImportPreview, ExportedAccount
│   ├── response.rs          # API 响应封装
│   └── toolbox.rs           # WhoisResult, DnsLookupResult, IpLookupResult, SslCheckResult
│
├── traits/                   # 存储层抽象 Trait
│   ├── mod.rs
│   ├── account_repository.rs    # AccountRepository trait
│   ├── credential_store.rs      # CredentialStore trait
│   └── provider_registry.rs     # ProviderRegistry trait + InMemoryProviderRegistry
│
├── services/                 # 业务逻辑服务层
│   ├── mod.rs                   # ServiceContext 定义
│   ├── account_bootstrap_service.rs      # 启动账户恢复
│   ├── account_lifecycle_service.rs      # 账户 CRUD 操作
│   ├── account_metadata_service.rs       # 账户查询
│   ├── credential_management_service.rs  # 凭证验证与管理
│   ├── domain_service.rs                 # 域名列表
│   ├── dns_service.rs                    # DNS 记录 CRUD
│   ├── import_export_service.rs          # 账户导入导出（支持加密）
│   ├── provider_metadata_service.rs      # Provider 元数据查询
│   └── toolbox/                          # 工具服务
│       ├── mod.rs
│       ├── dns.rs                        # DNS 查询
│       ├── ip.rs                         # IP 地理位置
│       ├── ssl.rs                        # SSL 证书检查
│       ├── whois.rs                      # WHOIS 查询
│       └── whois_servers.json            # WHOIS 服务器配置
│
├── crypto/                   # 加密模块
│   ├── mod.rs               # AES-256-GCM 加密/解密
│   └── versions.rs          # 加密算法版本管理
│
└── utils/                    # 工具函数
    ├── mod.rs
    └── datetime.rs          # DateTime 序列化辅助
```

---

## 核心架构

### 设计理念

本库采用**依赖注入**模式实现平台无关性：

1. **定义抽象 Trait**：用于存储操作
2. **平台层注入具体实现**：在应用启动时注入
3. **服务通过 Trait 接口操作**：不感知底层存储细节

这使得相同的业务逻辑可以运行在：
- **Tauri 桌面端**：使用系统钥匙串（`keyring` crate）和本地存储（`tauri-plugin-store`）
- **Tauri Android**：使用 Stronghold 安全存储
- **Actix-Web**：使用数据库（SeaORM）+ AES 加密

### ServiceContext

`ServiceContext` 是依赖注入容器，持有所有存储实现：

```rust
// src/services/mod.rs

pub struct ServiceContext {
    pub credential_store: Arc<dyn CredentialStore>,
    pub account_repository: Arc<dyn AccountRepository>,
    pub provider_registry: Arc<dyn ProviderRegistry>,
}

impl ServiceContext {
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
        provider_registry: Arc<dyn ProviderRegistry>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
            provider_registry,
        }
    }

    /// 通过账户 ID 获取 Provider 实例
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::ProviderNotFound(account_id.to_string()))
    }

    /// 标记账户为无效状态（如凭证过期）
    pub async fn mark_account_invalid(&self, account_id: &str, error_msg: &str) {
        let _ = self.account_repository
            .update_status(account_id, AccountStatus::Error, Some(error_msg.to_string()))
            .await;
    }
}
```

**使用模式**：

```rust
// 平台层创建 ServiceContext，注入具体实现
let ctx = Arc::new(ServiceContext::new(
    Arc::new(KeychainStore::new()),           // 平台特定实现
    Arc::new(TauriAccountRepository::new()),  // 平台特定实现
    Arc::new(InMemoryProviderRegistry::new()), // 通用实现
));

// 服务接收 ServiceContext
let lifecycle_service = AccountLifecycleService::new(ctx.clone());
let dns_service = DnsService::new(ctx.clone());
```

### 核心 Trait

#### 1. AccountRepository

抽象账户元数据持久化（CRUD 操作）。

```rust
// src/traits/account_repository.rs

#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// 获取所有账户
    async fn find_all(&self) -> CoreResult<Vec<Account>>;

    /// 按 ID 获取账户
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;

    /// 保存账户（插入或更新）
    async fn save(&self, account: &Account) -> CoreResult<()>;

    /// 按 ID 删除账户
    async fn delete(&self, id: &str) -> CoreResult<()>;

    /// 批量保存账户
    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()>;

    /// 更新账户状态（如标记为 Error）
    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>
    ) -> CoreResult<()>;
}
```

**平台实现**：
- `TauriAccountRepository`：使用 `tauri-plugin-store` 进行 JSON 文件存储
- `DatabaseAccountRepository`：使用 SeaORM 进行数据库持久化

#### 2. CredentialStore

抽象安全凭证存储。

```rust
// src/traits/credential_store.rs

/// account_id -> 凭证键值对 的映射
pub type CredentialsMap = HashMap<String, HashMap<String, String>>;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// 加载所有凭证（启动时批量恢复使用）
    async fn load_all(&self) -> CoreResult<CredentialsMap>;

    /// 为账户保存凭证
    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()>;

    /// 加载账户凭证
    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>>;

    /// 删除账户凭证
    async fn delete(&self, account_id: &str) -> CoreResult<()>;

    /// 检查账户凭证是否存在
    async fn exists(&self, account_id: &str) -> CoreResult<bool>;
}
```

**平台实现**：
- `KeychainStore`：使用系统钥匙串（macOS Keychain、Windows 凭据管理器、Linux Secret Service）
- `StrongholdStore`：使用 Tauri Stronghold 插件（Android）
- `DatabaseCredentialStore`：使用 SeaORM + AES 加密（Web 后端）

#### 3. ProviderRegistry

管理运行时 `DnsProvider` 实例。

```rust
// src/traits/provider_registry.rs

#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    /// 为账户注册 Provider 实例
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);

    /// 注销 Provider 实例
    async fn unregister(&self, account_id: &str);

    /// 按账户 ID 获取 Provider 实例
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;

    /// 列出所有已注册的账户 ID
    async fn list_account_ids(&self) -> Vec<String>;
}
```

**默认实现**（`InMemoryProviderRegistry`）：

```rust
pub struct InMemoryProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn DnsProvider>>>,
}

impl InMemoryProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
}
```

这个内存实现被所有平台使用，因为 Provider 实例不需要持久化。

---

## 类型系统

### 账户类型

```rust
// src/types/account.rs

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,                    // UUID
    pub name: String,                  // 用户定义的名称
    pub provider: ProviderType,        // DNS 服务商类型
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Option<AccountStatus>, // Active 或 Error
    pub error: Option<String>,         // 状态为 Error 时的错误信息
}

/// 账户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,  // 账户正常工作
    Error,   // 凭证无效或其他错误
}

/// 创建账户请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider: ProviderType,
    pub credentials: HashMap<String, String>,
}

/// 更新账户请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: String,
    pub name: Option<String>,                          // 可选：新名称
    pub credentials: Option<HashMap<String, String>>,  // 可选：新凭证
}
```

### 域名类型

```rust
// src/types/domain.rs

/// 应用层域名（扩展 ProviderDomain，添加 account_id）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDomain {
    pub id: String,
    pub name: String,
    pub account_id: String,       // 核心层添加
    pub provider: ProviderType,
    pub status: DomainStatus,
    pub record_count: Option<u32>,
}

impl AppDomain {
    /// 从 Provider 层域名转换
    pub fn from_provider(provider_domain: ProviderDomain, account_id: String) -> Self {
        Self {
            id: provider_domain.id,
            name: provider_domain.name,
            account_id,
            provider: provider_domain.provider,
            status: provider_domain.status,
            record_count: provider_domain.record_count,
        }
    }
}
```

### 导入导出类型

```rust
// src/types/export.rs

/// 导出文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub header: ExportFileHeader,
    pub data: serde_json::Value,  // 加密时为 Base64 密文；未加密时为 JSON 数组
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFileHeader {
    pub version: u32,              // 文件格式版本
    pub encrypted: bool,           // 是否加密
    pub salt: Option<String>,      // Base64 盐值（加密时）
    pub nonce: Option<String>,     // Base64 nonce/IV（加密时）
    pub exported_at: String,       // ISO 8601 时间戳
    pub app_version: String,       // 应用版本
}

/// 导出的账户数据（包含凭证）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedAccount {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub credentials: HashMap<String, String>,
}

/// 导入预览结果
#[derive(Debug, Clone, Serialize)]
pub struct ImportPreview {
    pub encrypted: bool,
    pub account_count: usize,
    pub accounts: Option<Vec<ImportPreviewAccount>>,  // 未加密或已解密后可用
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPreviewAccount {
    pub name: String,
    pub provider: ProviderType,
    pub has_conflict: bool,  // 与现有账户名称冲突
}

/// 导入结果
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub failures: Vec<ImportFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportFailure {
    pub name: String,
    pub reason: String,
}
```

### 工具箱类型

```rust
// src/types/toolbox.rs

/// WHOIS 查询结果
#[derive(Debug, Clone, Serialize)]
pub struct WhoisResult {
    pub domain: String,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiration_date: Option<String>,
    pub updated_date: Option<String>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub raw: String,  // 原始 WHOIS 响应
}

/// DNS 查询结果
#[derive(Debug, Clone, Serialize)]
pub struct DnsLookupResult {
    pub nameserver: String,
    pub records: Vec<DnsLookupRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsLookupRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,  // MX/SRV 记录
}

/// IP 地理位置查询结果
#[derive(Debug, Clone, Serialize)]
pub struct IpLookupResult {
    pub query: String,           // 原始输入
    pub is_domain: bool,         // 输入是否为域名
    pub results: Vec<IpGeoInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IpGeoInfo {
    pub ip: String,
    pub ip_version: String,      // "IPv4" 或 "IPv6"
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    pub as_name: Option<String>,
}

/// SSL 证书检查结果
#[derive(Debug, Clone, Serialize)]
pub struct SslCheckResult {
    pub domain: String,
    pub port: u16,
    pub connection_status: String,  // "https" | "http" | "failed"
    pub cert_info: Option<SslCertInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SslCertInfo {
    pub domain: String,
    pub issuer: String,
    pub subject: String,
    pub valid_from: String,
    pub valid_to: String,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub is_valid: bool,
    pub san: Vec<String>,
    pub serial_number: String,
    pub signature_algorithm: String,
    pub certificate_chain: Vec<CertChainItem>,
}
```

---

## 服务层

所有服务遵循相同模式：

```rust
pub struct XxxService {
    ctx: Arc<ServiceContext>,
}

impl XxxService {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    // 服务方法...
}
```

### AccountBootstrapService

在应用启动时恢复所有账户。

```rust
// src/services/account_bootstrap_service.rs

pub struct RestoreResult {
    pub success_count: usize,
    pub failed_accounts: Vec<String>,  // 恢复失败的账户 ID
}

impl AccountBootstrapService {
    /// 从存储恢复所有账户
    ///
    /// 此方法：
    /// 1. 从 AccountRepository 加载所有账户元数据
    /// 2. 从 CredentialStore 加载所有凭证
    /// 3. 重建 DnsProvider 实例
    /// 4. 将 Provider 注册到 ProviderRegistry
    /// 5. 将恢复失败的账户标记为 Error 状态
    pub async fn restore_accounts(&self) -> CoreResult<RestoreResult>;
}
```

**典型用法**（应用启动时）：

```rust
let bootstrap = AccountBootstrapService::new(ctx.clone());
let result = bootstrap.restore_accounts().await?;
println!("恢复了 {} 个账户，{} 个失败", result.success_count, result.failed_accounts.len());
```

### AccountLifecycleService

管理账户 CRUD 操作。

```rust
// src/services/account_lifecycle_service.rs

impl AccountLifecycleService {
    /// 创建新账户
    ///
    /// 步骤：
    /// 1. 验证凭证格式
    /// 2. 创建 Provider 并通过 API 验证凭证
    /// 3. 将凭证保存到 CredentialStore
    /// 4. 将 Provider 注册到 ProviderRegistry
    /// 5. 将账户元数据保存到 AccountRepository
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account>;

    /// 更新现有账户
    ///
    /// 支持更新名称和/或凭证。
    /// 如果更新凭证，会重新验证。
    pub async fn update_account(&self, request: UpdateAccountRequest) -> CoreResult<Account>;

    /// 删除账户
    ///
    /// 操作顺序（防止幽灵账户）：
    /// 1. 从 AccountRepository 删除元数据
    /// 2. 从 ProviderRegistry 注销
    /// 3. 从 CredentialStore 删除凭证
    pub async fn delete_account(&self, account_id: &str) -> CoreResult<()>;

    /// 批量删除账户
    pub async fn batch_delete_accounts(&self, account_ids: &[String]) -> CoreResult<BatchDeleteAccountResult>;
}
```

### AccountMetadataService

查询账户元数据。

```rust
// src/services/account_metadata_service.rs

impl AccountMetadataService {
    /// 获取所有账户
    pub async fn list_accounts(&self) -> CoreResult<Vec<Account>>;

    /// 按 ID 获取账户
    pub async fn get_account(&self, account_id: &str) -> CoreResult<Account>;
}
```

### CredentialManagementService

管理凭证验证和存储。

```rust
// src/services/credential_management_service.rs

impl CredentialManagementService {
    /// 验证凭证并创建 Provider 实例
    ///
    /// 成功时返回创建的 Provider。
    /// 验证失败时返回 CredentialValidationError。
    pub async fn validate_and_create_provider(
        &self,
        provider_type: &ProviderType,
        credentials: &HashMap<String, String>,
    ) -> CoreResult<Arc<dyn DnsProvider>>;

    /// 为账户注册 Provider 实例
    pub async fn register_provider(&self, account_id: &str, provider: Arc<dyn DnsProvider>);

    /// 注销 Provider 实例
    pub async fn unregister_provider(&self, account_id: &str);

    /// 保存凭证到存储
    pub async fn save_credentials(
        &self,
        account_id: &str,
        credentials: &HashMap<String, String>
    ) -> CoreResult<()>;

    /// 从存储删除凭证
    pub async fn delete_credentials(&self, account_id: &str) -> CoreResult<()>;
}
```

### DnsService

管理 DNS 记录。

```rust
// src/services/dns_service.rs

impl DnsService {
    /// 列出域名的 DNS 记录
    pub async fn list_records(
        &self,
        account_id: &str,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> CoreResult<PaginatedResponse<DnsRecord>>;

    /// 创建 DNS 记录
    pub async fn create_record(
        &self,
        account_id: &str,
        request: &CreateDnsRecordRequest,
    ) -> CoreResult<DnsRecord>;

    /// 更新 DNS 记录
    pub async fn update_record(
        &self,
        account_id: &str,
        record_id: &str,
        request: &UpdateDnsRecordRequest,
    ) -> CoreResult<DnsRecord>;

    /// 删除 DNS 记录
    pub async fn delete_record(
        &self,
        account_id: &str,
        record_id: &str,
        domain_id: &str,
    ) -> CoreResult<()>;

    /// 批量删除 DNS 记录
    pub async fn batch_delete_records(
        &self,
        account_id: &str,
        domain_id: &str,
        record_ids: &[String],
    ) -> CoreResult<BatchDeleteResult>;
}
```

**凭证失效检测**：

DnsService 会自动检测凭证失效：

```rust
async fn list_records(...) -> CoreResult<...> {
    let provider = self.ctx.get_provider(account_id).await?;

    match provider.list_records(domain_id, params).await {
        Ok(response) => Ok(response),
        Err(ProviderError::InvalidCredentials { .. }) => {
            // 标记账户为无效
            self.ctx.mark_account_invalid(account_id, "凭证已过期").await;
            Err(CoreError::InvalidCredentials(account_id.to_string()))
        }
        Err(e) => Err(CoreError::Provider(e)),
    }
}
```

### DomainService

管理域名。

```rust
// src/services/domain_service.rs

impl DomainService {
    /// 列出账户的域名
    pub async fn list_domains(
        &self,
        account_id: &str,
        params: &PaginationParams,
    ) -> CoreResult<PaginatedResponse<AppDomain>>;

    /// 获取单个域名
    pub async fn get_domain(
        &self,
        account_id: &str,
        domain_id: &str,
    ) -> CoreResult<AppDomain>;
}
```

### ImportExportService

处理账户导入导出，支持可选加密。

```rust
// src/services/import_export_service.rs

impl ImportExportService {
    /// 导出账户为 JSON（可选加密）
    pub async fn export_accounts(&self, request: &ExportAccountsRequest) -> CoreResult<ExportResponse>;

    /// 预览导入文件（解析但不导入）
    pub async fn preview_import(
        &self,
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<ImportPreview>;

    /// 从文件导入账户
    pub async fn import_accounts(
        &self,
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<ImportResult>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportAccountsRequest {
    pub account_ids: Vec<String>,
    pub encrypt: bool,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResponse {
    pub content: String,      // JSON 字符串（可能包含加密数据）
    pub filename: String,     // 建议的文件名
}
```

### ProviderMetadataService

获取 DNS 服务商元数据。

```rust
// src/services/provider_metadata_service.rs

impl ProviderMetadataService {
    /// 获取所有可用 Provider 的元数据
    pub fn get_all_providers(&self) -> Vec<ProviderMetadata>;

    /// 获取特定 Provider 的元数据
    pub fn get_provider(&self, provider_type: &ProviderType) -> Option<ProviderMetadata>;
}
```

### ToolboxService

无状态工具服务。

```rust
// src/services/toolbox/mod.rs

impl ToolboxService {
    /// WHOIS 查询
    pub async fn whois(&self, domain: &str) -> CoreResult<WhoisResult>;

    /// DNS 查询
    pub async fn dns_lookup(
        &self,
        domain: &str,
        record_type: &str,
        nameserver: Option<&str>,
    ) -> CoreResult<DnsLookupResult>;

    /// IP 地理位置查询
    pub async fn ip_lookup(&self, query: &str) -> CoreResult<IpLookupResult>;

    /// SSL 证书检查
    pub async fn ssl_check(&self, domain: &str, port: Option<u16>) -> CoreResult<SslCheckResult>;
}
```

---

## 错误处理

### CoreError

核心层统一错误类型：

```rust
// src/error.rs

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum CoreError {
    #[error("Provider 未找到: {0}")]
    ProviderNotFound(String),

    #[error("账户未找到: {0}")]
    AccountNotFound(String),

    #[error("域名未找到: {0}")]
    DomainNotFound(String),

    #[error("记录未找到: {0}")]
    RecordNotFound(String),

    #[error("凭证错误: {0}")]
    CredentialError(String),

    #[error("凭证验证失败")]
    CredentialValidation(CredentialValidationError),

    #[error("来自 {provider} 的 API 错误: {message}")]
    ApiError { provider: String, message: String },

    #[error("账户凭证无效: {0}")]
    InvalidCredentials(String),

    #[error("序列化错误: {0}")]
    SerializationError(String),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("导入/导出错误: {0}")]
    ImportExportError(String),

    #[error("未选择任何账户")]
    NoAccountsSelected,

    #[error("不支持的文件版本")]
    UnsupportedFileVersion,

    #[error("存储错误: {0}")]
    StorageError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("Provider 错误: {0}")]
    Provider(#[from] ProviderError),
}

pub type CoreResult<T> = std::result::Result<T, CoreError>;
```

### 错误传播

```
Provider API 错误
       ↓
ProviderError (dns-orchestrator-provider)
       ↓
CoreError::Provider（通过 From trait 自动转换）
       ↓
平台层 (Tauri/Actix-Web)
       ↓
前端（带错误码和详情的 JSON）
```

### 序列化格式

错误序列化为 tagged union 格式，便于前端处理：

```json
{
  "code": "CredentialValidation",
  "details": {
    "type": "missingField",
    "provider": "cloudflare",
    "field": "apiToken",
    "label": "API Token"
  }
}
```

---

## 加密模块

加密模块为账户导入导出提供 AES-256-GCM 加密。

### 算法

- **加密**：AES-256-GCM
- **密钥派生**：PBKDF2-HMAC-SHA256
- **盐**：16 字节，每次加密随机生成
- **Nonce/IV**：12 字节，每次加密随机生成

### 版本管理

```rust
// src/crypto/versions.rs

pub const CURRENT_FILE_VERSION: u32 = 2;

/// 获取特定版本的 PBKDF2 迭代次数
pub fn get_pbkdf2_iterations(version: u32) -> u32 {
    match version {
        1 => 100_000,   // 旧版本
        2 => 600_000,   // OWASP 2023 推荐
        _ => 600_000,   // 默认使用当前版本
    }
}

pub fn get_current_iterations() -> u32 {
    get_pbkdf2_iterations(CURRENT_FILE_VERSION)
}
```

### API

```rust
// src/crypto/mod.rs

/// 使用密码加密明文
/// 返回 (salt_base64, nonce_base64, ciphertext_base64)
pub fn encrypt(plaintext: &[u8], password: &str) -> CoreResult<(String, String, String)>;

/// 使用密码解密密文
pub fn decrypt(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
) -> CoreResult<Vec<u8>>;

/// 使用特定迭代次数解密（用于向后兼容）
pub fn decrypt_with_iterations(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
    iterations: u32,
) -> CoreResult<Vec<u8>>;
```

### 使用示例

```rust
use dns_orchestrator_core::crypto::{encrypt, decrypt};

// 加密
let plaintext = b"敏感数据";
let (salt, nonce, ciphertext) = encrypt(plaintext, "password123")?;

// 解密
let decrypted = decrypt(&ciphertext, "password123", &salt, &nonce)?;
assert_eq!(plaintext, decrypted.as_slice());
```

---

## 数据流图

### 创建账户流程

```
┌─────────────────┐
│  CreateAccount  │
│     请求        │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  CredentialManagementService        │
│  validate_and_create_provider()     │
│                                     │
│  1. ProviderCredentials::from_map() │
│  2. create_provider()               │
│  3. provider.validate_credentials() │
└────────┬────────────────────────────┘
         │ Provider 实例
         ▼
┌─────────────────────────────────────┐
│  CredentialStore.save()             │
│  （保存凭证到安全存储）              │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  ProviderRegistry.register()        │
│  （在内存中注册 Provider）          │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  AccountRepository.save()           │
│  （保存账户元数据）                  │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────┐
│    Account      │
│   （响应）      │
└─────────────────┘
```

### 启动恢复流程

```
┌─────────────────────────────────────┐
│        应用程序启动                  │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  AccountRepository.find_all()       │
│  （加载所有账户元数据）              │
└────────┬────────────────────────────┘
         │ Vec<Account>
         ▼
┌─────────────────────────────────────┐
│  CredentialStore.load_all()         │
│  （批量加载所有凭证）                │
└────────┬────────────────────────────┘
         │ CredentialsMap
         ▼
┌─────────────────────────────────────┐
│  对每个账户：                        │
│  1. create_provider(credentials)    │
│  2. ProviderRegistry.register()     │
│                                     │
│  失败时：                            │
│  - 标记账户为 Error 状态             │
│  - 继续处理下一个账户                │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  RestoreResult                      │
│  { success_count, failed_accounts } │
└─────────────────────────────────────┘
```

### DNS 操作流程

```
┌─────────────────┐
│  list_records   │
│  (account_id,   │
│   domain_id)    │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  ServiceContext.get_provider()      │
│  （从 ProviderRegistry 获取）       │
└────────┬────────────────────────────┘
         │ Arc<dyn DnsProvider>
         ▼
┌─────────────────────────────────────┐
│  provider.list_records()            │
│  （调用 DNS 服务商 API）            │
└────────┬────────────────────────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐  ┌────────────────────────┐
│  成功  │  │ InvalidCredentials     │
│        │  │                        │
│ 返回   │  │ mark_account_invalid() │
│ 记录   │  │ 返回错误                │
└────────┘  └────────────────────────┘
```

---

## 平台适配指南

要在新平台使用 `dns-orchestrator-core`，需要实现三个 Trait。

### 1. 实现 AccountRepository

```rust
use dns_orchestrator_core::{AccountRepository, Account, AccountStatus, CoreResult};

pub struct MyAccountRepository {
    // 你的存储后端（数据库、文件等）
}

#[async_trait]
impl AccountRepository for MyAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        // 从存储加载所有账户
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        // 按 ID 加载账户
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        // 保存账户（插入或更新）
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        // 删除账户
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        // 批量保存
    }

    async fn update_status(&self, id: &str, status: AccountStatus, error: Option<String>) -> CoreResult<()> {
        // 更新账户状态
    }
}
```

### 2. 实现 CredentialStore

```rust
use dns_orchestrator_core::{CredentialStore, CredentialsMap, CoreResult};

pub struct MyCredentialStore {
    // 你的安全存储（钥匙串、数据库等）
}

#[async_trait]
impl CredentialStore for MyCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        // 批量加载所有凭证
    }

    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()> {
        // 安全保存凭证
    }

    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>> {
        // 加载凭证
    }

    async fn delete(&self, account_id: &str) -> CoreResult<()> {
        // 删除凭证
    }

    async fn exists(&self, account_id: &str) -> CoreResult<bool> {
        // 检查凭证是否存在
    }
}
```

### 3. 组装所有组件

```rust
use dns_orchestrator_core::{
    ServiceContext,
    InMemoryProviderRegistry,
    AccountBootstrapService,
    AccountLifecycleService,
    DnsService,
    // ... 其他服务
};

// 使用你的实现创建 ServiceContext
let ctx = Arc::new(ServiceContext::new(
    Arc::new(MyCredentialStore::new()),
    Arc::new(MyAccountRepository::new()),
    Arc::new(InMemoryProviderRegistry::new()),  // 使用内置实现
));

// 启动时恢复账户
let bootstrap = AccountBootstrapService::new(ctx.clone());
let result = bootstrap.restore_accounts().await?;

// 使用服务
let lifecycle = AccountLifecycleService::new(ctx.clone());
let dns = DnsService::new(ctx.clone());

// 创建账户
let account = lifecycle.create_account(CreateAccountRequest {
    name: "我的 DNS".to_string(),
    provider: ProviderType::Cloudflare,
    credentials: [("apiToken".to_string(), "token".to_string())].into(),
}).await?;

// 列出记录
let records = dns.list_records(&account.id, "domain_id", &params).await?;
```

### 示例：Tauri 实现

```rust
// Tauri 后端使用：
// - KeychainStore 作为 CredentialStore（系统钥匙串）
// - TauriAccountRepository 作为 AccountRepository（tauri-plugin-store）
// - InMemoryProviderRegistry 作为 ProviderRegistry

let credential_store = Arc::new(KeychainStore::new(app_handle.clone()));
let account_repository = Arc::new(TauriAccountRepository::new(app_handle.clone()));
let provider_registry = Arc::new(InMemoryProviderRegistry::new());

let ctx = Arc::new(ServiceContext::new(
    credential_store,
    account_repository,
    provider_registry,
));
```

### 示例：Actix-Web 实现

```rust
// Actix-Web 后端使用：
// - DatabaseCredentialStore（SeaORM + AES 加密）
// - DatabaseAccountRepository（SeaORM）
// - InMemoryProviderRegistry

let credential_store = Arc::new(DatabaseCredentialStore::new(db_pool.clone(), encryption_key));
let account_repository = Arc::new(DatabaseAccountRepository::new(db_pool.clone()));
let provider_registry = Arc::new(InMemoryProviderRegistry::new());

let ctx = Arc::new(ServiceContext::new(
    credential_store,
    account_repository,
    provider_registry,
));
```

---

## 从 Provider 库重新导出的类型

为方便使用，核心库重新导出了 `dns-orchestrator-provider` 中常用的类型：

```rust
// 来自 dns-orchestrator-provider
pub use dns_orchestrator_provider::{
    // Provider trait
    DnsProvider,

    // 类型
    DnsRecord,
    DnsRecordType,
    RecordData,
    ProviderDomain,
    DomainStatus,
    ProviderType,
    ProviderMetadata,
    ProviderCredentials,
    ProviderCredentialField,
    FieldType,
    ProviderFeatures,
    ProviderLimits,

    // 请求/响应
    CreateDnsRecordRequest,
    UpdateDnsRecordRequest,
    PaginationParams,
    RecordQueryParams,
    PaginatedResponse,

    // 批量操作
    BatchCreateResult,
    BatchUpdateResult,
    BatchDeleteResult,
    BatchCreateFailure,
    BatchUpdateFailure,
    BatchDeleteFailure,
    BatchUpdateItem,

    // 错误
    ProviderError,
    CredentialValidationError,

    // 工厂
    create_provider,
    get_all_provider_metadata,
};
```

---

## Feature Flags

```toml
# Cargo.toml

[features]
default = ["native-tls"]
native-tls = ["dns-orchestrator-provider/native-tls"]  # 桌面平台
rustls = ["dns-orchestrator-provider/rustls"]          # Android（避免 OpenSSL）
```

根据目标平台选择适当的 TLS 后端：

- **桌面（macOS、Windows、Linux）**：使用 `native-tls`（默认）
- **Android**：使用 `rustls` 避免 OpenSSL 交叉编译问题

```toml
# Android 构建
dns-orchestrator-core = { path = "../dns-orchestrator-core", default-features = false, features = ["rustls"] }
```
