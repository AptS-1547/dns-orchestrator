# Architecture Documentation

This document provides an in-depth look at the architectural design of DNS Orchestrator, explaining the key components, design patterns, and technical decisions.

## Table of Contents

- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Project Structure](#project-structure)
- [Frontend Architecture](#frontend-architecture)
- [App Bootstrap Layer](#app-bootstrap-layer)
- [Core Library](#core-library)
- [Backend Architecture](#backend-architecture)
- [Provider Library](#provider-library)
- [Security Architecture](#security-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)

## Overview

DNS Orchestrator is a cross-platform application built with a **five-layer architecture**:

```
Frontend → Backend → App Bootstrap → Core Library → Provider Library → DNS APIs
```

- **Frontend**: React-based UI with TypeScript, Tailwind CSS, and Zustand for state management
- **Backend**: Rust-based Tauri commands (desktop/mobile), with actix-web backend for web
- **App Bootstrap**: Platform-agnostic service assembly and startup (`dns-orchestrator-app` crate)
- **Core Library**: Platform-agnostic business logic (`dns-orchestrator-core` crate)
- **Provider Library**: Standalone `dns-orchestrator-provider` crate for DNS provider integrations
- **Communication**: Transport abstraction layer supports both Tauri IPC and HTTP

### Technology Choices

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **UI Framework** | React 19 + TypeScript 5 | Strong ecosystem, type safety, component reusability |
| **State Management** | Zustand 5 | Lightweight, no boilerplate, simple API |
| **Styling** | Tailwind CSS 4 | Utility-first, rapid development, consistent design |
| **Desktop Framework** | Tauri 2 | Smaller bundle size than Electron, Rust security benefits |
| **Core Library** | Standalone Rust crate | Platform-agnostic business logic, trait-based DI |
| **Web Backend** | actix-web | High performance, async, production-ready |
| **Provider Library** | Standalone Rust crate | Reusable across Tauri and web backends |
| **HTTP Client** | reqwest | Industry standard, async, TLS support |
| **Credential Storage** | keyring / Stronghold | Cross-platform system keychain integration |
| **Build Tool** | Vite 7 | Fast HMR, optimized production builds |

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                               │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  React Components (src/components/)                           │  │
│  │  - AccountList, DnsRecordTable, DomainList, Toolbox           │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Zustand Stores (src/stores/)                                 │  │
│  │  - accountStore, dnsStore, domainStore, toolboxStore          │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Service Layer (src/services/)                                │  │
│  │  - accountService, dnsService, domainService, toolboxService  │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Transport Abstraction (src/services/transport/)              │  │
│  │  - ITransport interface                                       │  │
│  │  - TauriTransport (Tauri IPC) | HttpTransport (REST API)      │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
└─────────────────────────┼────────────────────────────────────────────┘
                          │
        ┌─────────────────┴─────────────────┐
        │                                   │
        ▼ Tauri IPC                         ▼ HTTP REST
┌───────────────────────────┐    ┌───────────────────────────┐
│   TAURI BACKEND           │    │   ACTIX-WEB BACKEND       │
│   (dns-orchestrator-tauri)│    │   (dns-orchestrator-web)  │
│                           │    │                           │
│  ┌─────────────────────┐  │    │  ┌─────────────────────┐  │
│  │  Commands Layer     │  │    │  │  HTTP Handlers      │  │
│  │  - account.rs       │  │    │  │  (REST endpoints)   │  │
│  │  - dns.rs           │  │    │  │                     │  │
│  │  - domain.rs        │  │    │  └──────────┬──────────┘  │
│  │  - toolbox.rs       │  │    │             │             │
│  └──────────┬──────────┘  │    │  ┌──────────▼──────────┐  │
│             │             │    │  │  SeaORM Database    │  │
│  ┌──────────▼──────────┐  │    │  │  (MySQL/PG/SQLite)  │  │
│  │  Adapters           │  │    │  └─────────────────────┘  │
│  │  - TauriCredStore   │  │    │                           │
│  │  - TauriAccountRepo │  │    └───────────┬───────────────┘
│  │  - TauriMetadataRepo│  │                │
│  └──────────┬──────────┘  │                │
└─────────────┼─────────────┘                │
              │                              │
              └──────────────┬───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  APP BOOTSTRAP               │
              │  (dns-orchestrator-app)      │
              │                              │
              │  ┌────────────────────────┐  │
              │  │  AppStateBuilder      │  │
              │  │  + StartupHooks       │  │
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  AppState             │  │
              │  │  (service container)  │  │
              │  │  + run_migration()    │  │
              │  │  + run_account_restore│  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  CORE LIBRARY                │
              │  (dns-orchestrator-core)     │
              │                              │
              │  ┌────────────────────────┐  │
              │  │  ServiceContext        │  │
              │  │  + CredentialStore     │  │
              │  │  + AccountRepository   │  │
              │  │  + ProviderRegistry    │  │
              │  │  + DomainMetadataRepo  │  │
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  Business Services     │  │
              │  │  - AccountService      │  │
              │  │  - DnsService          │  │
              │  │  - DomainService       │  │
              │  │  - DomainMetadataSvc   │  │
              │  │  - ImportExportSvc     │  │
              │  │  - MigrationService    │  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  PROVIDER LIBRARY            │
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
              │  │  Provider Impls        │  │
              │  │  - CloudflareProvider  │  │
              │  │  - AliyunProvider      │  │
              │  │  - DnspodProvider      │  │
              │  │  - HuaweicloudProvider │  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │ HTTPS
              ┌──────────────▼───────────────┐
              │       EXTERNAL DNS APIS       │
              │  Cloudflare | Aliyun | DNSPod │
              │  Huawei Cloud                 │
              └───────────────────────────────┘
```

## Project Structure

```
dns-orchestrator/
├── src/                              # Frontend (React + TypeScript)
│   ├── components/                   # React components by feature
│   ├── services/                     # Service layer + Transport abstraction
│   ├── stores/                       # Zustand state management
│   ├── types/                        # TypeScript type definitions
│   ├── i18n/                         # Internationalization
│   ├── hooks/                        # Custom React hooks
│   └── lib/                          # Utility functions
│
├── dns-orchestrator-provider/        # DNS Provider Library (零内部依赖)
│   └── src/
│       ├── traits.rs                 # DnsProvider trait
│       ├── types.rs                  # ProviderCredentials, DnsRecord, etc.
│       ├── factory.rs                # create_provider(), metadata
│       └── providers/                # Cloudflare, Aliyun, DNSPod, HuaweiCloud
│
├── dns-orchestrator-core/            # Core Business Logic Library
│   └── src/
│       ├── services/                 # ServiceContext + 7 Services
│       │   ├── mod.rs                # ServiceContext (DI container)
│       │   ├── account_service.rs    # Unified account service
│       │   ├── dns_service.rs        # DNS record operations
│       │   ├── domain_service.rs     # Domain listing
│       │   ├── domain_metadata_service.rs
│       │   ├── import_export_service.rs
│       │   ├── migration_service.rs
│       │   └── provider_metadata_service.rs
│       └── traits/                   # 4 storage traits
│           ├── credential_store.rs   # CredentialStore
│           ├── account_repository.rs # AccountRepository
│           ├── provider_registry.rs  # ProviderRegistry + InMemory impl
│           └── domain_metadata_repository.rs
│
├── dns-orchestrator-app/             # App Bootstrap Layer
│   └── src/lib.rs                    # AppState, AppStateBuilder, StartupHooks
│
├── dns-orchestrator-toolbox/         # Network Diagnostic Tools (独立)
│   └── src/services/                 # WHOIS, DNS, IP, SSL, HTTP, DNSSEC
│
├── dns-orchestrator-tauri/           # Tauri Frontend (Desktop/Mobile)
│   └── src/
│       ├── lib.rs                    # TauriStartupHooks, run()
│       ├── commands/                 # Tauri command handlers (thin wrappers)
│       └── adapters/                 # TauriCredentialStore, TauriAccountRepo, etc.
│
├── dns-orchestrator-web/             # Web Frontend (Actix-web, WIP)
│   └── src/main.rs
│
└── vite.config.ts                    # Platform-aware build config
```

## Frontend Architecture

### Service Layer

The service layer abstracts backend communication:

```typescript
// src/services/transport/types.ts
export interface ITransport {
  invoke<K extends NoArgsCommands>(command: K): Promise<CommandMap[K]["result"]>
  invoke<K extends WithArgsCommands>(
    command: K,
    args: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]>
}

// CommandMap provides type-safe command definitions
export interface CommandMap {
  list_accounts: { args: Record<string, never>; result: ApiResponse<Account[]> }
  create_account: { args: { request: CreateAccountRequest }; result: ApiResponse<Account> }
  // ... all 24 commands with full type safety
}
```

**Transport Implementations**:

```typescript
// src/services/transport/tauri.transport.ts (Desktop/Mobile)
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

**Build-time Transport Selection**:

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

### Component Structure

Components are organized by feature domain:

```
src/components/
├── account/              # Account forms, dialogs
├── accounts/             # Accounts page, batch actions
├── dns/                  # DNS record table, forms, row
├── domain/               # Domain list, selector
├── domains/              # Domain page
├── home/                 # Home dashboard
├── toolbox/              # WHOIS, DNS, IP, SSL lookup
├── settings/             # Settings page
├── layout/               # RootLayout, Sidebar
├── navigation/           # Breadcrumb, tabs
├── titlebar/             # Window controls
├── error/                # Error boundary
└── ui/                   # Radix UI wrappers (shadcn/ui)
```

### State Management (Zustand)

Each feature domain has its own store with fine-grained selectors:

```typescript
// src/stores/dnsStore.ts
interface DnsStore {
  // State
  records: DnsRecord[]
  currentPage: number
  pageSize: number
  totalCount: number
  hasMore: boolean
  searchQuery: string
  filterType: RecordType | 'ALL'
  selectedIds: Set<string>

  // Actions
  fetchRecords: (accountId: string, domainId: string) => Promise<void>
  createRecord: (request: CreateDnsRecordRequest) => Promise<void>
  updateRecord: (recordId: string, request: UpdateDnsRecordRequest) => Promise<void>
  deleteRecord: (recordId: string, domainId: string) => Promise<void>
  batchDelete: (recordIds: string[], domainId: string) => Promise<BatchDeleteResult>

  // Selection
  toggleSelection: (recordId: string) => void
  selectAll: () => void
  clearSelection: () => void
}

// Usage with useShallow to optimize re-renders
const { records, hasMore } = useDnsStore(useShallow(state => ({
  records: state.records,
  hasMore: state.hasMore,
})))
```

## App Bootstrap Layer

> See [App Bootstrap Layer Design](./app-bootstrap.md) for full documentation.

The `dns-orchestrator-app` crate provides platform-agnostic service assembly and startup:

```rust
// Any frontend can build a fully-wired AppState:
let state = AppStateBuilder::new()
    .credential_store(platform_credential_store)
    .account_repository(platform_account_repo)
    .domain_metadata_repository(platform_metadata_repo)
    .build()?;

// Run startup sequence (migration + account restoration):
state.run_startup(&platform_hooks).await?;
```

This layer sits between the frontend backends and the Core library, eliminating duplicated service assembly code across frontends.

## Core Library

The `dns-orchestrator-core` crate provides **platform-agnostic business logic** through trait-based dependency injection.

### ServiceContext

The central dependency container:

```rust
// dns-orchestrator-core/src/services/mod.rs
pub struct ServiceContext {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
    provider_registry: Arc<dyn ProviderRegistry>,
    domain_metadata_repository: Arc<dyn DomainMetadataRepository>,
}

impl ServiceContext {
    /// Get provider instance for an account
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// Mark account as invalid (credential error)
    pub async fn mark_account_invalid(&self, account_id: &str, error: &str) -> CoreResult<()> {
        self.account_repository.update_status(account_id, AccountStatus::Error, Some(error)).await
    }
}
```

### Trait Abstractions

Platform-specific implementations injected via traits:

```rust
// dns-orchestrator-core/src/traits/credential_store.rs

/// Map of account_id -> type-safe provider credentials
pub type CredentialsMap = HashMap<String, ProviderCredentials>;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn load_all(&self) -> CoreResult<CredentialsMap>;
    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()>;
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>>;
    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()>;
    async fn remove(&self, account_id: &str) -> CoreResult<()>;
    async fn load_raw_json(&self) -> CoreResult<String>;
    async fn save_raw_json(&self, json: &str) -> CoreResult<()>;
}

// dns-orchestrator-core/src/traits/account_repository.rs
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_all(&self) -> CoreResult<Vec<Account>>;
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;
    async fn save(&self, account: &Account) -> CoreResult<()>;
    async fn delete(&self, id: &str) -> CoreResult<()>;
    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()>;
    async fn update_status(&self, id: &str, status: AccountStatus, error: Option<String>) -> CoreResult<()>;
}

// dns-orchestrator-core/src/traits/provider_registry.rs
#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);
    async fn unregister(&self, account_id: &str);
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;
    async fn list_account_ids(&self) -> Vec<String>;
}

// dns-orchestrator-core/src/traits/domain_metadata_repository.rs
#[async_trait]
pub trait DomainMetadataRepository: Send + Sync {
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>>;
    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()>;
    async fn update(&self, key: &DomainMetadataKey, update: &DomainMetadataUpdate) -> CoreResult<()>;
    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()>;
    // ... batch operations, favorites, tags
}
```

### Fine-grained Services

Business logic split into focused services:

| Service | Responsibility |
|---------|---------------|
| `AccountService` | Unified account CRUD, credential management, provider registration, account restoration |
| `DnsService` | DNS record CRUD, batch delete |
| `DomainService` | List domains, get domain details |
| `DomainMetadataService` | Favorites, tags, domain metadata CRUD |
| `ProviderMetadataService` | Query provider metadata (stateless) |
| `ImportExportService` | Encrypted account backup/restore |
| `MigrationService` | Credential format migration (v1.7.0) |

```rust
// Example: AccountService (unified, replaces 4 old services)
pub struct AccountService {
    ctx: Arc<ServiceContext>,
}

impl AccountService {
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
        // 1. Validate credentials with the provider's API
        // 2. Save credentials securely via CredentialStore
        // 3. Register provider instance in ProviderRegistry
        // 4. Save account metadata via AccountRepository
    }

    pub async fn restore_accounts(&self) -> CoreResult<RestoreResult> {
        // Load all accounts + credentials, recreate provider instances
    }
}
```

## Backend Architecture

### Tauri Application Setup

```rust
// dns-orchestrator-tauri/src/lib.rs
// AppState is imported from dns-orchestrator-app (shared across all frontends)
use dns_orchestrator_app::{AppState, AppStateBuilder, StartupHooks};

// Platform adapters are Tauri-specific
let state = AppStateBuilder::new()
    .credential_store(Arc::new(TauriCredentialStore::new()))
    .account_repository(Arc::new(TauriAccountRepository::new(app_handle.clone())))
    .domain_metadata_repository(Arc::new(TauriDomainMetadataRepository::new(app_handle)))
    .build()?;

// Migration (blocking) + account restore (background)
state.run_migration(&TauriStartupHooks { app_handle }).await;
tokio::spawn(async move { state.run_account_restore().await });
```

### Adapter Implementations

The Tauri backend implements core traits:

| Trait | Adapter | Backend |
|-------|---------|---------|
| `CredentialStore` | `TauriCredentialStore` | keyring (Desktop) / Stronghold (Android) |
| `AccountRepository` | `TauriAccountRepository` | tauri-plugin-store (JSON file) |
| `DomainMetadataRepository` | `TauriDomainMetadataRepository` | tauri-plugin-store (JSON file) |
| `ProviderRegistry` | `InMemoryProviderRegistry` | HashMap in memory (from core) |

```rust
// dns-orchestrator-tauri/src/adapters/credential_store.rs
pub struct TauriCredentialStore {
    cache: Arc<RwLock<Option<CredentialsMap>>>,
    #[cfg(target_os = "android")]
    app_handle: AppHandle,
}

#[async_trait]
impl CredentialStore for TauriCredentialStore {
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        // Load all credentials (with cache), then look up by account_id
        let all = self.load_all().await?;
        Ok(all.get(account_id).cloned())
    }

    // Desktop: system keychain via `keyring` crate
    // Android: tauri-plugin-store with app sandbox
}
```

### Platform-Specific Dependencies

```toml
# src-tauri/Cargo.toml

# Desktop (macOS, Windows, Linux)
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

## Provider Library

### Design Goals

1. **Reusability**: Same provider code works in Tauri and actix-web backends
2. **Feature Flags**: Enable providers and TLS backends selectively
3. **Type Safety**: `RecordData` enum for structured DNS record data
4. **Unified Error Handling**: `ProviderError` maps all provider-specific errors

### DnsProvider Trait

```rust
// dns-orchestrator-provider/src/traits.rs
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// Provider identifier (e.g., "cloudflare")
    fn id(&self) -> &'static str;

    /// Provider metadata (type-level, no instance needed)
    fn metadata() -> ProviderMetadata where Self: Sized;

    /// Validate credentials
    async fn validate_credentials(&self) -> Result<bool>;

    /// List domains (paginated)
    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<ProviderDomain>>;

    /// Get domain details
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain>;

    /// List DNS records (paginated + search + filter)
    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams) -> Result<PaginatedResponse<DnsRecord>>;

    /// Create DNS record
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;

    /// Update DNS record
    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest) -> Result<DnsRecord>;

    /// Delete DNS record
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;

    // Batch operations (TODO - see trait docs for implementation plan)
    async fn batch_create_records(&self, requests: &[CreateDnsRecordRequest]) -> Result<BatchCreateResult>;
    async fn batch_update_records(&self, updates: &[BatchUpdateItem]) -> Result<BatchUpdateResult>;
    async fn batch_delete_records(&self, domain_id: &str, record_ids: &[String]) -> Result<BatchDeleteResult>;
}
```

### Type-Safe Record Data (v1.5.0)

```rust
// dns-orchestrator-provider/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RecordData {
    /// A record: IPv4 address
    A { address: String },

    /// AAAA record: IPv6 address
    AAAA { address: String },

    /// CNAME record: Alias
    CNAME { target: String },

    /// MX record: Mail exchange
    MX { priority: u16, exchange: String },

    /// TXT record: Text
    TXT { text: String },

    /// NS record: Name server
    NS { nameserver: String },

    /// SRV record: Service location
    SRV { priority: u16, weight: u16, port: u16, target: String },

    /// CAA record: Certificate Authority Authorization
    CAA { flags: u8, tag: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub domain_id: String,
    pub name: String,
    pub ttl: u32,
    pub data: RecordData,           // Type-safe record data
    pub proxied: Option<bool>,      // Cloudflare-specific
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
```

### Feature Flags

```toml
# dns-orchestrator-provider/Cargo.toml
[features]
default = ["native-tls", "all-providers"]

# TLS backend (choose one)
native-tls = ["reqwest/native-tls"]     # Desktop default
rustls = ["reqwest/rustls-tls"]          # Android (avoids OpenSSL cross-compile)

# Providers (enable individually or all)
cloudflare = []
aliyun = []
dnspod = []
huaweicloud = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud"]
```

### Error Handling

```rust
// dns-orchestrator-provider/src/error.rs
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    /// Network request failed
    NetworkError { provider: String, detail: String },

    /// Invalid credentials
    InvalidCredentials { provider: String, raw_message: Option<String> },

    /// Record already exists
    RecordExists { provider: String, record_name: String, raw_message: Option<String> },

    /// Record not found
    RecordNotFound { provider: String, record_id: String, raw_message: Option<String> },

    /// Invalid parameter (TTL, value, etc.)
    InvalidParameter { provider: String, param: String, detail: String },

    /// Unsupported record type
    UnsupportedRecordType { provider: String, record_type: String },

    /// Quota exceeded
    QuotaExceeded { provider: String, raw_message: Option<String> },

    /// Domain not found
    DomainNotFound { provider: String, domain: String, raw_message: Option<String> },

    /// Domain locked/disabled
    DomainLocked { provider: String, domain: String, raw_message: Option<String> },

    /// Permission denied
    PermissionDenied { provider: String, raw_message: Option<String> },

    /// Response parse error
    ParseError { provider: String, detail: String },

    /// Serialization error
    SerializationError { provider: String, detail: String },

    /// Unknown error (fallback)
    Unknown { provider: String, raw_code: Option<String>, raw_message: String },
}
```

**Error Mapping**:

Each provider implements `ProviderErrorMapper` to map raw API errors:

```rust
// Internal trait for error mapping
pub(crate) trait ProviderErrorMapper {
    fn provider_name(&self) -> &'static str;
    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError;
}
```

### Provider Structure

Each provider is organized in a subdirectory:

```
providers/cloudflare/
├── mod.rs          # Provider struct, re-exports
├── provider.rs     # DnsProvider trait implementation
├── http.rs         # HTTP client wrapper
├── types.rs        # Cloudflare-specific types
└── error.rs        # ProviderErrorMapper implementation
```

## Security Architecture

### Credential Storage by Platform

| Platform | Storage Mechanism |
|----------|-------------------|
| **macOS** | Keychain via `keyring` crate |
| **Windows** | Credential Manager via `keyring` crate |
| **Linux** | Secret Service (GNOME Keyring/KWallet) via `keyring` crate |
| **Android** | Stronghold via `tauri-plugin-stronghold` |

### Credential Flow

```
Create Account
     │
     ▼
┌─────────────────────────────────────┐
│ AccountService                      │
│                                     │
│ 1. Create provider instance         │
│ 2. Validate credentials (API call)  │
│ 3. Store in CredentialStore         │
│ 4. Register in ProviderRegistry     │
│ 5. Save account in AccountRepository│
└─────────────────────────────────────┘
```

### Account Import/Export Encryption

```rust
// AES-GCM encryption with PBKDF2 key derivation
pub fn encrypt_data(data: &str, password: &str) -> Result<String>
pub fn decrypt_data(encrypted: &str, password: &str) -> Result<String>
```

## Performance Optimizations

1. **Pagination**: Server-side pagination with configurable page size
2. **Search Debouncing**: 300ms debounce on search input
3. **Infinite Scroll**: IntersectionObserver-based loading
4. **Memory Cache**: Credentials and accounts cached in memory
5. **Background Restoration**: Account restoration runs async, doesn't block startup
6. **Rust Async**: Tokio async runtime for non-blocking I/O
7. **Feature Flags**: Only compile enabled providers
8. **useShallow**: Fine-grained Zustand subscriptions

## Data Flow

### DNS Record Query Flow

```
1. User types in search box (debounced 300ms)
2. dnsStore.fetchRecords() called
3. dnsService.listRecords() invoked
4. Transport.invoke('list_dns_records', args)
5. Route to backend:
   ├─ Tauri: IPC to Rust command
   └─ Web: HTTP POST to actix-web
6. Command handler calls DnsService
7. DnsService gets provider from ServiceContext
8. Provider makes HTTPS request to DNS API
9. Response flows back through layers
10. Store updates, UI re-renders
```

### Account Creation Flow

```
Frontend                           Backend                          Core
   │                                  │                               │
   │ createAccount(request)           │                               │
   ├─────────────────────────────────►│                               │
   │                                  │ AccountService                │
   │                                  │      .create_account()        │
   │                                  ├──────────────────────────────►│
   │                                  │                               │
   │                                  │  1. create_provider()         │
   │                                  │  2. provider.validate()       │
   │                                  │  3. CredentialStore.set()     │
   │                                  │  4. ProviderRegistry.register │
   │                                  │  5. AccountRepository.save()  │
   │                                  │                               │
   │◄─────────────────────────────────┤◄──────────────────────────────┤
   │        Account                   │                               │
```

## Design Decisions

### Why Separate Core Library?

| Benefit | Description |
|---------|-------------|
| **Platform Agnostic** | Same business logic for Tauri, actix-web, CLI |
| **Testability** | Mock traits for unit testing |
| **Single Responsibility** | Backend adapters only handle platform APIs |
| **Type Safety** | Shared types across all backends |

### Why Separate Provider Library?

| Benefit | Description |
|---------|-------------|
| **Reusability** | Same code for Tauri and actix-web backends |
| **Testability** | Unit test providers independently |
| **Feature Flags** | Compile only needed providers |
| **TLS Flexibility** | Switch between native-tls and rustls per platform |

### Why Transport Abstraction?

| Benefit | Description |
|---------|-------------|
| **Multi-Platform** | Same frontend code for desktop, mobile, and web |
| **Type Safety** | CommandMap enforces correct args/return types |
| **Testability** | Mock transport for frontend testing |

### Why App Bootstrap Layer?

| Benefit | Description |
|---------|-------------|
| **No Duplication** | Service assembly and startup logic shared across all frontends |
| **Easy Onboarding** | New frontend only needs 3 adapter implementations |
| **Consistent Startup** | Migration and account restoration work the same everywhere |
| **Platform Hooks** | `StartupHooks` trait allows platform-specific backup without coupling |

### Why Focused Services?

| Benefit | Description |
|---------|-------------|
| **Single Responsibility** | Each service has one clear purpose |
| **Composability** | Services can be composed (e.g., AccountLifecycle) |
| **Testability** | Test each service in isolation |
| **Flexibility** | Replace or extend individual services |

### Why actix-web for Web Backend?

| Criterion | actix-web | axum |
|-----------|-----------|------|
| **Performance** | Fastest Rust web framework | Very fast |
| **Maturity** | Battle-tested in production | Newer |
| **Ecosystem** | Large plugin ecosystem | Growing |

---

This architecture balances simplicity, security, and performance while supporting multiple platforms with shared code.
