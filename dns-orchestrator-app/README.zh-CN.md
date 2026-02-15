# dns-orchestrator-app

DNS Orchestrator 的平台无关引导层 crate。

简体中文 | [English](./README.md)

## 适用场景

这个 crate 面向需要复用启动逻辑和服务装配的前端/入口程序：

- 不依赖 Tauri 运行时的桌面端
- 终端或 CLI 前端
- MCP / 服务端编排入口

## 功能特性

- 通过 `AppStateBuilder` 组装共享应用状态
- 提供统一启动流程：
  - `run_startup()`
  - `run_migration()`
  - `run_account_restore()`
- 通过 `StartupHooks` 注入平台特定的启动逻辑
- 内置存储适配器（`KeyringCredentialStore` / `SqliteStore`）

## Feature Flags

- `rustls`（默认）：透传到 `dns-orchestrator-core/rustls`
- `keyring-store`：启用 `KeyringCredentialStore`（系统钥匙串）
- `sqlite-store`：启用 `SqliteStore`（SQLite + SeaORM）

## 启动语义

- 先执行迁移，再执行账户恢复。
- 迁移失败会记录日志，启动流程仍会继续进入恢复阶段。
- 部分迁移失败时，会把对应账户标记为 `Error`。
- `StartupHooks` 可用于备份凭证，并决定清理或保留备份。

## 快速开始（SQLite 存储）

> 需要启用 `sqlite-store`。

```rust
use std::sync::Arc;

use dns_orchestrator_app::adapters::SqliteStore;
use dns_orchestrator_app::{AppStateBuilder, NoopStartupHooks};

async fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::path::Path::new("./data/dns-orchestrator.db");

    let store = Arc::new(SqliteStore::new(
        db_path,
        Some("replace-with-your-password".to_string()),
    ).await?);

    let app = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store)
        .build()?;

    app.run_startup(&NoopStartupHooks).await?;
    Ok(())
}
```

## 快速开始（Keyring 存储）

> 需要启用 `keyring-store`。

```rust
use std::sync::Arc;

use dns_orchestrator_app::adapters::KeyringCredentialStore;

fn create_credential_store() -> Arc<KeyringCredentialStore> {
    Arc::new(KeyringCredentialStore::new())
}
```

## 开发

```bash
# 编译检查（含全部可选 feature）
cargo check -p dns-orchestrator-app --all-features

# 运行该 crate 测试（当前测试依赖 sqlite-store）
cargo test -p dns-orchestrator-app --features sqlite-store
```

## 目录结构

- `src/lib.rs`：`AppState`、`AppStateBuilder`、`StartupHooks`
- `src/adapters/`：内置适配器（Keyring / SQLite）
- `tests/`：启动流程与 SQLite 存储集成测试

## 许可证

MIT
