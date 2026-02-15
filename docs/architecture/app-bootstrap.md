# App 引导层设计

`dns-orchestrator-app` 是多前端共享的引导层 crate，负责将 Core 层的零件组装成可用的服务栈，并提供统一的启动流程。

## 目录

- [设计动机](#设计动机)
- [核心组件](#核心组件)
- [AppStateBuilder](#appstatebuilder)
- [StartupHooks](#startuphooks)
- [启动流程](#启动流程)
- [各前端接入方式](#各前端接入方式)
- [Crate 依赖关系](#crate-依赖关系)

## 设计动机

Core 层通过 trait 抽象了所有存储操作，但"谁来把这些零件拼起来"没有共享实现。在引入 App 层之前，`AppState` 的构建和启动序列（迁移 → 账户恢复）硬编码在 Tauri 的 `setup()` hook 中，每个新前端都需要重复这套逻辑。

App 层解决的问题：

| 问题 | 解决方式 |
|------|---------|
| 服务组装逻辑重复 | `AppStateBuilder` 统一构建 |
| 启动序列重复 | `run_migration()` + `run_account_restore()` |
| 平台备份逻辑耦合 | `StartupHooks` trait 回调 |

## 核心组件

### AppState

平台无关的应用状态容器，持有所有 Service 实例：

```rust
pub struct AppState {
    pub ctx: Arc<ServiceContext>,
    pub account_service: Arc<AccountService>,
    pub provider_metadata_service: ProviderMetadataService,
    pub import_export_service: ImportExportService,
    pub domain_service: DomainService,
    pub domain_metadata_service: Arc<DomainMetadataService>,
    pub dns_service: DnsService,
    pub restore_completed: AtomicBool,
}
```

`AppState` 不包含任何平台特定代码。前端通过它访问所有业务功能。

### AppStateBuilder

Builder 模式注入平台适配器：

```rust
let state = AppStateBuilder::new()
    .credential_store(...)              // 必填
    .account_repository(...)            // 必填
    .domain_metadata_repository(...)    // 必填
    .provider_registry(...)             // 可选，默认 InMemoryProviderRegistry
    .build()?;
```

三个必填适配器对应 Core 层的三个存储 trait：

| 适配器 | Core Trait | 职责 |
|--------|-----------|------|
| `credential_store` | `CredentialStore` | 凭证的安全存储 |
| `account_repository` | `AccountRepository` | 账户元数据持久化 |
| `domain_metadata_repository` | `DomainMetadataRepository` | 域名元数据（收藏、标签） |

`provider_registry` 默认使用 Core 层提供的 `InMemoryProviderRegistry`，大多数前端不需要自定义。

Builder 内部完成：
1. 创建 `ServiceContext`（注入所有适配器）
2. 实例化所有 Service（AccountService、DnsService、DomainService 等）
3. 返回完整的 `AppState`

### StartupHooks

平台特定的启动回调 trait：

```rust
#[async_trait]
pub trait StartupHooks: Send + Sync {
    /// 迁移前备份凭证，返回备份标识（如文件路径）
    async fn backup_credentials(&self, raw_json: &str) -> Option<String> { None }

    /// 迁移成功后清理备份
    async fn cleanup_backup(&self, backup_info: &str) {}

    /// 迁移失败时保留备份
    async fn preserve_backup(&self, backup_info: &str, error: &str) {}
}
```

提供 `NoopStartupHooks` 用于不需要备份的前端（如数据库后端）。

## 启动流程

```text
AppStateBuilder::build()
        │
        ▼
   AppState 就绪
        │
        ▼
run_migration(hooks)
   ├── hooks.backup_credentials()     ← 平台特定备份
   ├── MigrationService.migrate_if_needed()
   ├── 标记失败账户为 Error 状态
   └── hooks.cleanup_backup() 或 hooks.preserve_backup()
        │
        ▼
run_account_restore()
   ├── AccountService.restore_accounts()
   └── restore_completed = true
```

两个阶段可以分开调用（Tauri 阻塞迁移、后台恢复），也可以用 `run_startup()` 一次性执行。

## 各前端接入方式

### Tauri（桌面/移动）

```rust
// 创建平台适配器
let credential_store = Arc::new(TauriCredentialStore::new());
let account_repository = Arc::new(TauriAccountRepository::new(app_handle.clone()));
let domain_metadata_repository = Arc::new(TauriDomainMetadataRepository::new(app_handle));

// 构建 AppState
let state = AppStateBuilder::new()
    .credential_store(credential_store)
    .account_repository(account_repository)
    .domain_metadata_repository(domain_metadata_repository)
    .build()?;

// 阻塞迁移
state.run_migration(&TauriStartupHooks { app_handle }).await;

// 后台恢复
tokio::spawn(async move { state.run_account_restore().await });
```

存储后端：Keychain（桌面）/ Stronghold（Android）+ tauri-plugin-store

### Web（Actix-web）

```rust
let state = AppStateBuilder::new()
    .credential_store(Arc::new(DbCredentialStore::new(db.clone())))
    .account_repository(Arc::new(DbAccountRepository::new(db.clone())))
    .domain_metadata_repository(Arc::new(DbDomainMetadataRepository::new(db)))
    .build()?;

state.run_startup(&NoopStartupHooks).await?;

let state = web::Data::new(state);
```

存储后端：SeaORM（MySQL/PostgreSQL/SQLite）

### TUI/MCP（未来）

```rust
let config_dir = dirs::config_dir().unwrap().join("dns-orchestrator");

let state = AppStateBuilder::new()
    .credential_store(Arc::new(FileCredentialStore::new(&config_dir)))
    .account_repository(Arc::new(FileAccountRepository::new(&config_dir)))
    .domain_metadata_repository(Arc::new(FileDomainMetadataRepository::new(&config_dir)))
    .build()?;

state.run_startup(&NoopStartupHooks).await?;
```

存储后端：加密 JSON 文件

## Crate 依赖关系

```text
dns-orchestrator-provider    (零内部依赖)
│  DnsProvider trait, ProviderCredentials, 工厂函数
│
├──▶ dns-orchestrator-core    (依赖 provider)
│      ServiceContext, 存储 trait, 业务 Service
│
├──▶ dns-orchestrator-app     (依赖 core)
│      AppState, AppStateBuilder, StartupHooks
│
├──▶ dns-orchestrator-tauri   (依赖 app + core + toolbox)
│      Tauri 适配器 + 命令 + TauriStartupHooks
│
├──▶ dns-orchestrator-web     (依赖 app + core)
│      Actix-web 适配器 + HTTP handlers
│
│   dns-orchestrator-toolbox  (独立，零内部依赖)
│      WHOIS / DNS / IP / SSL 工具
```

每个前端只需要：
1. 实现 3 个存储 trait（`CredentialStore`、`AccountRepository`、`DomainMetadataRepository`）
2. 用 `AppStateBuilder` 注入适配器
3. 调用 `run_startup()` 或分步调用 `run_migration()` + `run_account_restore()`
4. 通过 `AppState` 上的 Service 处理业务请求

---

**返回**: [架构文档](./README.md) | [文档中心](../README.md)
