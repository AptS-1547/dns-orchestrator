# dns-orchestrator-core Architecture

This document describes the architecture of the `dns-orchestrator-core` library, a platform-agnostic business logic layer for DNS management.

## Table of Contents

- [Overview](#overview)
- [Directory Structure](#directory-structure)
- [Core Architecture](#core-architecture)
  - [Design Philosophy](#design-philosophy)
  - [ServiceContext](#servicecontext)
  - [Core Traits](#core-traits)
- [Type System](#type-system)
  - [Account Types](#account-types)
  - [Domain Types](#domain-types)
  - [Import/Export Types](#importexport-types)
  - [Toolbox Types](#toolbox-types)
- [Services](#services)
  - [AccountBootstrapService](#accountbootstrapservice)
  - [AccountLifecycleService](#accountlifecycleservice)
  - [AccountMetadataService](#accountmetadataservice)
  - [CredentialManagementService](#credentialmanagementservice)
  - [DnsService](#dnsservice)
  - [DomainService](#domainservice)
  - [ImportExportService](#importexportservice)
  - [ProviderMetadataService](#providermetadataservice)
  - [ToolboxService](#toolboxservice)
- [Error Handling](#error-handling)
- [Crypto Module](#crypto-module)
- [Data Flow Diagrams](#data-flow-diagrams)
- [Platform Adaptation Guide](#platform-adaptation-guide)

---

## Overview

`dns-orchestrator-core` is the core business logic library for DNS Orchestrator. It provides:

- **Platform-agnostic design**: Works with both Tauri (desktop/mobile) and Actix-Web (server) backends
- **Trait-based storage abstraction**: Storage implementation is injected at runtime
- **Account and credential management**: Secure handling of DNS provider accounts
- **Multi-provider support**: Unified interface for Cloudflare, Aliyun, DNSPod, and Huaweicloud

### Relationship with dns-orchestrator-provider

```
┌─────────────────────────────────────┐
│         Platform Layer              │
│   (Tauri / Actix-Web Backend)       │
├─────────────────────────────────────┤
│       dns-orchestrator-core         │  ← This library
│   (Business Logic & Services)       │
├─────────────────────────────────────┤
│     dns-orchestrator-provider       │
│  (DNS Provider Implementations)     │
└─────────────────────────────────────┘
```

- **dns-orchestrator-provider**: Defines `DnsProvider` trait and implements DNS API calls
- **dns-orchestrator-core**: Orchestrates business logic, manages accounts/credentials, and calls Provider APIs

---

## Directory Structure

```
src/
├── lib.rs                    # Library entry, re-exports public API
├── error.rs                  # Unified error types (CoreError, CoreResult)
│
├── types/                    # Data type definitions
│   ├── mod.rs
│   ├── account.rs           # Account, AccountStatus, CreateAccountRequest
│   ├── domain.rs            # AppDomain (extends ProviderDomain with account_id)
│   ├── export.rs            # ExportFile, ImportPreview, ExportedAccount
│   ├── response.rs          # API response wrappers
│   └── toolbox.rs           # WhoisResult, DnsLookupResult, IpLookupResult, SslCheckResult
│
├── traits/                   # Storage abstraction traits
│   ├── mod.rs
│   ├── account_repository.rs    # AccountRepository trait
│   ├── credential_store.rs      # CredentialStore trait
│   └── provider_registry.rs     # ProviderRegistry trait + InMemoryProviderRegistry
│
├── services/                 # Business logic services
│   ├── mod.rs                   # ServiceContext definition
│   ├── account_bootstrap_service.rs      # Startup account restoration
│   ├── account_lifecycle_service.rs      # Account CRUD operations
│   ├── account_metadata_service.rs       # Account queries
│   ├── credential_management_service.rs  # Credential validation and management
│   ├── domain_service.rs                 # Domain listing
│   ├── dns_service.rs                    # DNS record CRUD
│   ├── import_export_service.rs          # Account import/export with encryption
│   ├── provider_metadata_service.rs      # Provider metadata queries
│   └── toolbox/                          # Utility services
│       ├── mod.rs
│       ├── dns.rs                        # DNS lookup
│       ├── ip.rs                         # IP geolocation
│       ├── ssl.rs                        # SSL certificate check
│       ├── whois.rs                      # WHOIS query
│       └── whois_servers.json            # WHOIS server configuration
│
├── crypto/                   # Encryption module
│   ├── mod.rs               # AES-256-GCM encryption/decryption
│   └── versions.rs          # Encryption algorithm versioning
│
└── utils/                    # Utility functions
    ├── mod.rs
    └── datetime.rs          # DateTime serialization helpers
```

---

## Core Architecture

### Design Philosophy

The library follows **Dependency Injection** pattern to achieve platform independence:

1. **Define abstract traits** for storage operations
2. **Platform layer injects concrete implementations** at startup
3. **Services operate through trait interfaces**, unaware of underlying storage

This allows the same business logic to run on:
- **Tauri Desktop**: Uses system keychain (via `keyring` crate) and local store (via `tauri-plugin-store`)
- **Tauri Android**: Uses Stronghold for secure storage
- **Actix-Web**: Uses database (SeaORM) with AES encryption

### ServiceContext

`ServiceContext` is the dependency injection container that holds all storage implementations:

```rust
// src/services/mod.rs

pub struct ServiceContext {
    pub credential_store: Arc<dyn CredentialStore>,
    pub account_repository: Arc<dyn AccountRepository>,
    pub provider_registry: Arc<dyn ProviderRegistry>,
}

impl ServiceContext {
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
        provider_registry: Arc<dyn ProviderRegistry>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
            provider_registry,
        }
    }

    /// Get a provider instance by account ID
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// Mark an account as invalid (e.g., credentials expired)
    pub async fn mark_account_invalid(&self, account_id: &str, error_msg: &str) {
        let _ = self.account_repository
            .update_status(account_id, AccountStatus::Error, Some(error_msg.to_string()))
            .await;
    }
}
```

**Usage Pattern**:

```rust
// Platform layer creates ServiceContext with concrete implementations
let ctx = Arc::new(ServiceContext::new(
    Arc::new(KeychainStore::new()),           // Platform-specific
    Arc::new(TauriAccountRepository::new()),  // Platform-specific
    Arc::new(InMemoryProviderRegistry::new()), // Shared implementation
));

// Services receive ServiceContext
let lifecycle_service = AccountLifecycleService::new(ctx.clone());
let dns_service = DnsService::new(ctx.clone());
```

### Core Traits

#### 1. AccountRepository

Abstracts account metadata persistence (CRUD operations).

```rust
// src/traits/account_repository.rs

#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Get all accounts
    async fn find_all(&self) -> CoreResult<Vec<Account>>;

    /// Get account by ID
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;

    /// Save an account (insert or update)
    async fn save(&self, account: &Account) -> CoreResult<()>;

    /// Delete an account by ID
    async fn delete(&self, id: &str) -> CoreResult<()>;

    /// Batch save accounts
    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()>;

    /// Update account status (e.g., mark as Error)
    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>
    ) -> CoreResult<()>;
}
```

**Platform Implementations**:
- `TauriAccountRepository`: Uses `tauri-plugin-store` for JSON file storage
- `DatabaseAccountRepository`: Uses SeaORM for database persistence

#### 2. CredentialStore

Abstracts secure credential storage.

```rust
// src/traits/credential_store.rs

/// Map of account_id -> credential key-value pairs
pub type CredentialsMap = HashMap<String, HashMap<String, String>>;

#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Load all credentials (used at startup for batch restoration)
    async fn load_all(&self) -> CoreResult<CredentialsMap>;

    /// Save credentials for an account
    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()>;

    /// Load credentials for an account
    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>>;

    /// Delete credentials for an account
    async fn delete(&self, account_id: &str) -> CoreResult<()>;

    /// Check if credentials exist for an account
    async fn exists(&self, account_id: &str) -> CoreResult<bool>;
}
```

**Platform Implementations**:
- `KeychainStore`: Uses system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- `StrongholdStore`: Uses Tauri Stronghold plugin for Android
- `DatabaseCredentialStore`: Uses SeaORM + AES encryption for web backend

#### 3. ProviderRegistry

Manages runtime `DnsProvider` instances.

```rust
// src/traits/provider_registry.rs

#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    /// Register a provider instance for an account
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);

    /// Unregister a provider instance
    async fn unregister(&self, account_id: &str);

    /// Get a provider instance by account ID
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;

    /// List all registered account IDs
    async fn list_account_ids(&self) -> Vec<String>;
}
```

**Default Implementation** (`InMemoryProviderRegistry`):

```rust
pub struct InMemoryProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn DnsProvider>>>,
}

impl InMemoryProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }
}
```

This in-memory implementation is used by all platforms, as provider instances don't need persistence.

---

## Type System

### Account Types

```rust
// src/types/account.rs

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,                    // UUID
    pub name: String,                  // User-defined name
    pub provider: ProviderType,        // DNS provider type
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: Option<AccountStatus>, // Active or Error
    pub error: Option<String>,         // Error message when status is Error
}

/// Account status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,  // Account is working normally
    Error,   // Credentials invalid or other errors
}

/// Request to create a new account
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider: ProviderType,
    pub credentials: HashMap<String, String>,
}

/// Request to update an account
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: String,
    pub name: Option<String>,                          // Optional: new name
    pub credentials: Option<HashMap<String, String>>,  // Optional: new credentials
}
```

### Domain Types

```rust
// src/types/domain.rs

/// Application-layer domain (extends ProviderDomain with account_id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDomain {
    pub id: String,
    pub name: String,
    pub account_id: String,       // Added by core layer
    pub provider: ProviderType,
    pub status: DomainStatus,
    pub record_count: Option<u32>,
}

impl AppDomain {
    /// Convert from provider-layer domain
    pub fn from_provider(provider_domain: ProviderDomain, account_id: String) -> Self {
        Self {
            id: provider_domain.id,
            name: provider_domain.name,
            account_id,
            provider: provider_domain.provider,
            status: provider_domain.status,
            record_count: provider_domain.record_count,
        }
    }
}
```

### Import/Export Types

```rust
// src/types/export.rs

/// Export file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub header: ExportFileHeader,
    pub data: serde_json::Value,  // Encrypted: Base64 ciphertext; Unencrypted: JSON array
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFileHeader {
    pub version: u32,              // File format version
    pub encrypted: bool,           // Whether data is encrypted
    pub salt: Option<String>,      // Base64 salt (if encrypted)
    pub nonce: Option<String>,     // Base64 nonce/IV (if encrypted)
    pub exported_at: String,       // ISO 8601 timestamp
    pub app_version: String,       // Application version
}

/// Exported account data (includes credentials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedAccount {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub credentials: HashMap<String, String>,
}

/// Import preview result
#[derive(Debug, Clone, Serialize)]
pub struct ImportPreview {
    pub encrypted: bool,
    pub account_count: usize,
    pub accounts: Option<Vec<ImportPreviewAccount>>,  // Available if not encrypted or decrypted
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportPreviewAccount {
    pub name: String,
    pub provider: ProviderType,
    pub has_conflict: bool,  // Name conflicts with existing account
}

/// Import result
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub failures: Vec<ImportFailure>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportFailure {
    pub name: String,
    pub reason: String,
}
```

### Toolbox Types

```rust
// src/types/toolbox.rs

/// WHOIS query result
#[derive(Debug, Clone, Serialize)]
pub struct WhoisResult {
    pub domain: String,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiration_date: Option<String>,
    pub updated_date: Option<String>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub raw: String,  // Raw WHOIS response
}

/// DNS lookup result
#[derive(Debug, Clone, Serialize)]
pub struct DnsLookupResult {
    pub nameserver: String,
    pub records: Vec<DnsLookupRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsLookupRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,  // For MX/SRV records
}

/// IP geolocation result
#[derive(Debug, Clone, Serialize)]
pub struct IpLookupResult {
    pub query: String,           // Original input
    pub is_domain: bool,         // Whether input was a domain
    pub results: Vec<IpGeoInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IpGeoInfo {
    pub ip: String,
    pub ip_version: String,      // "IPv4" or "IPv6"
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    pub as_name: Option<String>,
}

/// SSL certificate check result
#[derive(Debug, Clone, Serialize)]
pub struct SslCheckResult {
    pub domain: String,
    pub port: u16,
    pub connection_status: String,  // "https" | "http" | "failed"
    pub cert_info: Option<SslCertInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SslCertInfo {
    pub domain: String,
    pub issuer: String,
    pub subject: String,
    pub valid_from: String,
    pub valid_to: String,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub is_valid: bool,
    pub san: Vec<String>,
    pub serial_number: String,
    pub signature_algorithm: String,
    pub certificate_chain: Vec<CertChainItem>,
}
```

---

## Services

All services follow the same pattern:

```rust
pub struct XxxService {
    ctx: Arc<ServiceContext>,
}

impl XxxService {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    // Service methods...
}
```

### AccountBootstrapService

Restores all accounts at application startup.

```rust
// src/services/account_bootstrap_service.rs

pub struct RestoreResult {
    pub success_count: usize,
    pub failed_accounts: Vec<String>,  // Account IDs that failed to restore
}

impl AccountBootstrapService {
    /// Restore all accounts from storage
    ///
    /// This method:
    /// 1. Loads all account metadata from AccountRepository
    /// 2. Loads all credentials from CredentialStore
    /// 3. Rebuilds DnsProvider instances
    /// 4. Registers providers to ProviderRegistry
    /// 5. Marks failed accounts as Error status
    pub async fn restore_accounts(&self) -> CoreResult<RestoreResult>;
}
```

**Typical Usage** (at application startup):

```rust
let bootstrap = AccountBootstrapService::new(ctx.clone());
let result = bootstrap.restore_accounts().await?;
println!("Restored {} accounts, {} failed", result.success_count, result.failed_accounts.len());
```

### AccountLifecycleService

Manages account CRUD operations.

```rust
// src/services/account_lifecycle_service.rs

impl AccountLifecycleService {
    /// Create a new account
    ///
    /// Steps:
    /// 1. Validate credentials format
    /// 2. Create provider and verify credentials with API
    /// 3. Save credentials to CredentialStore
    /// 4. Register provider to ProviderRegistry
    /// 5. Save account metadata to AccountRepository
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account>;

    /// Update an existing account
    ///
    /// Supports updating name and/or credentials.
    /// If credentials are updated, they are re-validated.
    pub async fn update_account(&self, request: UpdateAccountRequest) -> CoreResult<Account>;

    /// Delete an account
    ///
    /// Order of operations (prevents ghost accounts):
    /// 1. Delete metadata from AccountRepository
    /// 2. Unregister from ProviderRegistry
    /// 3. Delete credentials from CredentialStore
    pub async fn delete_account(&self, account_id: &str) -> CoreResult<()>;

    /// Batch delete accounts
    pub async fn batch_delete_accounts(&self, account_ids: &[String]) -> CoreResult<BatchDeleteAccountResult>;
}
```

### AccountMetadataService

Queries account metadata.

```rust
// src/services/account_metadata_service.rs

impl AccountMetadataService {
    /// Get all accounts
    pub async fn list_accounts(&self) -> CoreResult<Vec<Account>>;

    /// Get account by ID
    pub async fn get_account(&self, account_id: &str) -> CoreResult<Account>;
}
```

### CredentialManagementService

Manages credential validation and storage.

```rust
// src/services/credential_management_service.rs

impl CredentialManagementService {
    /// Validate credentials and create a provider instance
    ///
    /// Returns the created provider on success.
    /// Returns CredentialValidationError on validation failure.
    pub async fn validate_and_create_provider(
        &self,
        provider_type: &ProviderType,
        credentials: &HashMap<String, String>,
    ) -> CoreResult<Arc<dyn DnsProvider>>;

    /// Register a provider instance for an account
    pub async fn register_provider(&self, account_id: &str, provider: Arc<dyn DnsProvider>);

    /// Unregister a provider instance
    pub async fn unregister_provider(&self, account_id: &str);

    /// Save credentials to storage
    pub async fn save_credentials(
        &self,
        account_id: &str,
        credentials: &HashMap<String, String>
    ) -> CoreResult<()>;

    /// Delete credentials from storage
    pub async fn delete_credentials(&self, account_id: &str) -> CoreResult<()>;
}
```

### DnsService

Manages DNS records.

```rust
// src/services/dns_service.rs

impl DnsService {
    /// List DNS records for a domain
    pub async fn list_records(
        &self,
        account_id: &str,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> CoreResult<PaginatedResponse<DnsRecord>>;

    /// Create a DNS record
    pub async fn create_record(
        &self,
        account_id: &str,
        request: &CreateDnsRecordRequest,
    ) -> CoreResult<DnsRecord>;

    /// Update a DNS record
    pub async fn update_record(
        &self,
        account_id: &str,
        record_id: &str,
        request: &UpdateDnsRecordRequest,
    ) -> CoreResult<DnsRecord>;

    /// Delete a DNS record
    pub async fn delete_record(
        &self,
        account_id: &str,
        record_id: &str,
        domain_id: &str,
    ) -> CoreResult<()>;

    /// Batch delete DNS records
    pub async fn batch_delete_records(
        &self,
        account_id: &str,
        domain_id: &str,
        record_ids: &[String],
    ) -> CoreResult<BatchDeleteResult>;
}
```

**Credential Invalidation Detection**:

DnsService automatically detects credential invalidation:

```rust
async fn list_records(...) -> CoreResult<...> {
    let provider = self.ctx.get_provider(account_id).await?;

    match provider.list_records(domain_id, params).await {
        Ok(response) => Ok(response),
        Err(ProviderError::InvalidCredentials { raw_message, .. }) => {
            // Mark account as invalid with original error message
            let error_message = raw_message.unwrap_or_else(|| "Invalid credentials".to_string());
            self.ctx.mark_account_invalid(account_id, &error_message).await;
            Err(CoreError::InvalidCredentials(account_id.to_string()))
        }
        Err(e) => Err(CoreError::Provider(e)),
    }
}
```

### DomainService

Manages domains.

```rust
// src/services/domain_service.rs

impl DomainService {
    /// List domains for an account
    pub async fn list_domains(
        &self,
        account_id: &str,
        params: &PaginationParams,
    ) -> CoreResult<PaginatedResponse<AppDomain>>;

    /// Get a single domain
    pub async fn get_domain(
        &self,
        account_id: &str,
        domain_id: &str,
    ) -> CoreResult<AppDomain>;
}
```

### ImportExportService

Handles account import and export with optional encryption.

```rust
// src/services/import_export_service.rs

impl ImportExportService {
    /// Export accounts to JSON (optionally encrypted)
    pub async fn export_accounts(&self, request: &ExportAccountsRequest) -> CoreResult<ExportResponse>;

    /// Preview an import file (parse without importing)
    pub async fn preview_import(
        &self,
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<ImportPreview>;

    /// Import accounts from file
    pub async fn import_accounts(
        &self,
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<ImportResult>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportAccountsRequest {
    pub account_ids: Vec<String>,
    pub encrypt: bool,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResponse {
    pub content: String,      // JSON string (may contain encrypted data)
    pub filename: String,     // Suggested filename
}
```

### ProviderMetadataService

Retrieves DNS provider metadata.

```rust
// src/services/provider_metadata_service.rs

impl ProviderMetadataService {
    /// Get metadata for all available providers
    pub fn get_all_providers(&self) -> Vec<ProviderMetadata>;

    /// Get metadata for a specific provider
    pub fn get_provider(&self, provider_type: &ProviderType) -> Option<ProviderMetadata>;
}
```

### ToolboxService

Stateless utility services.

```rust
// src/services/toolbox/mod.rs

impl ToolboxService {
    /// WHOIS query
    pub async fn whois(&self, domain: &str) -> CoreResult<WhoisResult>;

    /// DNS lookup
    pub async fn dns_lookup(
        &self,
        domain: &str,
        record_type: &str,
        nameserver: Option<&str>,
    ) -> CoreResult<DnsLookupResult>;

    /// IP geolocation lookup
    pub async fn ip_lookup(&self, query: &str) -> CoreResult<IpLookupResult>;

    /// SSL certificate check
    pub async fn ssl_check(&self, domain: &str, port: Option<u16>) -> CoreResult<SslCheckResult>;
}
```

---

## Error Handling

### CoreError

The unified error type for the core layer:

```rust
// src/error.rs

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum CoreError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Credential error: {0}")]
    CredentialError(String),

    #[error("Credential validation failed")]
    CredentialValidation(CredentialValidationError),

    #[error("API error from {provider}: {message}")]
    ApiError { provider: String, message: String },

    #[error("Invalid credentials for account: {0}")]
    InvalidCredentials(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Import/export error: {0}")]
    ImportExportError(String),

    #[error("No accounts selected")]
    NoAccountsSelected,

    #[error("Unsupported file version")]
    UnsupportedFileVersion,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
}

pub type CoreResult<T> = std::result::Result<T, CoreError>;
```

### Error Propagation

```
Provider API Error
       ↓
ProviderError (dns-orchestrator-provider)
       ↓
CoreError::Provider (auto-converted via From trait)
       ↓
Platform layer (Tauri/Actix-Web)
       ↓
Frontend (JSON with error code and details)
```

### Serialization Format

Errors are serialized as tagged unions for easy frontend handling:

```json
{
  "code": "CredentialValidation",
  "details": {
    "type": "missingField",
    "provider": "cloudflare",
    "field": "apiToken",
    "label": "API Token"
  }
}
```

---

## Crypto Module

The crypto module provides AES-256-GCM encryption for account export/import.

### Algorithm

- **Encryption**: AES-256-GCM
- **Key Derivation**: PBKDF2-HMAC-SHA256
- **Salt**: 16 bytes, randomly generated per encryption
- **Nonce/IV**: 12 bytes, randomly generated per encryption

### Version Management

```rust
// src/crypto/versions.rs

pub const CURRENT_FILE_VERSION: u32 = 2;

/// Get PBKDF2 iterations for a specific version
pub fn get_pbkdf2_iterations(version: u32) -> u32 {
    match version {
        1 => 100_000,   // Legacy
        2 => 600_000,   // OWASP 2023 recommendation
        _ => 600_000,   // Default to current
    }
}

pub fn get_current_iterations() -> u32 {
    get_pbkdf2_iterations(CURRENT_FILE_VERSION)
}
```

### API

```rust
// src/crypto/mod.rs

/// Encrypt plaintext with password
/// Returns (salt_base64, nonce_base64, ciphertext_base64)
pub fn encrypt(plaintext: &[u8], password: &str) -> CoreResult<(String, String, String)>;

/// Decrypt ciphertext with password
pub fn decrypt(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
) -> CoreResult<Vec<u8>>;

/// Decrypt with specific iteration count (for backward compatibility)
pub fn decrypt_with_iterations(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
    iterations: u32,
) -> CoreResult<Vec<u8>>;
```

### Usage Example

```rust
use dns_orchestrator_core::crypto::{encrypt, decrypt};

// Encrypt
let plaintext = b"sensitive data";
let (salt, nonce, ciphertext) = encrypt(plaintext, "password123")?;

// Decrypt
let decrypted = decrypt(&ciphertext, "password123", &salt, &nonce)?;
assert_eq!(plaintext, decrypted.as_slice());
```

---

## Data Flow Diagrams

### Create Account Flow

```
┌─────────────────┐
│  CreateAccount  │
│    Request      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  CredentialManagementService        │
│  validate_and_create_provider()     │
│                                     │
│  1. ProviderCredentials::from_map() │
│  2. create_provider()               │
│  3. provider.validate_credentials() │
└────────┬────────────────────────────┘
         │ Provider instance
         ▼
┌─────────────────────────────────────┐
│  CredentialStore.save()             │
│  (Save credentials to secure store) │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  ProviderRegistry.register()        │
│  (Register provider in memory)      │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  AccountRepository.save()           │
│  (Save account metadata)            │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────┐
│  Account        │
│  (Response)     │
└─────────────────┘
```

### Bootstrap (Startup Restore) Flow

```
┌─────────────────────────────────────┐
│  Application Startup                │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  AccountRepository.find_all()       │
│  (Load all account metadata)        │
└────────┬────────────────────────────┘
         │ Vec<Account>
         ▼
┌─────────────────────────────────────┐
│  CredentialStore.load_all()         │
│  (Batch load all credentials)       │
└────────┬────────────────────────────┘
         │ CredentialsMap
         ▼
┌─────────────────────────────────────┐
│  For each account:                  │
│  1. create_provider(credentials)    │
│  2. ProviderRegistry.register()     │
│                                     │
│  On failure:                        │
│  - Mark account as Error status     │
│  - Continue with next account       │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  RestoreResult                      │
│  { success_count, failed_accounts } │
└─────────────────────────────────────┘
```

### DNS Operation Flow

```
┌─────────────────┐
│  list_records   │
│  (account_id,   │
│   domain_id)    │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  ServiceContext.get_provider()      │
│  (Get from ProviderRegistry)        │
└────────┬────────────────────────────┘
         │ Arc<dyn DnsProvider>
         ▼
┌─────────────────────────────────────┐
│  provider.list_records()            │
│  (Call DNS provider API)            │
└────────┬────────────────────────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐  ┌────────────────────────┐
│ Success│  │ InvalidCredentials     │
│        │  │                        │
│ Return │  │ mark_account_invalid() │
│ records│  │ Return error           │
└────────┘  └────────────────────────┘
```

---

## Platform Adaptation Guide

To use `dns-orchestrator-core` in a new platform, you need to implement three traits.

### 1. Implement AccountRepository

```rust
use dns_orchestrator_core::{AccountRepository, Account, AccountStatus, CoreResult};

pub struct MyAccountRepository {
    // Your storage backend (database, file, etc.)
}

#[async_trait]
impl AccountRepository for MyAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        // Load all accounts from your storage
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        // Load account by ID
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        // Save account (insert or update)
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        // Delete account
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        // Batch save
    }

    async fn update_status(&self, id: &str, status: AccountStatus, error: Option<String>) -> CoreResult<()> {
        // Update account status
    }
}
```

### 2. Implement CredentialStore

```rust
use dns_orchestrator_core::{CredentialStore, CredentialsMap, CoreResult};

pub struct MyCredentialStore {
    // Your secure storage (keychain, database, etc.)
}

#[async_trait]
impl CredentialStore for MyCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        // Batch load all credentials
    }

    async fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> CoreResult<()> {
        // Save credentials securely
    }

    async fn load(&self, account_id: &str) -> CoreResult<HashMap<String, String>> {
        // Load credentials
    }

    async fn delete(&self, account_id: &str) -> CoreResult<()> {
        // Delete credentials
    }

    async fn exists(&self, account_id: &str) -> CoreResult<bool> {
        // Check if credentials exist
    }
}
```

### 3. Wire Everything Together

```rust
use dns_orchestrator_core::{
    ServiceContext,
    InMemoryProviderRegistry,
    AccountBootstrapService,
    AccountLifecycleService,
    DnsService,
    // ... other services
};

// Create ServiceContext with your implementations
let ctx = Arc::new(ServiceContext::new(
    Arc::new(MyCredentialStore::new()),
    Arc::new(MyAccountRepository::new()),
    Arc::new(InMemoryProviderRegistry::new()),  // Use built-in implementation
));

// Bootstrap at startup
let bootstrap = AccountBootstrapService::new(ctx.clone());
let result = bootstrap.restore_accounts().await?;

// Use services
let lifecycle = AccountLifecycleService::new(ctx.clone());
let dns = DnsService::new(ctx.clone());

// Create account
let account = lifecycle.create_account(CreateAccountRequest {
    name: "My DNS".to_string(),
    provider: ProviderType::Cloudflare,
    credentials: [("apiToken".to_string(), "token".to_string())].into(),
}).await?;

// List records
let records = dns.list_records(&account.id, "domain_id", &params).await?;
```

### Example: Tauri Implementation

```rust
// Tauri backend uses:
// - KeychainStore for CredentialStore (system keychain)
// - TauriAccountRepository for AccountRepository (tauri-plugin-store)
// - InMemoryProviderRegistry for ProviderRegistry

let credential_store = Arc::new(KeychainStore::new(app_handle.clone()));
let account_repository = Arc::new(TauriAccountRepository::new(app_handle.clone()));
let provider_registry = Arc::new(InMemoryProviderRegistry::new());

let ctx = Arc::new(ServiceContext::new(
    credential_store,
    account_repository,
    provider_registry,
));
```

### Example: Actix-Web Implementation

```rust
// Actix-Web backend uses:
// - DatabaseCredentialStore (SeaORM + AES encryption)
// - DatabaseAccountRepository (SeaORM)
// - InMemoryProviderRegistry

let credential_store = Arc::new(DatabaseCredentialStore::new(db_pool.clone(), encryption_key));
let account_repository = Arc::new(DatabaseAccountRepository::new(db_pool.clone()));
let provider_registry = Arc::new(InMemoryProviderRegistry::new());

let ctx = Arc::new(ServiceContext::new(
    credential_store,
    account_repository,
    provider_registry,
));
```

---

## Re-exported Types from Provider Library

For convenience, the core library re-exports commonly used types from `dns-orchestrator-provider`:

```rust
// From dns-orchestrator-provider
pub use dns_orchestrator_provider::{
    // Provider trait
    DnsProvider,

    // Types
    DnsRecord,
    DnsRecordType,
    RecordData,
    ProviderDomain,
    DomainStatus,
    ProviderType,
    ProviderMetadata,
    ProviderCredentials,
    ProviderCredentialField,
    FieldType,
    ProviderFeatures,
    ProviderLimits,

    // Request/Response
    CreateDnsRecordRequest,
    UpdateDnsRecordRequest,
    PaginationParams,
    RecordQueryParams,
    PaginatedResponse,

    // Batch operations
    BatchCreateResult,
    BatchUpdateResult,
    BatchDeleteResult,
    BatchCreateFailure,
    BatchUpdateFailure,
    BatchDeleteFailure,
    BatchUpdateItem,

    // Errors
    ProviderError,
    CredentialValidationError,

    // Factory
    create_provider,
    get_all_provider_metadata,
};
```

---

## Feature Flags

```toml
# Cargo.toml

[features]
default = ["native-tls"]
native-tls = ["dns-orchestrator-provider/native-tls"]  # Desktop platforms
rustls = ["dns-orchestrator-provider/rustls"]          # Android (avoids OpenSSL)
```

Select the appropriate TLS backend based on your target platform:

- **Desktop (macOS, Windows, Linux)**: Use `native-tls` (default)
- **Android**: Use `rustls` to avoid OpenSSL cross-compilation issues

```toml
# For Android
dns-orchestrator-core = { path = "../dns-orchestrator-core", default-features = false, features = ["rustls"] }
```
