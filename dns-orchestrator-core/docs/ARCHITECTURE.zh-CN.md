# dns-orchestrator-core 架构文档

本文档描述 `dns-orchestrator-core` 当前架构。

内容基于现有代码实现，已反映服务重构后的模型：账户相关能力统一为 `AccountService`，并将域名元数据仓库纳入 `ServiceContext` 一等依赖。

## 概述

`dns-orchestrator-core` 是平台无关业务层：

- 不负责 UI、文件系统策略或数据库 Schema。
- 依赖平台层注入的存储/运行时 trait 实现。
- 通过 `dns-orchestrator-provider` 执行具体 DNS 厂商 API 调用。

分层关系：

```
平台层 (Tauri / Actix-Web / 自定义后端)
    -> 注入 repository/store/registry 到 ServiceContext
    -> 调用 core services

核心层 (dns-orchestrator-core)
    -> 账户/域名/DNS/元数据/导入导出/迁移业务逻辑
    -> Provider 错误转换与账户状态管理

Provider 层 (dns-orchestrator-provider)
    -> 各厂商实现与 API 适配
```

## 目录结构

```text
src/
├── lib.rs
├── error.rs
├── crypto/
│   ├── mod.rs
│   └── versions.rs
├── services/
│   ├── mod.rs
│   ├── account_service.rs
│   ├── dns_service.rs
│   ├── domain_service.rs
│   ├── domain_metadata_service.rs
│   ├── import_export_service.rs
│   ├── migration_service.rs
│   └── provider_metadata_service.rs
├── traits/
│   ├── mod.rs
│   ├── account_repository.rs
│   ├── credential_store.rs
│   ├── domain_metadata_repository.rs
│   └── provider_registry.rs
├── types/
│   ├── mod.rs
│   ├── account.rs
│   ├── dns.rs
│   ├── domain.rs
│   ├── domain_metadata.rs
│   └── export.rs
└── utils/
    ├── mod.rs
    └── datetime.rs
```

## 核心组合

### ServiceContext

`ServiceContext` 是运行时依赖容器。

```rust
pub struct ServiceContext {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
    provider_registry: Arc<dyn ProviderRegistry>,
    domain_metadata_repository: Arc<dyn DomainMetadataRepository>,
}
```

核心职责：

- 提供注入依赖的类型化访问器
- 按 `account_id` 获取 provider 实例
- 统一处理 provider 错误，并在凭证失效时标记账户为 `Error`

关键行为：

- `handle_provider_error()` 负责 `ProviderError -> CoreError` 转换
- 遇到 `InvalidCredentials` 时自动调用 `mark_account_invalid()`

### Trait 合约

平台层必须实现以下 trait。

1. `AccountRepository`
- 账户元数据 CRUD：`find_all`、`find_by_id`、`save`、`delete`、`save_all`
- 账户状态变更：`update_status`

2. `CredentialStore`
- 类型安全凭证操作：`load_all`、`save_all`、`get`、`set`、`remove`
- 迁移辅助：`load_raw_json`、`save_raw_json`
- 值类型为 `ProviderCredentials`，不再是原始 map

3. `ProviderRegistry`
- 运行时内存注册表：`account_id -> Arc<dyn DnsProvider>`
- 默认实现：`InMemoryProviderRegistry`

4. `DomainMetadataRepository`
- 元数据查询/更新/删除（单条 + 批量）
- 收藏与标签索引查询
- 账户级清理：`delete_by_account`

## 服务层

### 1) AccountService

`AccountService` 是统一账户服务。

它替代旧拆分模型（`AccountBootstrapService`、`AccountLifecycleService`、`AccountMetadataService`、`CredentialManagementService`），对外提供一致 API。

主要职责：

- 账户 CRUD 与状态更新
- 凭证校验与持久化
- provider 注册与注销
- 启动恢复：`restore_accounts`
- 导入路径建号：`create_account_from_import`

设计要点：

- `create_account` 在持久化前先校验凭证
- 元数据写入失败时，会清理已保存凭证并注销 provider
- `delete_account` 采用安全顺序：先删凭证，再清理运行时 provider，最后删账户元数据

### 2) DnsService

基于账户绑定 provider 的 DNS 记录服务：

- `list_records`
- `create_record`
- `update_record`
- `delete_record`
- `batch_delete_records`（有并发上限）

Provider 错误统一走 `ServiceContext::handle_provider_error`。

### 3) DomainMetadataService

管理域名元数据（键为 `account_id + domain_id`）：

- 收藏状态：`toggle_favorite`、`list_favorites`
- 标签操作：`add_tag`、`remove_tag`、`set_tags`
- 批量标签：`batch_add_tags`、`batch_remove_tags`、`batch_set_tags`
- 部分更新：`update_metadata`

服务内置校验规则：

- 标签非空、单个最大 50 字符、最多 10 个
- 颜色必须是预定义键值
- 备注最长 500 字符

### 4) DomainService

域名读取服务：

- `list_domains`
- `get_domain`

`list_domains` 会批量读取 `DomainMetadataService` 元数据并合并到返回结果。

构造函数：

```rust
pub fn new(ctx: Arc<ServiceContext>, metadata_service: Arc<DomainMetadataService>) -> Self
```

### 5) ImportExportService

以 `.dnso` JSON 结构执行账户导入导出。

构造依赖 `AccountService`：

```rust
pub fn new(account_service: Arc<AccountService>) -> Self
```

核心 API：

- `export_accounts`
- `preview_import`
- `import_accounts`

行为说明：

- 导出可选加密
- 加密导入支持“需密码”预览反馈
- 导入落库通过 `AccountService::create_account_from_import`

### 6) MigrationService

将旧凭证格式迁移为类型安全格式。

- 通过 `CredentialStore::load_all` 检测是否需要迁移
- 收到 `CoreError::MigrationRequired` 时读取原始 JSON 并按账户 provider 类型转换
- 使用 `save_all` 批量写回

说明：

- 备份策略不在 core 层处理，由平台层负责

### 7) ProviderMetadataService

无状态服务，仅返回 provider 元数据列表（`list_providers`）。

## 类型系统

### Core 自有类型

- account：`Account`、`AccountStatus`、创建/更新请求
- dns：`BatchDeleteRequest`
- domain：`AppDomain`（在 provider domain 基础上附加 `account_id` 与可选 metadata）
- metadata：`DomainMetadata`、`DomainMetadataKey`、批量标签类型
- import/export：`ExportFile`、`ImportPreview`、`ImportResult` 等请求响应类型

### Provider 类型重导出

`types/mod.rs` 重导出了常用 provider 类型，例如：

- `DnsRecord`、`ProviderDomain`、`ProviderMetadata`
- `ProviderCredentials`、`ProviderType`
- 分页与记录查询类型

这样平台层通常可从一个 crate 完成主要类型导入。

## 错误模型

统一错误类型为 `CoreError`。

主要类别：

- 账户/域名/provider 查找错误
- 存储与序列化错误
- 校验与导入导出错误
- 迁移相关错误
- provider 错误包装：`CoreError::Provider`

`CoreError::is_expected()` 用于日志分级（预期错误 vs 非预期错误）。

## 加密模块

导入导出加密位于 `src/crypto`，提供：

- AES-256-GCM
- PBKDF2-HMAC-SHA256
- 基于文件版本号的迭代次数映射

`versions.rs` 当前映射：

- v1: 100,000 次
- v2: 600,000 次

`CURRENT_FILE_VERSION` 决定导出文件版本。

## 关键运行流程

### 启动流程

1. （可选）执行 `MigrationService::migrate_if_needed`
2. 创建 `AccountService`
3. 调用 `restore_accounts`
4. 有效账户的 provider 被注册进 registry

### 账户创建/更新/删除

1. 校验凭证并创建 provider
2. 保存凭证
3. 注册 provider
4. 保存账户元数据
5. 失败时执行可回滚清理

### 域名列表 + 元数据合并

1. `DomainService` 先从 provider 拉取域名列表
2. 转成 `AppDomain`
3. 按 `(account_id, domain_id)` 批量读取 metadata
4. 合并 metadata 后返回

## 平台接入建议

最小接入顺序：

1. 实现四个存储 trait
2. 组装 `ServiceContext`
3. 按依赖顺序创建服务实例：
   - `AccountService`
   - `DomainMetadataService`
   - `DomainService`
   - `DnsService`
   - `ImportExportService`
   - `MigrationService`
4. 在应用启动生命周期中执行迁移与恢复

推荐约束：

- 在应用级别持有 `Arc<ServiceContext>` 与服务单例
- 避免绕过服务层直接写 repository
- provider 调用异常优先通过 core service 返回，保证账户失效标记逻辑一致
