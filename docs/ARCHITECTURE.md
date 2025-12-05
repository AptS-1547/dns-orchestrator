# Architecture Documentation

This document provides an in-depth look at the architectural design of DNS Orchestrator, explaining the key components, design patterns, and technical decisions.

## Table of Contents

- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Frontend Architecture](#frontend-architecture)
- [Backend Architecture](#backend-architecture)
- [Security Architecture](#security-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)

## Overview

DNS Orchestrator is a cross-platform desktop application built with a clear separation between frontend and backend:

- **Frontend**: React-based UI with TypeScript, Tailwind CSS, and Zustand for state management
- **Backend**: Rust-based Tauri commands for business logic and DNS provider integrations
- **Communication**: Tauri IPC bridge enables type-safe communication between frontend and backend
- **Security**: System keychain integration ensures secure credential storage

### Technology Choices

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **UI Framework** | React 19 + TypeScript | Strong ecosystem, type safety, component reusability |
| **State Management** | Zustand | Lightweight, no boilerplate, simple API |
| **Styling** | Tailwind CSS 4 | Utility-first, rapid development, consistent design |
| **Desktop Framework** | Tauri 2 | Smaller bundle size than Electron, Rust security benefits |
| **Backend Language** | Rust | Memory safety, performance, async/await support |
| **HTTP Client** | Reqwest | Industry standard, async, TLS support |
| **Credential Storage** | keyring crate | Cross-platform system keychain integration |
| **Build Tool** | Vite 7 | Fast HMR, optimized production builds |

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  React Components (src/components/)                      │   │
│  │  - AccountList, DnsRecordTable, DomainList, Toolbox      │   │
│  └──────────────────┬───────────────────────────────────────┘   │
│                     │                                             │
│  ┌──────────────────▼───────────────────────────────────────┐   │
│  │  Zustand Stores (src/stores/)                            │   │
│  │  - accountStore, dnsStore, domainStore, toolboxStore     │   │
│  └──────────────────┬───────────────────────────────────────┘   │
└────────────────────┬┼───────────────────────────────────────────┘
                     ││ Tauri IPC Bridge (invoke)
┌────────────────────▼▼───────────────────────────────────────────┐
│                   TAURI COMMANDS (src-tauri/src/commands/)       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  account.rs | dns.rs | domain.rs | toolbox.rs            │   │
│  └──────────────────┬───────────────────────────────────────┘   │
│                     │                                             │
│  ┌──────────────────▼───────────────────────────────────────┐   │
│  │  AppState (src-tauri/src/lib.rs)                         │   │
│  │  - ProviderRegistry                                       │   │
│  │  - CredentialStore (Keychain)                            │   │
│  │  - Account Metadata                                       │   │
│  └─────┬──────────────────────────┬─────────────────────────┘   │
│        │                          │                               │
│  ┌─────▼──────────┐      ┌────────▼──────────────────────────┐  │
│  │  Credentials   │      │  ProviderRegistry                 │  │
│  │  (keyring)     │      │  - Dynamic Provider Management    │  │
│  └────────────────┘      └─────────┬─────────────────────────┘  │
│                                     │                             │
│  ┌──────────────────────────────────▼──────────────────────┐    │
│  │  DNS Providers (src-tauri/src/providers/)               │    │
│  │  - CloudflareProvider, AliyunProvider, DnspodProvider    │    │
│  │  - HuaweicloudProvider                                   │    │
│  │  All implement: DnsProvider trait                        │    │
│  └──────────────────────┬───────────────────────────────────┘    │
└─────────────────────────┼────────────────────────────────────────┘
                          │ HTTPS Requests
┌─────────────────────────▼────────────────────────────────────────┐
│                    EXTERNAL DNS APIS                              │
│  Cloudflare API | Aliyun DNS | DNSPod API | Huawei Cloud DNS     │
└───────────────────────────────────────────────────────────────────┘
```

## Frontend Architecture

### Component Structure

Components are organized by feature domain:

```
src/components/
├── account/          # Account management
│   ├── AccountList.tsx
│   ├── AccountForm.tsx
│   ├── ExportDialog.tsx
│   ├── ImportDialog.tsx
│   └── ProviderIcon.tsx
├── dns/              # DNS record management
│   ├── DnsRecordTable.tsx  (main list with pagination)
│   ├── DnsRecordRow.tsx
│   └── DnsRecordForm.tsx
├── domain/           # Domain management
│   ├── DomainList.tsx
│   └── DomainCard.tsx
├── toolbox/          # Network utilities
│   ├── ToolboxPage.tsx
│   ├── DnsLookup.tsx
│   ├── WhoisLookup.tsx
│   └── HistoryChips.tsx
├── settings/         # Application settings
│   └── SettingsPage.tsx
└── ui/               # Reusable components (21 components)
    ├── button.tsx
    ├── dialog.tsx
    ├── select.tsx
    └── ... (Radix UI wrappers)
```

**Design Pattern**: Feature-based organization ensures high cohesion and low coupling.

### State Management (Zustand)

Each feature domain has its own store:

```typescript
// src/stores/accountStore.ts
interface AccountStore {
  accounts: Account[];
  currentAccount: Account | null;
  setAccounts: (accounts: Account[]) => void;
  setCurrentAccount: (account: Account | null) => void;
  // ...
}

// src/stores/dnsStore.ts
interface DnsStore {
  records: DnsRecord[];
  currentPage: number;
  totalPages: number;
  searchQuery: string;
  filterType: RecordType | 'ALL';
  PAGE_SIZE: 20;
  // ...
}
```

**Benefits**:
- **Separation of Concerns**: Each store manages its own domain
- **No Prop Drilling**: Components access state directly
- **Type Safety**: Full TypeScript support
- **Minimal Boilerplate**: Simple API without actions/reducers

### Internationalization (i18n)

Translation files use a structured format:

```typescript
// src/i18n/locales/en-US.ts
export default {
  common: { /* common strings */ },
  account: { /* account-specific */ },
  dns: { /* DNS-specific */ },
  providers: {
    cloudflare: 'Cloudflare',
    aliyun: 'Alibaba Cloud DNS',
    // ...
  },
};
```

Language switching is instant and doesn't require app restart.

### Component Communication

```
User Action → Component → Zustand Store → Tauri invoke() → Backend
                   ↑                                          ↓
                   └──────────── Store Update ←──────────────┘
```

Example flow:
1. User clicks "Create DNS Record"
2. `DnsRecordForm` validates input
3. Calls `createDnsRecord()` from `dnsStore`
4. Store invokes Tauri command: `invoke('create_dns_record', { ... })`
5. Backend processes request and returns result
6. Store updates state with new record
7. UI re-renders automatically

## Backend Architecture

### Tauri Application State

```rust
// src-tauri/src/lib.rs
pub struct AppState {
    pub registry: ProviderRegistry,           // Provider instances
    pub credential_store: Arc<dyn CredentialStore>,  // System keychain
    pub accounts: RwLock<Vec<Account>>,       // Account metadata
    pub app_handle: tauri::AppHandle,         // Tauri handle
}
```

**Lifecycle**:
1. Application starts → `setup()` hook creates `AppState`
2. Loads account metadata from persistent storage
3. Restores credentials from system keychain
4. Registers providers in `ProviderRegistry`
5. State is managed globally via `app.manage(state)`

### Provider Abstraction

The `DnsProvider` trait defines a common interface for all providers:

```rust
#[async_trait]
pub trait DnsProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn validate_credentials(&self) -> Result<bool>;

    async fn list_domains(&self, params: &PaginationParams)
        -> Result<PaginatedResponse<Domain>>;

    async fn get_domain(&self, domain_id: &str) -> Result<Domain>;

    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams)
        -> Result<PaginatedResponse<DnsRecord>>;

    async fn create_record(&self, req: &CreateDnsRecordRequest)
        -> Result<DnsRecord>;

    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest)
        -> Result<DnsRecord>;

    async fn delete_record(&self, record_id: &str, domain_id: &str)
        -> Result<()>;
}
```

**Benefits**:
- **Polymorphism**: Treat all providers uniformly
- **Extensibility**: Add new providers without changing core logic
- **Testability**: Mock providers for testing
- **Type Safety**: Compile-time guarantees

### Provider Registry

Dynamic management of provider instances:

```rust
pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl ProviderRegistry {
    pub async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);
    pub async fn unregister(&self, account_id: &str);
    pub async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;
}
```

**Pattern**: Registry pattern for centralized instance management

**Key Features**:
- Thread-safe with `Arc<RwLock<>>`
- Lazy initialization (providers created on-demand)
- Automatic cleanup when accounts are deleted

### Command Layer

Tauri commands bridge frontend and backend:

```rust
// src-tauri/src/commands/dns.rs
#[tauri::command]
pub async fn list_dns_records(
    account_id: String,
    domain_id: String,
    page: u32,
    page_size: u32,
    search: Option<String>,
    record_type: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<PaginatedResponse<DnsRecord>, String> {
    // 1. Get provider from registry
    let provider = state.registry.get(&account_id).await
        .ok_or("Provider not found")?;

    // 2. Build query params
    let params = RecordQueryParams { page, page_size, search, record_type };

    // 3. Call provider method
    provider.list_records(&domain_id, &params).await
        .map_err(|e| e.to_string())
}
```

**Error Handling**: All errors are converted to strings for JSON serialization

## Security Architecture

### Credential Storage

**Design Goal**: Never store API credentials in plaintext

```rust
pub trait CredentialStore: Send + Sync {
    fn store(&self, account_id: &str, credentials: HashMap<String, String>) -> Result<()>;
    fn retrieve(&self, account_id: &str) -> Result<HashMap<String, String>>;
    fn delete(&self, account_id: &str) -> Result<()>;
}

// Platform-specific implementation
pub struct KeychainStore;

impl CredentialStore for KeychainStore {
    fn store(&self, account_id: &str, credentials: HashMap<String, String>) -> Result<()> {
        // Uses keyring crate to access system keychain
        // macOS: Keychain
        // Windows: Credential Manager
        // Linux: Secret Service (GNOME Keyring, KWallet)
    }
}
```

**Benefits**:
- OS-level security (encrypted, access-controlled)
- Integration with system authentication
- Credentials persist across app restarts
- No plaintext files

### Account Import/Export Encryption

```rust
// src-tauri/src/crypto.rs
pub fn encrypt_data(data: &str, password: &str) -> Result<String> {
    // 1. Derive key from password using Argon2
    // 2. Generate random nonce
    // 3. Encrypt with ChaCha20-Poly1305 AEAD
    // 4. Return base64-encoded: nonce || ciphertext || tag
}

pub fn decrypt_data(encrypted: &str, password: &str) -> Result<String> {
    // 1. Base64 decode
    // 2. Split nonce, ciphertext, tag
    // 3. Derive key from password
    // 4. Decrypt and verify authentication tag
}
```

**Security Properties**:
- **Authenticated Encryption**: ChaCha20-Poly1305 prevents tampering
- **Key Derivation**: Argon2 resists brute-force attacks
- **Random Nonces**: Prevents replay attacks
- **Password-Based**: User controls access

### HTTPS Communication

All DNS provider APIs are accessed over HTTPS:

```rust
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

let response = client
    .get(&url)
    .header("Authorization", format!("Bearer {}", api_token))
    .send()
    .await?;
```

**Security**: TLS 1.2+, certificate validation, secure defaults

## Performance Optimizations

### 1. Pagination

**Problem**: Loading thousands of DNS records at once causes UI lag

**Solution**: Server-side pagination with configurable page size

```typescript
// Frontend
const PAGE_SIZE = 20;

// Backend
pub struct RecordQueryParams {
    pub page: u32,
    pub page_size: u32,
    pub search: Option<String>,
    pub record_type: Option<String>,
}
```

**Benefits**:
- Reduced memory usage
- Faster initial load
- Smooth scrolling

### 2. Search Debouncing

**Problem**: Sending API requests on every keystroke is wasteful

**Solution**: Debounce search input with `use-debounce`

```typescript
const [searchQuery, setSearchQuery] = useState('');
const debouncedSearch = useDebounce(searchQuery, 300);

useEffect(() => {
  fetchRecords({ search: debouncedSearch });
}, [debouncedSearch]);
```

**Benefits**:
- Reduces API calls by ~90%
- Better user experience (no lag)
- Lower backend load

### 3. Infinite Scroll

**Problem**: Traditional pagination requires clicking "Next" repeatedly

**Solution**: IntersectionObserver-based infinite scroll

```typescript
const observer = new IntersectionObserver((entries) => {
  if (entries[0].isIntersecting && hasMore && !loading) {
    loadMore();
  }
}, { threshold: 1.0 });

observer.observe(sentinelRef.current);
```

**Benefits**:
- Natural browsing experience
- Automatic loading as user scrolls
- Memory-efficient (old pages can be garbage collected)

### 4. Rust Async Concurrency

All provider API calls use Tokio async runtime:

```rust
#[tokio::main]
async fn main() {
    // Multiple concurrent requests
    let (domains, records) = tokio::join!(
        provider.list_domains(&params),
        provider.list_records(&domain_id, &params),
    );
}
```

**Benefits**:
- Non-blocking I/O
- Efficient use of system resources
- Better responsiveness under load

## Data Flow

### Account Creation Flow

```
1. User fills AccountForm
2. Frontend validates input
3. invoke('create_account', { name, provider_type, credentials })
4. Backend:
   a. Create Provider instance
   b. validate_credentials()
   c. Store credentials in keychain
   d. Save account metadata to storage
   e. Register provider in ProviderRegistry
5. Frontend updates accountStore
6. UI shows new account
```

### DNS Record Query Flow

```
1. User selects account and domain
2. User types in search box (debounced)
3. Frontend calls dnsStore.fetchRecords()
4. invoke('list_dns_records', { account_id, domain_id, page, search, type })
5. Backend:
   a. Get provider from ProviderRegistry
   b. Build RecordQueryParams
   c. Call provider.list_records()
   d. Provider makes HTTPS request to DNS API
   e. Parse response and map to DnsRecord type
   f. Return PaginatedResponse
6. Frontend updates dnsStore.records
7. DnsRecordTable re-renders with new data
8. If user scrolls to bottom, repeat from step 3 with page+1
```

### Credential Retrieval Flow

```
App Startup:
1. Load account metadata from storage
2. For each account:
   a. Retrieve credentials from keychain
   b. Create provider instance
   c. Register in ProviderRegistry

Result: All accounts ready with credentials loaded
```

## Design Decisions

### Why Tauri over Electron?

| Criterion | Tauri | Electron |
|-----------|-------|----------|
| **Bundle Size** | ~15 MB | ~100+ MB |
| **Memory Usage** | ~50 MB | ~150+ MB |
| **Security** | Rust memory safety | V8 sandbox |
| **Startup Time** | Fast | Slower |
| **Native APIs** | Direct access | Through Node.js |

**Decision**: Tauri's smaller footprint and Rust security benefits outweigh Electron's larger ecosystem.

### Why Zustand over Redux?

| Criterion | Zustand | Redux |
|-----------|---------|-------|
| **Boilerplate** | Minimal | High (actions, reducers) |
| **Learning Curve** | Low | Steep |
| **TypeScript** | Excellent | Good |
| **Bundle Size** | 1 KB | 11 KB |
| **Middleware** | Simple hooks | Complex middleware |

**Decision**: For this app's complexity, Zustand provides sufficient power with much less code.

### Why System Keychain over Encrypted File?

**Advantages of System Keychain**:
1. OS-level security (hardware-backed on some systems)
2. No need to manage encryption keys
3. Integration with system authentication
4. Protected against unauthorized access
5. Industry best practice

**Disadvantages of Encrypted File**:
1. Key management complexity
2. Vulnerable if password is weak
3. Easier to extract and attack offline
4. Not protected by OS security features

### Why Separate Providers Instead of Generic Client?

**Benefits of Provider Abstraction**:
1. **Flexibility**: Each provider has unique quirks (pagination, rate limits, etc.)
2. **Maintainability**: Changes to one provider don't affect others
3. **Testability**: Mock individual providers easily
4. **Extensibility**: Add providers without changing core logic
5. **Type Safety**: Compile-time guarantees per provider

A generic client would force all providers into the same mold, losing these benefits.

## Future Improvements

Potential architectural enhancements:

1. **Provider Plugin System**: Load providers dynamically via WebAssembly or dynamic libraries
2. **Caching Layer**: Cache frequently-accessed domains/records in SQLite
3. **Background Sync**: Periodically check for DNS record changes
4. **Offline Mode**: Queue operations when network is unavailable
5. **Multi-Account Operations**: Bulk operations across multiple accounts
6. **GraphQL API**: Replace REST with GraphQL for more efficient querying

---

This architecture balances simplicity, security, and performance. It's designed to be maintainable by individual developers while being robust enough for production use.
