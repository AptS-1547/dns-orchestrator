# dns-orchestrator-core

Platform-agnostic core business logic library for DNS Orchestrator.

[简体中文](./README.zh-CN.md) | English

## Features

- **Unified Account Service** - Single `AccountService` for account CRUD, credential validation/storage, provider registration, and startup restore
- **Trait-Based Storage Abstraction** - Inject platform-specific implementations via `AccountRepository`, `CredentialStore`, `ProviderRegistry`, and `DomainMetadataRepository`
- **DNS Record Management** - List, create, update, delete, and batch delete DNS records through a unified service API
- **Domain Metadata Layer** - Favorites, tags, color, and note metadata with single/batch operations
- **Encrypted Import/Export** - `.dnso` account backup/import with optional AES-256-GCM encryption
- **Credential Migration Support** - Legacy credential format migration to typed `ProviderCredentials`

## Core Services

| Service | Description |
|---------|-------------|
| `AccountService` | Account lifecycle, credential operations, provider registration, startup restore |
| `DnsService` | DNS record list/create/update/delete and batch delete |
| `DomainService` | Domain listing and metadata merge |
| `DomainMetadataService` | Favorites/tags/color/note metadata operations |
| `ImportExportService` | Account export/import and encrypted preview flow |
| `MigrationService` | Legacy credential format migration |
| `ProviderMetadataService` | Supported provider metadata listing |

## Quick Start

### Install

```toml
[dependencies]
dns-orchestrator-core = "0.1"
```

### Implement Storage Traits

Platform layer must implement:

- `AccountRepository`
- `CredentialStore`
- `DomainMetadataRepository`
- `ProviderRegistry` (or use built-in `InMemoryProviderRegistry`)

### Initialize Context and Services

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

### Startup Sequence

```rust
let migration = migration_service.migrate_if_needed().await?;
let restore = account_service.restore_accounts().await?;

println!("migration: {:?}", migration);
println!("restored: {}, errors: {}", restore.success_count, restore.error_count);
```

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│ Platform Layer (Tauri / Actix-Web / custom backend)      │
│ Implements storage traits and wires dependencies          │
└────────────────────────────┬──────────────────────────────┘
                             │ ServiceContext
┌────────────────────────────▼──────────────────────────────┐
│ dns-orchestrator-core                                    │
│ - AccountService (unified lifecycle + credential logic)  │
│ - DnsService / DomainService / DomainMetadataService     │
│ - ImportExportService / MigrationService                 │
└────────────────────────────┬──────────────────────────────┘
                             │ DnsProvider trait
┌────────────────────────────▼──────────────────────────────┐
│ dns-orchestrator-provider                                │
│ Cloudflare / Aliyun / DNSPod / Huaweicloud ...           │
└───────────────────────────────────────────────────────────┘
```

Detailed architecture: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

## Import/Export Encryption

Encrypted export uses:

- `AES-256-GCM`
- `PBKDF2-HMAC-SHA256`
- 16-byte random salt + 12-byte random nonce
- Version-based PBKDF2 iterations (`v1=100_000`, `v2=600_000`)

Version constants are defined in `src/crypto/versions.rs`.

## Development

```bash
# From repository root
cargo check -p dns-orchestrator-core
cargo test -p dns-orchestrator-core
```

## License

MIT
