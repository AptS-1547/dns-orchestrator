# 架构文档

本文档深入介绍 DNS Orchestrator 的架构设计，解释关键组件、设计模式和技术决策。

## 目录

- [概述](#概述)
- [架构图](#架构图)
- [项目结构](#项目结构)
- [前端架构](#前端架构)
- [核心库](#核心库)
- [后端架构](#后端架构)
- [Provider 库](#provider-库)
- [安全架构](#安全架构)
- [性能优化](#性能优化)
- [数据流](#数据流)
- [设计决策](#设计决策)

## 概述

DNS Orchestrator 是一个采用**四层架构**的跨平台应用：

```
前端 → 后端 → 核心库 → Provider 库 → DNS APIs
```

- **前端**：基于 React 的 UI，使用 TypeScript、Tailwind CSS 和 Zustand 状态管理
- **后端**：基于 Rust 的 Tauri 命令（桌面/移动端），以及用于 Web 的 actix-web 后端
- **核心库**：平台无关的业务逻辑（`dns-orchestrator-core` crate）
- **Provider 库**：独立的 `dns-orchestrator-provider` crate，用于 DNS 提供商集成
- **通信**：Transport 抽象层支持 Tauri IPC 和 HTTP 两种方式

### 技术选型

| 组件 | 技术 | 选择理由 |
|------|------|----------|
| **UI 框架** | React 19 + TypeScript 5 | 强大的生态系统、类型安全、组件可复用 |
| **状态管理** | Zustand 5 | 轻量级、无样板代码、简洁的 API |
| **样式方案** | Tailwind CSS 4 | 实用优先、快速开发、设计一致性 |
| **桌面框架** | Tauri 2 | 比 Electron 体积更小、Rust 的安全优势 |
| **核心库** | 独立 Rust crate | 平台无关的业务逻辑、基于 trait 的依赖注入 |
| **Web 后端** | actix-web | 高性能、异步、生产就绪 |
| **Provider 库** | 独立 Rust crate | 可在 Tauri 和 Web 后端复用 |
| **HTTP 客户端** | reqwest | 行业标准、异步、支持 TLS |
| **凭证存储** | keyring / Stronghold | 跨平台系统钥匙串集成 |
| **构建工具** | Vite 7 | 快速 HMR、优化的生产构建 |

## 架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                           用户界面                                   │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  React 组件 (src/components/)                                 │  │
│  │  - AccountList, DnsRecordTable, DomainList, Toolbox           │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Zustand 状态管理 (src/stores/)                               │  │
│  │  - accountStore, dnsStore, domainStore, toolboxStore          │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  服务层 (src/services/)                                       │  │
│  │  - accountService, dnsService, domainService, toolboxService  │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Transport 抽象层 (src/services/transport/)                   │  │
│  │  - ITransport 接口                                            │  │
│  │  - TauriTransport (Tauri IPC) | HttpTransport (REST API)      │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
└─────────────────────────┼────────────────────────────────────────────┘
                          │
        ┌─────────────────┴─────────────────┐
        │                                   │
        ▼ Tauri IPC                         ▼ HTTP REST
┌───────────────────────────┐    ┌───────────────────────────┐
│   TAURI 后端              │    │   ACTIX-WEB 后端          │
│   (src-tauri/)            │    │   (src-actix-web/)        │
│                           │    │                           │
│  ┌─────────────────────┐  │    │  ┌─────────────────────┐  │
│  │  命令层             │  │    │  │  HTTP 处理器        │  │
│  │  - account.rs       │  │    │  │  (REST 端点)        │  │
│  │  - dns.rs           │  │    │  │                     │  │
│  │  - domain.rs        │  │    │  └──────────┬──────────┘  │
│  │  - toolbox.rs       │  │    │             │             │
│  └──────────┬──────────┘  │    │  ┌──────────▼──────────┐  │
│             │             │    │  │  SeaORM 数据库      │  │
│  ┌──────────▼──────────┐  │    │  │  (MySQL/PG/SQLite)  │  │
│  │  AppState           │  │    │  └─────────────────────┘  │
│  │  (9 个服务)         │  │    │                           │
│  └──────────┬──────────┘  │    └───────────┬───────────────┘
│             │             │                │
└─────────────┼─────────────┘                │
              │                              │
              └──────────────┬───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  核心库                       │
              │  (dns-orchestrator-core)     │
              │                              │
              │  ┌────────────────────────┐  │
              │  │  ServiceContext        │  │
              │  │  + CredentialStore     │  │
              │  │  + AccountRepository   │  │
              │  │  + ProviderRegistry    │  │
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  业务服务              │  │
              │  │  - AccountLifecycle    │  │
              │  │  - CredentialManagement│  │
              │  │  - DnsService          │  │
              │  │  - DomainService       │  │
              │  │  - ImportExport        │  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  Provider 库                 │
              │  (dns-orchestrator-provider) │
              │                              │
              │  ┌────────────────────────┐  │
              │  │  DnsProvider Trait     │  │
              │  │  - id(), metadata()    │  │
              │  │  - list_domains()      │  │
              │  │  - list_records()      │  │
              │  │  - create/update/del   │  │
              │  │  - batch_* (TODO)      │  │
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  Provider 实现         │  │
              │  │  - CloudflareProvider  │  │
              │  │  - AliyunProvider      │  │
              │  │  - DnspodProvider      │  │
              │  │  - HuaweicloudProvider │  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │ HTTPS
              ┌──────────────▼───────────────┐
              │       外部 DNS API            │
              │  Cloudflare | 阿里云 | DNSPod │
              │  华为云                       │
              └───────────────────────────────┘
```

## 项目结构

```
dns-orchestrator/
├── src/                              # 前端 (React + TypeScript)
│   ├── components/                   # 按功能组织的 React 组件
│   │   ├── ui/                       # Radix UI 封装 (shadcn/ui)
│   │   ├── account/                  # 账户管理
│   │   ├── accounts/                 # 账户页面
│   │   ├── dns/                      # DNS 记录管理
│   │   ├── domain/                   # 域名组件
│   │   ├── domains/                  # 域名选择器页面
│   │   ├── toolbox/                  # 网络工具
│   │   ├── settings/                 # 设置页面
│   │   ├── layout/                   # 布局组件
│   │   ├── navigation/               # 导航组件
│   │   ├── titlebar/                 # 窗口标题栏
│   │   └── error/                    # 错误边界
│   ├── services/                     # 服务层
│   │   ├── transport/                # Transport 抽象
│   │   │   ├── types.ts              # ITransport 接口, CommandMap
│   │   │   ├── tauri.transport.ts    # Tauri IPC 实现
│   │   │   └── http.transport.ts     # HTTP REST 实现
│   │   ├── account.service.ts
│   │   ├── dns.service.ts
│   │   ├── domain.service.ts
│   │   ├── toolbox.service.ts
│   │   └── file.service.ts
│   ├── stores/                       # Zustand 状态管理
│   │   ├── accountStore.ts           # 账户状态 + providers
│   │   ├── dnsStore.ts               # DNS 记录 + 分页
│   │   ├── domainStore.ts            # 按账户的域名
│   │   ├── settingsStore.ts          # 主题、语言、调试
│   │   ├── toolboxStore.ts           # 工具箱历史
│   │   └── updaterStore.ts           # 自动更新状态
│   ├── types/                        # TypeScript 类型定义
│   ├── i18n/                         # 国际化 (en, zh-CN)
│   ├── hooks/                        # 自定义 React hooks
│   ├── lib/                          # 工具函数
│   └── constants/                    # 常量
│
├── dns-orchestrator-core/            # 核心业务逻辑库
│   ├── src/
│   │   ├── lib.rs                    # 库入口, 重导出
│   │   ├── error.rs                  # CoreError, CoreResult
│   │   ├── services/                 # 业务服务
│   │   │   ├── mod.rs                # ServiceContext
│   │   │   ├── account_metadata_service.rs
│   │   │   ├── credential_management_service.rs
│   │   │   ├── account_lifecycle_service.rs
│   │   │   ├── account_bootstrap_service.rs
│   │   │   ├── provider_metadata_service.rs
│   │   │   ├── import_export_service.rs
│   │   │   ├── domain_service.rs
│   │   │   ├── dns_service.rs
│   │   │   └── toolbox/              # 工具箱服务
│   │   ├── traits/                   # 平台抽象 trait
│   │   │   ├── credential_store.rs   # CredentialStore trait
│   │   │   ├── account_repository.rs # AccountRepository trait
│   │   │   └── provider_registry.rs  # ProviderRegistry trait
│   │   ├── types/                    # 内部类型
│   │   ├── crypto/                   # AES-GCM 加密
│   │   └── utils/                    # 工具函数
│   └── Cargo.toml
│
├── dns-orchestrator-provider/        # DNS Provider 库
│   ├── src/
│   │   ├── lib.rs                    # 库入口, 重导出
│   │   ├── traits.rs                 # DnsProvider trait
│   │   ├── types.rs                  # RecordData, ProviderCredentials 等
│   │   ├── error.rs                  # ProviderError 枚举 (13 种变体)
│   │   ├── factory.rs                # create_provider(), metadata
│   │   ├── http_client.rs            # HTTP 客户端封装
│   │   └── providers/                # Provider 实现
│   │       ├── cloudflare/           # Cloudflare (mod, provider, http, types, error)
│   │       ├── aliyun/               # 阿里云 DNS
│   │       ├── dnspod/               # 腾讯云 DNSPod
│   │       └── huaweicloud/          # 华为云 DNS
│   ├── tests/                        # 集成测试
│   └── Cargo.toml                    # Feature flags
│
├── src-tauri/                        # Tauri 后端 (桌面/移动端)
│   ├── src/
│   │   ├── lib.rs                    # AppState, run()
│   │   ├── commands/                 # Tauri 命令处理器
│   │   │   ├── account.rs            # 10 个命令
│   │   │   ├── domain.rs             # 2 个命令
│   │   │   ├── dns.rs                # 5 个命令
│   │   │   ├── toolbox.rs            # 4 个命令
│   │   │   └── updater.rs            # 仅 Android (3 个命令)
│   │   ├── adapters/                 # 核心 trait 实现
│   │   │   ├── credential_store.rs   # TauriCredentialStore
│   │   │   └── account_repository.rs # TauriAccountRepository
│   │   ├── types.rs                  # 前端类型
│   │   └── error.rs                  # 错误转换
│   ├── capabilities/                 # Tauri 2 权限配置
│   └── Cargo.toml                    # 平台特定依赖
│
├── src-actix-web/                    # Web 后端 (开发中)
│   ├── src/main.rs                   # Actix-web 服务器入口
│   └── migration/                    # SeaORM 数据库迁移
│
└── vite.config.ts                    # 平台感知的构建配置
```

## 前端架构

### 服务层

服务层抽象了后端通信：

```typescript
// src/services/transport/types.ts
export interface ITransport {
  invoke<K extends NoArgsCommands>(command: K): Promise<CommandMap[K]["result"]>
  invoke<K extends WithArgsCommands>(
    command: K,
    args: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]>
}

// CommandMap 提供类型安全的命令定义
export interface CommandMap {
  list_accounts: { args: Record<string, never>; result: ApiResponse<Account[]> }
  create_account: { args: { request: CreateAccountRequest }; result: ApiResponse<Account> }
  // ... 全部 24 个命令，完全类型安全
}
```

**Transport 实现**：

```typescript
// src/services/transport/tauri.transport.ts (桌面/移动端)
export class TauriTransport implements ITransport {
  async invoke(command, args?) {
    return await tauriInvoke(command, args)
  }
}

// src/services/transport/http.transport.ts (Web)
export class HttpTransport implements ITransport {
  async invoke(command, args?) {
    return await fetch(`/api/${command}`, { method: 'POST', body: JSON.stringify(args) })
  }
}
```

**编译时 Transport 选择**：

```typescript
// vite.config.ts
resolve: {
  alias: {
    "#transport-impl": platform === "web"
      ? "./src/services/transport/http.transport.ts"
      : "./src/services/transport/tauri.transport.ts",
  },
}
```

### 组件结构

组件按功能域组织：

```
src/components/
├── account/              # 账户表单、对话框
├── accounts/             # 账户页面、批量操作
├── dns/                  # DNS 记录表格、表单、行
├── domain/               # 域名列表、选择器
├── domains/              # 域名页面
├── home/                 # 首页仪表盘
├── toolbox/              # WHOIS、DNS、IP、SSL 查询
├── settings/             # 设置页面
├── layout/               # RootLayout, Sidebar
├── navigation/           # 面包屑、标签页
├── titlebar/             # 窗口控件
├── error/                # 错误边界
└── ui/                   # Radix UI 封装 (shadcn/ui)
```

### 状态管理 (Zustand)

每个功能域有独立的 store，支持细粒度选择器：

```typescript
// src/stores/dnsStore.ts
interface DnsStore {
  // 状态
  records: DnsRecord[]
  currentPage: number
  pageSize: number
  totalCount: number
  hasMore: boolean
  searchQuery: string
  filterType: RecordType | 'ALL'
  selectedIds: Set<string>

  // 操作
  fetchRecords: (accountId: string, domainId: string) => Promise<void>
  createRecord: (request: CreateDnsRecordRequest) => Promise<void>
  updateRecord: (recordId: string, request: UpdateDnsRecordRequest) => Promise<void>
  deleteRecord: (recordId: string, domainId: string) => Promise<void>
  batchDelete: (recordIds: string[], domainId: string) => Promise<BatchDeleteResult>

  // 选择
  toggleSelection: (recordId: string) => void
  selectAll: () => void
  clearSelection: () => void
}

// 使用 useShallow 优化重渲染
const { records, hasMore } = useDnsStore(useShallow(state => ({
  records: state.records,
  hasMore: state.hasMore,
})))
```

## 核心库

`dns-orchestrator-core` crate 通过基于 trait 的依赖注入提供**平台无关的业务逻辑**。

### ServiceContext

核心依赖容器：

```rust
// dns-orchestrator-core/src/services/mod.rs
pub struct ServiceContext {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
    provider_registry: Arc<dyn ProviderRegistry>,
}

impl ServiceContext {
    /// 获取账户的 provider 实例
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// 标记账户为无效（凭证错误）
    pub async fn mark_account_invalid(&self, account_id: &str, error: &str) -> CoreResult<()> {
        self.account_repository.update_status(account_id, AccountStatus::Error, Some(error)).await
    }
}
```

### Trait 抽象

平台特定实现通过 trait 注入：

```rust
// dns-orchestrator-core/src/traits/credential_store.rs

/// account_id -> 凭证键值对 的映射
pub type CredentialsMap = HashMap<String, HashMap<String, String>>;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn load_all(&self) -> CoreResult<CredentialsMap>;
    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()>;
    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>>;
    async fn delete(&self, account_id: &str) -> CoreResult<()>;
    async fn exists(&self, account_id: &str) -> CoreResult<bool>;
}

// dns-orchestrator-core/src/traits/account_repository.rs
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_all(&self) -> CoreResult<Vec<Account>>;
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;
    async fn save(&self, account: Account) -> CoreResult<()>;
    async fn delete(&self, id: &str) -> CoreResult<()>;
    async fn update_status(&self, id: &str, status: AccountStatus, error: Option<&str>) -> CoreResult<()>;
}

// dns-orchestrator-core/src/traits/provider_registry.rs
#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;
    async fn remove(&self, account_id: &str);
}
```

### 细粒度服务

业务逻辑拆分为专注的服务：

| 服务 | 职责 |
|------|------|
| `AccountMetadataService` | 账户 CRUD（仅元数据，不含凭证） |
| `CredentialManagementService` | 验证、存储、删除凭证 |
| `AccountLifecycleService` | 完整账户生命周期（组合元数据 + 凭证） |
| `AccountBootstrapService` | 应用启动时恢复账户 |
| `ProviderMetadataService` | 查询 provider 元数据（无状态） |
| `ImportExportService` | 加密账户备份/恢复 |
| `DomainService` | 列出域名、获取域名详情 |
| `DnsService` | DNS 记录 CRUD、批量删除 |
| `ToolboxService` | WHOIS、DNS、IP、SSL 查询 |

```rust
// 示例：AccountLifecycleService 组合
pub struct AccountLifecycleService {
    metadata_service: Arc<AccountMetadataService>,
    credential_service: Arc<CredentialManagementService>,
}

impl AccountLifecycleService {
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
        // 1. 使用提供商的 API 验证凭证。
        // 2. 使用 CredentialStore 安全地保存凭证。
        // 3. 在 ProviderRegistry 中注册新的提供商实例。
        // 4. 使用 AccountRepository 保存账户元数据。
        // ...
    }
}
```

## 后端架构

### Tauri 应用状态

```rust
// src-tauri/src/lib.rs
pub struct AppState {
    /// 服务上下文（依赖注入容器）
    pub ctx: Arc<ServiceContext>,

    /// 细粒度服务
    pub account_metadata_service: Arc<AccountMetadataService>,
    pub credential_management_service: Arc<CredentialManagementService>,
    pub account_lifecycle_service: Arc<AccountLifecycleService>,
    pub account_bootstrap_service: Arc<AccountBootstrapService>,
    pub provider_metadata_service: ProviderMetadataService,
    pub import_export_service: ImportExportService,
    pub domain_service: DomainService,
    pub dns_service: DnsService,

    /// 账户恢复标志
    pub restore_completed: AtomicBool,
}
```

### 适配器实现

Tauri 后端实现核心 trait：

| Trait | 适配器 | 后端 |
|-------|--------|------|
| `CredentialStore` | `TauriCredentialStore` | keyring (桌面) / Stronghold (Android) |
| `AccountRepository` | `TauriAccountRepository` | tauri-plugin-store (JSON 文件) |
| `ProviderRegistry` | `InMemoryProviderRegistry` | 内存中的 HashMap |

```rust
// src-tauri/src/adapters/credential_store.rs
pub struct TauriCredentialStore {
    cache: Arc<RwLock<HashMap<String, ProviderCredentials>>>,
    #[cfg(target_os = "android")]
    app_handle: AppHandle,
}

#[async_trait]
impl CredentialStore for TauriCredentialStore {
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        // 先检查缓存
        if let Some(cred) = self.cache.read().await.get(account_id) {
            return Ok(Some(cred.clone()));
        }

        // 从系统钥匙串加载
        #[cfg(not(target_os = "android"))]
        {
            let entry = Entry::new("dns-orchestrator", account_id)?;
            // ... 加载并反序列化
        }

        #[cfg(target_os = "android")]
        {
            // 使用 Stronghold
        }
    }
}
```

### 平台特定依赖

```toml
# src-tauri/Cargo.toml

# 桌面端 (macOS, Windows, Linux)
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
dns-orchestrator-provider = { path = "../dns-orchestrator-provider", features = ["all-providers", "native-tls"] }
dns-orchestrator-core = { path = "../dns-orchestrator-core" }
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }

# Android
[target."cfg(target_os = \"android\")".dependencies]
dns-orchestrator-provider = { path = "../dns-orchestrator-provider", default-features = false, features = ["all-providers", "rustls"] }
dns-orchestrator-core = { path = "../dns-orchestrator-core", default-features = false, features = ["rustls"] }
tauri-plugin-stronghold = "2"
```

## Provider 库

### 设计目标

1. **可复用性**：相同的 provider 代码可在 Tauri 和 actix-web 后端使用
2. **Feature Flags**：选择性启用 provider 和 TLS 后端
3. **类型安全**：`RecordData` 枚举提供结构化的 DNS 记录数据
4. **统一错误处理**：`ProviderError` 映射所有 provider 特定错误

### DnsProvider Trait

```rust
// dns-orchestrator-provider/src/traits.rs
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Provider 标识符（如 "cloudflare"）
    fn id(&self) -> &'static str;

    /// Provider 元数据（类型级别，无需实例）
    fn metadata() -> ProviderMetadata where Self: Sized;

    /// 验证凭证
    async fn validate_credentials(&self) -> Result<bool>;

    /// 列出域名（分页）
    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<ProviderDomain>>;

    /// 获取域名详情
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain>;

    /// 列出 DNS 记录（分页 + 搜索 + 过滤）
    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams) -> Result<PaginatedResponse<DnsRecord>>;

    /// 创建 DNS 记录
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;

    /// 更新 DNS 记录
    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest) -> Result<DnsRecord>;

    /// 删除 DNS 记录
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;

    // 批量操作（TODO - 查看 trait 文档中的实现计划）
    async fn batch_create_records(&self, requests: &[CreateDnsRecordRequest]) -> Result<BatchCreateResult>;
    async fn batch_update_records(&self, updates: &[BatchUpdateItem]) -> Result<BatchUpdateResult>;
    async fn batch_delete_records(&self, domain_id: &str, record_ids: &[String]) -> Result<BatchDeleteResult>;
}
```

### 类型安全的记录数据 (v1.5.0)

```rust
// dns-orchestrator-provider/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RecordData {
    /// A 记录：IPv4 地址
    A { address: String },

    /// AAAA 记录：IPv6 地址
    AAAA { address: String },

    /// CNAME 记录：别名
    CNAME { target: String },

    /// MX 记录：邮件交换
    MX { priority: u16, exchange: String },

    /// TXT 记录：文本
    TXT { text: String },

    /// NS 记录：域名服务器
    NS { nameserver: String },

    /// SRV 记录：服务定位
    SRV { priority: u16, weight: u16, port: u16, target: String },

    /// CAA 记录：证书颁发机构授权
    CAA { flags: u8, tag: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub domain_id: String,
    pub name: String,
    pub ttl: u32,
    pub data: RecordData,           // 类型安全的记录数据
    pub proxied: Option<bool>,      // Cloudflare 专用
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

### Feature Flags

```toml
# dns-orchestrator-provider/Cargo.toml
[features]
default = ["native-tls", "all-providers"]

# TLS 后端（二选一）
native-tls = ["reqwest/native-tls"]     # 桌面端默认
rustls = ["reqwest/rustls-tls"]          # Android（避免 OpenSSL 交叉编译）

# Provider（单独启用或全部启用）
cloudflare = []
aliyun = []
dnspod = []
huaweicloud = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud"]
```

### 错误处理

```rust
// dns-orchestrator-provider/src/error.rs
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    /// 网络请求失败
    NetworkError { provider: String, detail: String },

    /// 凭证无效
    InvalidCredentials { provider: String, raw_message: Option<String> },

    /// 记录已存在
    RecordExists { provider: String, record_name: String, raw_message: Option<String> },

    /// 记录不存在
    RecordNotFound { provider: String, record_id: String, raw_message: Option<String> },

    /// 参数无效（TTL、值等）
    InvalidParameter { provider: String, param: String, detail: String },

    /// 不支持的记录类型
    UnsupportedRecordType { provider: String, record_type: String },

    /// 配额超限
    QuotaExceeded { provider: String, raw_message: Option<String> },

    /// 域名不存在
    DomainNotFound { provider: String, domain: String, raw_message: Option<String> },

    /// 域名被锁定/禁用
    DomainLocked { provider: String, domain: String, raw_message: Option<String> },

    /// 权限被拒绝
    PermissionDenied { provider: String, raw_message: Option<String> },

    /// 响应解析错误
    ParseError { provider: String, detail: String },

    /// 序列化错误
    SerializationError { provider: String, detail: String },

    /// 未知错误（兜底）
    Unknown { provider: String, raw_code: Option<String>, raw_message: String },
}
```

**错误映射**：

每个 provider 实现 `ProviderErrorMapper` 来映射原始 API 错误：

```rust
// 内部错误映射 trait
pub(crate) trait ProviderErrorMapper {
    fn provider_name(&self) -> &'static str;
    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError;
}
```

### Provider 结构

每个 provider 组织在子目录中：

```
providers/cloudflare/
├── mod.rs          # Provider 结构体, 重导出
├── provider.rs     # DnsProvider trait 实现
├── http.rs         # HTTP 客户端封装
├── types.rs        # Cloudflare 特定类型
└── error.rs        # ProviderErrorMapper 实现
```

## 安全架构

### 各平台凭证存储

| 平台 | 存储机制 |
|------|----------|
| **macOS** | Keychain（通过 `keyring` crate） |
| **Windows** | Credential Manager（通过 `keyring` crate） |
| **Linux** | Secret Service（GNOME Keyring/KWallet，通过 `keyring` crate） |
| **Android** | Stronghold（通过 `tauri-plugin-stronghold`） |

### 凭证流程

```
创建账户
     │
     ▼
┌─────────────────────────────────────┐
│ CredentialManagementService         │
│                                     │
│ 1. 创建 provider 实例               │
│ 2. 验证凭证（API 调用）             │
│ 3. 存储到 CredentialStore           │
│ 4. 注册到 ProviderRegistry          │
└─────────────────────────────────────┘
```

### 账户导入导出加密

```rust
// AES-GCM 加密 + PBKDF2 密钥派生
pub fn encrypt_data(data: &str, password: &str) -> Result<String>
pub fn decrypt_data(encrypted: &str, password: &str) -> Result<String>
```

## 性能优化

1. **分页**：服务端分页，可配置页大小
2. **搜索防抖**：搜索输入 300ms 防抖
3. **无限滚动**：基于 IntersectionObserver 的加载
4. **内存缓存**：凭证和账户缓存在内存中
5. **后台恢复**：账户恢复异步运行，不阻塞启动
6. **Rust 异步**：Tokio 异步运行时实现非阻塞 I/O
7. **Feature Flags**：只编译启用的 provider
8. **useShallow**：细粒度 Zustand 订阅

## 数据流

### DNS 记录查询流程

```
1. 用户在搜索框输入（300ms 防抖）
2. 调用 dnsStore.fetchRecords()
3. 调用 dnsService.listRecords()
4. Transport.invoke('list_dns_records', args)
5. 路由到后端：
   ├─ Tauri：IPC 到 Rust 命令
   └─ Web：HTTP POST 到 actix-web
6. 命令处理器调用 DnsService
7. DnsService 从 ServiceContext 获取 provider
8. Provider 向 DNS API 发起 HTTPS 请求
9. 响应沿着各层返回
10. Store 更新，UI 重新渲染
```

### 账户创建流程

```
前端                             后端                             核心库
   │                                  │                               │
   │ createAccount(request)           │                               │
   ├─────────────────────────────────►│                               │
   │                                  │ AccountLifecycleService       │
   │                                  │      .create_account()        │
   │                                  ├──────────────────────────────►│
   │                                  │                               │
   │                                  │ CredentialManagementService   │
   │                                  │   .validate_and_register()    │
   │                                  │         │                     │
   │                                  │         │ create_provider()   │
   │                                  │         │ provider.validate() │
   │                                  │         │ CredentialStore.set │
   │                                  │         │ ProviderRegistry    │
   │                                  │         │    .register()      │
   │                                  │         ▼                     │
   │                                  │ AccountMetadataService        │
   │                                  │   .create()                   │
   │                                  │         │                     │
   │                                  │         │ AccountRepository   │
   │                                  │         │   .save()           │
   │                                  │         ▼                     │
   │◄─────────────────────────────────┤◄──────────────────────────────┤
   │        Account                   │                               │
```

## 设计决策

### 为什么分离核心库？

| 优势 | 描述 |
|------|------|
| **平台无关** | 相同的业务逻辑用于 Tauri、actix-web、CLI |
| **可测试性** | 通过 mock trait 进行单元测试 |
| **单一职责** | 后端适配器只处理平台 API |
| **类型安全** | 所有后端共享类型 |

### 为什么分离 Provider 库？

| 优势 | 描述 |
|------|------|
| **可复用性** | 相同代码用于 Tauri 和 actix-web 后端 |
| **可测试性** | 独立测试各个 provider |
| **Feature Flags** | 只编译需要的 provider |
| **TLS 灵活性** | 按平台切换 native-tls 和 rustls |

### 为什么使用 Transport 抽象？

| 优势 | 描述 |
|------|------|
| **多平台** | 桌面、移动端、Web 使用相同的前端代码 |
| **类型安全** | CommandMap 确保正确的参数/返回类型 |
| **可测试性** | 前端测试时 mock transport |

### 为什么使用细粒度服务？

| 优势 | 描述 |
|------|------|
| **单一职责** | 每个服务有一个明确的目的 |
| **可组合性** | 服务可以组合（如 AccountLifecycle） |
| **可测试性** | 独立测试每个服务 |
| **灵活性** | 替换或扩展单个服务 |

### 为什么 Web 后端选择 actix-web？

| 标准 | actix-web | axum |
|------|-----------|------|
| **性能** | 最快的 Rust Web 框架 | 非常快 |
| **成熟度** | 生产环境久经考验 | 较新 |
| **生态系统** | 大型插件生态 | 正在成长 |

---

这个架构在简洁性、安全性和性能之间取得平衡，同时通过共享代码支持多平台。
