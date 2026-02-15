# dns-orchestrator-core Architecture

This document describes the current architecture of `dns-orchestrator-core`.

It reflects the post-refactor service model where account responsibilities are unified into `AccountService`, and domain metadata is a first-class dependency in `ServiceContext`.

## Overview

`dns-orchestrator-core` is a platform-agnostic business layer:

- It does not own UI, file-system policy, or database schema.
- It depends on storage/runtime traits injected by the platform layer.
- It delegates provider API calls to `dns-orchestrator-provider`.

Relationship:

```
Platform Layer (Tauri / Actix-Web / custom backend)
    -> injects repositories + stores + registry into ServiceContext
    -> calls core services

Core Layer (dns-orchestrator-core)
    -> account/domain/dns/metadata/import-export/migration logic
    -> provider error translation and account status management

Provider Layer (dns-orchestrator-provider)
    -> provider implementations and API adapters
```

## Directory Structure

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

## Core Composition

### ServiceContext

`ServiceContext` is the dependency container used by runtime services.

```rust
pub struct ServiceContext {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
    provider_registry: Arc<dyn ProviderRegistry>,
    domain_metadata_repository: Arc<dyn DomainMetadataRepository>,
}
```

Key responsibilities:

- provide typed accessors for injected dependencies
- resolve provider by `account_id`
- handle provider errors and mark account invalid on invalid-credential failures

Important behavior:

- `handle_provider_error()` converts `ProviderError -> CoreError`
- if the provider error is `InvalidCredentials`, account status is changed to `Error`

### Trait Contracts

The platform must implement these traits.

1. `AccountRepository`
- account metadata CRUD (`find_all`, `find_by_id`, `save`, `delete`, `save_all`)
- account status mutation (`update_status`)

2. `CredentialStore`
- typed credential operations (`load_all`, `save_all`, `get`, `set`, `remove`)
- migration helpers (`load_raw_json`, `save_raw_json`)
- value type is `ProviderCredentials` (not raw hash maps)

3. `ProviderRegistry`
- in-memory runtime registry for `account_id -> Arc<dyn DnsProvider>`
- default implementation: `InMemoryProviderRegistry`

4. `DomainMetadataRepository`
- metadata query/update/delete (single + batch)
- favorites and tag index queries
- account-scope cleanup (`delete_by_account`)

## Service Layer

### 1) AccountService

`AccountService` is the unified account domain service.

Replaces old split services (`AccountBootstrapService`, `AccountLifecycleService`, `AccountMetadataService`, `CredentialManagementService`) with one cohesive API.

Main responsibilities:

- account CRUD and status updates
- credential validation and persistence
- provider register/unregister
- startup restore (`restore_accounts`)
- import-path account creation (`create_account_from_import`)

Design notes:

- `create_account` flow validates credentials before persistence.
- if metadata save fails after credential save, service performs cleanup.
- `delete_account` uses safe order: delete credentials first, then remove runtime provider, then delete metadata.

### 2) DnsService

DNS record operations on a provider bound to an account:

- `list_records`
- `create_record`
- `update_record`
- `delete_record`
- `batch_delete_records` (concurrent deletes with bounded parallelism)

Error handling is routed via `ServiceContext::handle_provider_error`.

### 3) DomainMetadataService

Owns user metadata attached to domain keys (`account_id + domain_id`):

- favorite state (`toggle_favorite`, `list_favorites`)
- tags (`add_tag`, `remove_tag`, `set_tags`)
- batch tag operations (`batch_add_tags`, `batch_remove_tags`, `batch_set_tags`)
- partial updates (`update_metadata`)

Validation rules enforced in service:

- tag non-empty, max length 50, max 10 tags
- color must be one of predefined keys
- note length max 500

### 4) DomainService

Domain read service:

- `list_domains`
- `get_domain`

`list_domains` merges provider domains with metadata from `DomainMetadataService` in batch mode.

Constructor:

```rust
pub fn new(ctx: Arc<ServiceContext>, metadata_service: Arc<DomainMetadataService>) -> Self
```

### 5) ImportExportService

Imports/exports accounts as `.dnso` JSON payloads.

Constructor depends on `AccountService`:

```rust
pub fn new(account_service: Arc<AccountService>) -> Self
```

APIs:

- `export_accounts`
- `preview_import`
- `import_accounts`

Behavior:

- optional encryption at export time
- encrypted import supports password-required preview response
- imported accounts are created through `AccountService::create_account_from_import`

### 6) MigrationService

Migrates legacy credential payloads into typed credentials.

- detects migration need using `CredentialStore::load_all`
- if `CoreError::MigrationRequired`, reads raw JSON and transforms entries based on account provider type
- writes migrated values via `save_all`

Notes:

- backup policy is intentionally outside core (platform layer responsibility)

### 7) ProviderMetadataService

Stateless wrapper over provider metadata catalog (`list_providers`).

## Type System

### Core-owned types

- account: `Account`, `AccountStatus`, create/update requests
- dns: `BatchDeleteRequest`
- domain: `AppDomain` (extends provider domain with `account_id` + optional metadata)
- metadata: `DomainMetadata`, `DomainMetadataKey`, batch tag types
- import/export: `ExportFile`, `ImportPreview`, `ImportResult`, request/response types

### Re-exported provider types

`types/mod.rs` re-exports common provider-level types such as:

- `DnsRecord`, `ProviderDomain`, `ProviderMetadata`
- `ProviderCredentials`, `ProviderType`
- pagination and record request/response types

This keeps platform integration code importing from one crate in most cases.

## Error Model

`CoreError` is the unified error type.

Key categories:

- domain/account/provider lookup errors
- storage and serialization errors
- validation and import/export errors
- migration-specific errors
- wrapped provider errors (`CoreError::Provider`)

`CoreError::is_expected()` is used for log-level classification of user/expected failures.

## Crypto Module

The import/export encryption module (`src/crypto`) provides:

- AES-256-GCM authenticated encryption
- PBKDF2-HMAC-SHA256 key derivation
- file-version-based iteration lookup

Version mapping (`versions.rs`):

- v1: 100,000 iterations
- v2: 600,000 iterations

`CURRENT_FILE_VERSION` controls export format version.

## Runtime Flows

### Startup bootstrap

1. (optional) run `MigrationService::migrate_if_needed`
2. create `AccountService`
3. call `restore_accounts`
4. registry now holds provider instances for valid accounts

### Account create/update/delete

1. validate credentials and build provider
2. persist credentials
3. register provider
4. persist account metadata
5. on failure, rollback where possible

### Domain listing with metadata

1. `DomainService` fetches provider domains
2. converts to `AppDomain`
3. batch-loads metadata by `(account_id, domain_id)`
4. merges metadata into response items

## Platform Integration Guide

Minimal bootstrap sequence:

1. implement all four storage traits
2. build `ServiceContext`
3. create service instances in dependency order:
   - `AccountService`
   - `DomainMetadataService`
   - `DomainService`
   - `DnsService`
   - `ImportExportService`
   - `MigrationService`
4. call migration/restore in startup lifecycle

Recommended ownership pattern:

- keep `Arc<ServiceContext>` and service singletons at app scope
- do not bypass services to mutate repositories directly
- route provider-call failures through core services so account invalidation logic stays consistent
