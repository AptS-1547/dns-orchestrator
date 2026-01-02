# Architecture Documentation

This document provides an in-depth look at the architectural design of DNS Orchestrator, explaining the key components, design patterns, and technical decisions.

## Table of Contents

- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Project Structure](#project-structure)
- [Frontend Architecture](#frontend-architecture)
- [Core Library](#core-library)
- [Backend Architecture](#backend-architecture)
- [Provider Library](#provider-library)
- [Security Architecture](#security-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)

## Overview

DNS Orchestrator is a cross-platform application built with a **four-layer architecture**:

```
Frontend → Backend → Core Library → Provider Library → DNS APIs
```

- **Frontend**: React-based UI with TypeScript, Tailwind CSS, and Zustand for state management
- **Backend**: Rust-based Tauri commands (desktop/mobile), with actix-web backend for web
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
│   (src-tauri/)            │    │   (src-actix-web/)        │
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
│  │  AppState           │  │    │  └─────────────────────┘  │
│  │  (9 services)       │  │    │                           │
│  └──────────┬──────────┘  │    └───────────┬───────────────┘
│             │             │                │
└─────────────┼─────────────┘                │
              │                              │
              └──────────────┬───────────────┘
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
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  Business Services     │  │
              │  │  - AccountLifecycle    │  │
              │  │  - CredentialManagement│  │
              │  │  - DnsService          │  │
              │  │  - DomainService       │  │
              │  │  - ImportExport        │  │
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
│   │   ├── ui/                       # Radix UI wrappers (shadcn/ui)
│   │   ├── account/                  # Account management
│   │   ├── accounts/                 # Accounts page
│   │   ├── dns/                      # DNS record management
│   │   ├── domain/                   # Domain components
│   │   ├── domains/                  # Domain selector page
│   │   ├── toolbox/                  # Network utilities
│   │   ├── settings/                 # Settings page
│   │   ├── layout/                   # Layout components
│   │   ├── navigation/               # Navigation components
│   │   ├── titlebar/                 # Window title bar
│   │   └── error/                    # Error boundary
│   ├── services/                     # Service layer
│   │   ├── transport/                # Transport abstraction
│   │   │   ├── types.ts              # ITransport interface, CommandMap
│   │   │   ├── tauri.transport.ts    # Tauri IPC implementation
│   │   │   └── http.transport.ts     # HTTP REST implementation
│   │   ├── account.service.ts
│   │   ├── dns.service.ts
│   │   ├── domain.service.ts
│   │   ├── toolbox.service.ts
│   │   └── file.service.ts
│   ├── stores/                       # Zustand state management
│   │   ├── accountStore.ts           # Account state + providers
│   │   ├── dnsStore.ts               # DNS records + pagination
│   │   ├── domainStore.ts            # Domains by account
│   │   ├── settingsStore.ts          # Theme, language, debug
│   │   ├── toolboxStore.ts           # Toolbox history
│   │   └── updaterStore.ts           # Auto-update state
│   ├── types/                        # TypeScript type definitions
│   ├── i18n/                         # Internationalization (en, zh-CN)
│   ├── hooks/                        # Custom React hooks
│   ├── lib/                          # Utility functions
│   └── constants/                    # Constants
│
├── dns-orchestrator-core/            # Core Business Logic Library
│   ├── src/
│   │   ├── lib.rs                    # Library entry, re-exports
│   │   ├── error.rs                  # CoreError, CoreResult
│   │   ├── services/                 # Business services
│   │   │   ├── mod.rs                # ServiceContext
│   │   │   ├── account_metadata_service.rs
│   │   │   ├── credential_management_service.rs
│   │   │   ├── account_lifecycle_service.rs
│   │   │   ├── account_bootstrap_service.rs
│   │   │   ├── provider_metadata_service.rs
│   │   │   ├── import_export_service.rs
│   │   │   ├── domain_service.rs
│   │   │   ├── dns_service.rs
│   │   │   └── toolbox/              # Toolbox services
│   │   ├── traits/                   # Platform abstraction traits
│   │   │   ├── credential_store.rs   # CredentialStore trait
│   │   │   ├── account_repository.rs # AccountRepository trait
│   │   │   └── provider_registry.rs  # ProviderRegistry trait
│   │   ├── types/                    # Internal types
│   │   ├── crypto/                   # AES-GCM encryption
│   │   └── utils/                    # Utilities
│   └── Cargo.toml
│
├── dns-orchestrator-provider/        # DNS Provider Library
│   ├── src/
│   │   ├── lib.rs                    # Library entry, re-exports
│   │   ├── traits.rs                 # DnsProvider trait
│   │   ├── types.rs                  # RecordData, ProviderCredentials, etc.
│   │   ├── error.rs                  # ProviderError enum (13 variants)
│   │   ├── factory.rs                # create_provider(), metadata
│   │   ├── http_client.rs            # HTTP client wrapper
│   │   └── providers/                # Provider implementations
│   │       ├── cloudflare/           # Cloudflare (mod, provider, http, types, error)
│   │       ├── aliyun/               # Aliyun DNS
│   │       ├── dnspod/               # Tencent DNSPod
│   │       └── huaweicloud/          # Huawei Cloud DNS
│   ├── tests/                        # Integration tests
│   └── Cargo.toml                    # Feature flags
│
├── src-tauri/                        # Tauri Backend (Desktop/Mobile)
│   ├── src/
│   │   ├── lib.rs                    # AppState, run()
│   │   ├── commands/                 # Tauri command handlers
│   │   │   ├── account.rs            # 10 commands
│   │   │   ├── domain.rs             # 2 commands
│   │   │   ├── dns.rs                # 5 commands
│   │   │   ├── toolbox.rs            # 4 commands
│   │   │   └── updater.rs            # Android-only (3 commands)
│   │   ├── adapters/                 # Core trait implementations
│   │   │   ├── credential_store.rs   # TauriCredentialStore
│   │   │   └── account_repository.rs # TauriAccountRepository
│   │   ├── types.rs                  # Frontend-facing types
│   │   └── error.rs                  # Error conversions
│   ├── capabilities/                 # Tauri 2 permissions
│   └── Cargo.toml                    # Platform-specific deps
│
├── src-actix-web/                    # Web Backend (WIP)
│   ├── src/main.rs                   # Actix-web server entry
│   └── migration/                    # SeaORM database migrations
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

/// Map of account_id -> credential key-value pairs
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

### Fine-grained Services

Business logic split into focused services:

| Service | Responsibility |
|---------|---------------|
| `AccountMetadataService` | Account CRUD (metadata only, no credentials) |
| `CredentialManagementService` | Validate, store, delete credentials |
| `AccountLifecycleService` | Full account lifecycle (combines metadata + credentials) |
| `AccountBootstrapService` | Restore accounts on app startup |
| `ProviderMetadataService` | Query provider metadata (stateless) |
| `ImportExportService` | Encrypted account backup/restore |
| `DomainService` | List domains, get domain details |
| `DnsService` | DNS record CRUD, batch delete |
| `ToolboxService` | WHOIS, DNS, IP, SSL lookups |

```rust
// Example: AccountLifecycleService composition
pub struct AccountLifecycleService {
    metadata_service: Arc<AccountMetadataService>,
    credential_service: Arc<CredentialManagementService>,
}

impl AccountLifecycleService {
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
        // 1. Validate credentials with the provider's API.
        // 2. Save credentials securely using CredentialStore.
        // 3. Register the new provider instance in ProviderRegistry.
        // 4. Save account metadata using AccountRepository.
        // ...
    }
}
```

## Backend Architecture

### Tauri Application State

```rust
// src-tauri/src/lib.rs
pub struct AppState {
    /// Service context (DI container)
    pub ctx: Arc<ServiceContext>,

    /// Fine-grained services
    pub account_metadata_service: Arc<AccountMetadataService>,
    pub credential_management_service: Arc<CredentialManagementService>,
    pub account_lifecycle_service: Arc<AccountLifecycleService>,
    pub account_bootstrap_service: Arc<AccountBootstrapService>,
    pub provider_metadata_service: ProviderMetadataService,
    pub import_export_service: ImportExportService,
    pub domain_service: DomainService,
    pub dns_service: DnsService,

    /// Account restoration flag
    pub restore_completed: AtomicBool,
}
```

### Adapter Implementations

The Tauri backend implements core traits:

| Trait | Adapter | Backend |
|-------|---------|---------|
| `CredentialStore` | `TauriCredentialStore` | keyring (Desktop) / Stronghold (Android) |
| `AccountRepository` | `TauriAccountRepository` | tauri-plugin-store (JSON file) |
| `ProviderRegistry` | `InMemoryProviderRegistry` | HashMap in memory |

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
        // Check cache first
        if let Some(cred) = self.cache.read().await.get(account_id) {
            return Ok(Some(cred.clone()));
        }

        // Load from system keychain
        #[cfg(not(target_os = "android"))]
        {
            let entry = Entry::new("dns-orchestrator", account_id)?;
            // ... load and deserialize
        }

        #[cfg(target_os = "android")]
        {
            // Use Stronghold
        }
    }
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
│ CredentialManagementService         │
│                                     │
│ 1. Create provider instance         │
│ 2. Validate credentials (API call)  │
│ 3. Store in CredentialStore         │
│ 4. Register in ProviderRegistry     │
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

### Why Fine-grained Services?

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
