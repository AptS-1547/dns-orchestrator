# dns-orchestrator-core

DNS Orchestrator 的平台无关核心业务逻辑库。

简体中文 | [English](./README.md)

## 功能特性

- **统一账户服务** - 使用单一 `AccountService` 处理账户 CRUD、凭证校验/存储、Provider 注册和启动恢复
- **基于 Trait 的存储抽象** - 通过 `AccountRepository`、`CredentialStore`、`ProviderRegistry`、`DomainMetadataRepository` 注入平台实现
- **DNS 记录管理** - 统一服务 API 支持 DNS 记录列表、创建、更新、删除与批量删除
- **域名元数据层** - 支持收藏、标签、颜色、备注及单条/批量操作
- **加密导入导出** - 支持 `.dnso` 账号备份/导入，可选 AES-256-GCM 加密
- **凭证迁移支持** - 支持旧凭证格式迁移到类型安全的 `ProviderCredentials`

## 核心服务

| 服务 | 说明 |
|------|------|
| `AccountService` | 账户生命周期、凭证操作、Provider 注册、启动恢复 |
| `DnsService` | DNS 记录列表/创建/更新/删除与批量删除 |
| `DomainService` | 域名列表与元数据合并 |
| `DomainMetadataService` | 收藏/标签/颜色/备注元数据管理 |
| `ImportExportService` | 账户导入导出与加密预览流程 |
| `MigrationService` | 旧凭证格式迁移 |
| `ProviderMetadataService` | 支持的 Provider 元数据查询 |

## 快速开始

### 安装

```toml
[dependencies]
dns-orchestrator-core = "0.1"
```

### 实现存储 Trait

平台层需要实现：

- `AccountRepository`
- `CredentialStore`
- `DomainMetadataRepository`
- `ProviderRegistry`（或直接使用内置 `InMemoryProviderRegistry`）

### 初始化 Context 与 Services

```rust
use std::sync::Arc;
use dns_orchestrator_core::ServiceContext;
use dns_orchestrator_core::services::{
    AccountService, DnsService, DomainMetadataService, DomainService,
    ImportExportService, MigrationService, ProviderMetadataService,
};
use dns_orchestrator_core::traits::InMemoryProviderRegistry;

let ctx = Arc::new(ServiceContext::new(
    Arc::new(my_credential_store),
    Arc::new(my_account_repository),
    Arc::new(InMemoryProviderRegistry::new()),
    Arc::new(my_domain_metadata_repository),
));

let account_service = Arc::new(AccountService::new(ctx.clone()));
let domain_metadata_service = Arc::new(DomainMetadataService::new(
    ctx.domain_metadata_repository().clone(),
));

let dns_service = DnsService::new(ctx.clone());
let domain_service = DomainService::new(ctx.clone(), domain_metadata_service.clone());
let import_export_service = ImportExportService::new(account_service.clone());
let migration_service = MigrationService::new(
    ctx.credential_store().clone(),
    ctx.account_repository().clone(),
);
let provider_metadata_service = ProviderMetadataService::new();
```

### 启动流程

```rust
let migration = migration_service.migrate_if_needed().await?;
let restore = account_service.restore_accounts().await?;

println!("migration: {:?}", migration);
println!("restored: {}, errors: {}", restore.success_count, restore.error_count);
```

## 架构

```
┌───────────────────────────────────────────────────────────┐
│ 平台层 (Tauri / Actix-Web / 自定义后端)                  │
│ 实现存储 trait 并组装依赖                                 │
└────────────────────────────┬──────────────────────────────┘
                             │ ServiceContext
┌────────────────────────────▼──────────────────────────────┐
│ dns-orchestrator-core                                    │
│ - AccountService（统一生命周期与凭证逻辑）               │
│ - DnsService / DomainService / DomainMetadataService     │
│ - ImportExportService / MigrationService                 │
└────────────────────────────┬──────────────────────────────┘
                             │ DnsProvider trait
┌────────────────────────────▼──────────────────────────────┐
│ dns-orchestrator-provider                                │
│ Cloudflare / Aliyun / DNSPod / Huaweicloud ...           │
└───────────────────────────────────────────────────────────┘
```

详细架构说明见：[docs/ARCHITECTURE.zh-CN.md](./docs/ARCHITECTURE.zh-CN.md)

## 导入导出加密

加密导出使用：

- `AES-256-GCM`
- `PBKDF2-HMAC-SHA256`
- 16 字节随机 salt + 12 字节随机 nonce
- 按文件版本选择 PBKDF2 迭代次数（`v1=100_000`，`v2=600_000`）

版本常量定义在 `src/crypto/versions.rs`。

## 开发

```bash
# 在仓库根目录执行
cargo check -p dns-orchestrator-core
cargo test -p dns-orchestrator-core
```

## 许可证

MIT
