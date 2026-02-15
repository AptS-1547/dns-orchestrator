# dns-orchestrator-app

Platform-agnostic app bootstrap crate for DNS Orchestrator.

[简体中文](./README.zh-CN.md) | English

## Use Cases

This crate is intended for frontends that need shared startup logic and service wiring:

- desktop clients without Tauri runtime coupling
- terminal/CLI frontends
- MCP/server-side orchestration entrypoints

## Features

- Build shared application state with `AppStateBuilder`
- Run unified startup flow:
  - `run_startup()`
  - `run_migration()`
  - `run_account_restore()`
- Inject platform-specific startup behavior with `StartupHooks`
- Use built-in storage adapters (`KeyringCredentialStore` / `SqliteStore`)

## Feature Flags

- `rustls` (default): forwards to `dns-orchestrator-core/rustls`
- `keyring-store`: enables `KeyringCredentialStore` (system keychain)
- `sqlite-store`: enables `SqliteStore` (SQLite + SeaORM)

## Startup Semantics

- Migration runs before account restore.
- Migration failures are logged and startup continues to restore.
- Partial migration failures mark affected accounts as `Error`.
- `StartupHooks` can backup credentials and decide cleanup/preserve behavior.

## Quick Start (SQLite Store)

> Requires `sqlite-store` feature.

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

## Quick Start (Keyring Store)

> Requires `keyring-store` feature.

```rust
use std::sync::Arc;

use dns_orchestrator_app::adapters::KeyringCredentialStore;

fn create_credential_store() -> Arc<KeyringCredentialStore> {
    Arc::new(KeyringCredentialStore::new())
}
```

## Development

```bash
# Build check with all optional features
cargo check -p dns-orchestrator-app --all-features

# Run tests for this crate (current tests require sqlite-store)
cargo test -p dns-orchestrator-app --features sqlite-store
```

## Structure

- `src/lib.rs`: `AppState`, `AppStateBuilder`, `StartupHooks`
- `src/adapters/`: built-in adapters (Keyring / SQLite)
- `tests/`: startup and SQLite store integration tests

## License

MIT
