# DNS Orchestrator Provider Architecture

This document describes the architecture of the `dns-orchestrator-provider` crate, a standalone Rust library providing unified DNS provider abstraction for the DNS Orchestrator project.

## Overview

**dns-orchestrator-provider** is a type-safe, async-first Rust library that provides a unified interface for managing DNS records across multiple cloud providers:

- **Cloudflare**
- **Alibaba Cloud DNS (Aliyun)**
- **Tencent Cloud DNSPod**
- **Huawei Cloud DNS**

### Key Statistics

| Metric | Value |
|--------|-------|
| Total Lines of Code | ~5,500 |
| Source Files | 33 .rs files |
| Supported Providers | 4 |
| Supported Record Types | 8 (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA) |

## Directory Structure

```
dns-orchestrator-provider/
├── Cargo.toml                  # Dependencies and feature flags
├── rustfmt.toml                # Code formatting config
├── docs/
│   ├── ARCHITECTURE.md         # This document
│   ├── ARCHITECTURE.zh-CN.md   # Chinese version
│   ├── TESTING.md              # Integration testing guide
│   └── TESTING.zh-CN.md        # Chinese version
├── src/
│   ├── lib.rs                  # Library entry point, public API exports
│   ├── traits.rs               # DnsProvider trait definition
│   ├── types.rs                # Common type definitions
│   ├── error.rs                # Unified error types
│   ├── factory.rs              # Provider factory functions
│   ├── http_client.rs          # HTTP utility functions
│   ├── utils/
│   │   ├── mod.rs
│   │   └── datetime.rs         # DateTime serialization utilities
│   └── providers/
│       ├── mod.rs              # Provider module registration
│       ├── common.rs           # Shared utilities
│       ├── cloudflare/         # Cloudflare implementation
│       ├── aliyun/             # Alibaba Cloud implementation
│       ├── dnspod/             # Tencent Cloud implementation
│       └── huaweicloud/        # Huawei Cloud implementation
└── tests/
    ├── common/mod.rs           # Test utilities
    └── *_test.rs               # Integration tests per provider
```

## Core Architecture

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Consumer Layer                               │
│                  (Tauri Commands / Actix-Web API)                   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Factory Layer                                │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  create_provider(credentials) -> Arc<dyn DnsProvider>        │   │
│  │  get_all_provider_metadata() -> Vec<ProviderMetadata>        │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     DnsProvider Trait Layer                          │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  async trait DnsProvider: Send + Sync                        │   │
│  │    - validate_credentials()                                  │   │
│  │    - list_domains() / get_domain()                           │   │
│  │    - list_records() / create_record()                        │   │
│  │    - update_record() / delete_record()                       │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
        ┌─────────────┬───────────┼───────────┬─────────────┐
        ▼             ▼           ▼           ▼             ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ Cloudflare  │ │   Aliyun    │ │   DNSPod    │ │ Huaweicloud │
│  Provider   │ │  Provider   │ │  Provider   │ │  Provider   │
├─────────────┤ ├─────────────┤ ├─────────────┤ ├─────────────┤
│ provider.rs │ │ provider.rs │ │ provider.rs │ │ provider.rs │
│ http.rs     │ │ http.rs     │ │ http.rs     │ │ http.rs     │
│ error.rs    │ │ error.rs    │ │ error.rs    │ │ error.rs    │
│ types.rs    │ │ types.rs    │ │ types.rs    │ │ types.rs    │
│             │ │ sign.rs     │ │ sign.rs     │ │ sign.rs     │
└──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
       │               │               │               │
       ▼               ▼               ▼               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      HTTP Client Layer                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  HttpUtils::execute_request()                                │   │
│  │  HttpUtils::execute_request_with_retry()                     │   │
│  │  HttpUtils::parse_json()                                     │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       reqwest + TLS                                  │
│              (native-tls for desktop, rustls for Android)           │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Responsibility |
|-------|----------------|
| **Factory** | Create provider instances from credentials, expose metadata |
| **Trait** | Define unified async interface for all providers |
| **Provider** | Implement provider-specific logic, API calls, error mapping |
| **HTTP Client** | Handle HTTP requests, retries, JSON parsing |
| **TLS** | Secure transport (platform-specific backend selection) |

## Core Traits and Types

### DnsProvider Trait

The core trait defining the unified interface for all DNS providers:

```rust
#[async_trait]
pub trait DnsProvider: Send + Sync {
    // Metadata
    fn id(&self) -> &'static str;
    fn metadata() -> ProviderMetadata where Self: Sized;

    // Credential validation
    async fn validate_credentials(&self) -> Result<bool>;

    // Domain management
    async fn list_domains(&self, params: &PaginationParams)
        -> Result<PaginatedResponse<ProviderDomain>>;
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain>;

    // Record management
    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams)
        -> Result<PaginatedResponse<DnsRecord>>;
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;
    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest)
        -> Result<DnsRecord>;
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;

    // Batch operations (placeholder)
    async fn batch_create_records(...) -> Result<BatchCreateResult>;
    async fn batch_update_records(...) -> Result<BatchUpdateResult>;
    async fn batch_delete_records(...) -> Result<BatchDeleteResult>;
}
```

### Type-Safe DNS Records

DNS record data is represented as a tagged enum for compile-time type safety:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RecordData {
    A { address: String },
    AAAA { address: String },
    CNAME { target: String },
    MX { priority: u16, exchange: String },
    TXT { text: String },
    NS { nameserver: String },
    SRV { priority: u16, weight: u16, port: u16, target: String },
    CAA { flags: u8, tag: String, value: String },
}
```

### Provider Credentials

Type-safe credential definitions with serde tagging:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "credentials")]
pub enum ProviderCredentials {
    Cloudflare { api_token: String },
    Aliyun { access_key_id: String, access_key_secret: String },
    Dnspod { secret_id: String, secret_key: String },
    Huaweicloud { access_key_id: String, secret_access_key: String },
}
```

### Unified Error Types

All providers return standardized error types for consistent frontend handling:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    NetworkError { provider: String, detail: String },
    InvalidCredentials { provider: String, raw_message: Option<String> },
    RecordExists { provider: String, record_name: String, ... },
    RecordNotFound { provider: String, record_id: String, ... },
    InvalidParameter { provider: String, param: String, detail: String },
    UnsupportedRecordType { provider: String, record_type: String },
    QuotaExceeded { provider: String, ... },
    DomainNotFound { provider: String, domain: String, ... },
    DomainLocked { provider: String, domain: String, ... },
    PermissionDenied { provider: String, ... },
    ParseError { provider: String, detail: String },
    SerializationError { provider: String, detail: String },
    Unknown { provider: String, raw_code: Option<String>, raw_message: String },
}
```

## Provider Implementation

### Standard Module Structure

Each provider follows a consistent six-layer structure:

```
provider_name/
├── mod.rs          # Provider struct definition, Builder pattern
├── provider.rs     # DnsProvider trait implementation
├── http.rs         # HTTP request methods (API calls)
├── error.rs        # ProviderErrorMapper implementation
├── sign.rs         # Signature algorithm (cloud-specific, Cloudflare excluded)
└── types.rs        # API response type definitions
```

### Provider Comparison

| Provider | Lines | Auth Method | API Style | Max Records/Page |
|----------|-------|-------------|-----------|------------------|
| **Cloudflare** | ~900 | Bearer Token | RESTful | Domains: 50, Records: 100 |
| **Aliyun** | ~1000 | ACS3-HMAC-SHA256 | RPC (Query String) | 100 |
| **DNSPod** | ~1070 | TC3-HMAC-SHA256 | RPC (JSON Body) | 100 |
| **Huaweicloud** | ~1070 | V4 Signature | RESTful | 500 |

### Signature Algorithms

| Provider | Algorithm | Header/Location |
|----------|-----------|-----------------|
| Cloudflare | None (Bearer Token) | `Authorization: Bearer {token}` |
| Aliyun | ACS3-HMAC-SHA256 | `Authorization` header with canonical request |
| DNSPod | TC3-HMAC-SHA256 | `Authorization` header with derived key |
| Huaweicloud | AWS Signature V4 variant | `Authorization` header with canonical request |

### Error Mapping

Each provider implements the `ProviderErrorMapper` trait to convert raw API errors to standardized `ProviderError`:

```rust
pub(crate) trait ProviderErrorMapper {
    fn provider_name(&self) -> &'static str;
    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError;
}
```

Example error code mappings:

| Provider | Code | Mapped Error |
|----------|------|--------------|
| Cloudflare | 9109, 10000 | `InvalidCredentials` |
| Cloudflare | 81057 | `RecordExists` |
| Cloudflare | 81044 | `RecordNotFound` |
| Aliyun | InvalidAccessKeyId.NotFound | `InvalidCredentials` |
| DNSPod | AuthFailure | `InvalidCredentials` |
| Huaweicloud | DNS.0003 | `RecordNotFound` |

## HTTP Client Layer

### HttpUtils

Centralized HTTP utilities for all providers:

```rust
pub struct HttpUtils;

impl HttpUtils {
    /// Execute HTTP request and return (status_code, response_text)
    pub async fn execute_request(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
    ) -> Result<(u16, String), ProviderError>;

    /// Parse JSON response with error handling
    pub fn parse_json<T>(response_text: &str, provider_name: &str) -> Result<T, ProviderError>;

    /// Execute request with exponential backoff retry
    pub async fn execute_request_with_retry(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
        max_retries: u32,
    ) -> Result<(u16, String), ProviderError>;
}
```

### Retry Strategy

- **Retried**: Network errors only
- **Not retried**: Business errors (invalid credentials, record exists, etc.)
- **Backoff**: Exponential (100ms, 200ms, 400ms, ... max 10s)

### Shared HTTP Client

A global `reqwest::Client` is shared across all providers using `OnceLock`:

```rust
pub fn create_http_client() -> Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }).clone()
}
```

## Feature Flags

The crate uses Cargo feature flags for conditional compilation:

```toml
[features]
default = ["native-tls", "all-providers"]

# TLS backend (choose one)
native-tls = ["reqwest/native-tls"]    # Default for desktop
rustls = ["reqwest/rustls-tls"]        # Required for Android

# Provider selection
cloudflare = []
aliyun = []
dnspod = []
huaweicloud = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud"]
```

### Usage Examples

```bash
# Default (all providers, native-tls)
cargo build -p dns-orchestrator-provider

# Android (all providers, rustls)
cargo build -p dns-orchestrator-provider --no-default-features --features "rustls,all-providers"

# Single provider only
cargo build -p dns-orchestrator-provider --no-default-features --features "native-tls,cloudflare"
```

## Factory Pattern

### Creating Providers

```rust
use dns_orchestrator_provider::{create_provider, ProviderCredentials};

// Create from credentials enum
let credentials = ProviderCredentials::Cloudflare {
    api_token: "your-api-token".to_string(),
};
let provider = create_provider(credentials)?;

// Use the provider
let domains = provider.list_domains(&PaginationParams::default()).await?;
```

### Builder Pattern

Each provider supports optional configuration via builder:

```rust
let provider = CloudflareProvider::builder("api-token")
    .max_retries(3)
    .build();
```

### Provider Metadata

```rust
use dns_orchestrator_provider::get_all_provider_metadata;

let metadata = get_all_provider_metadata();
// Returns Vec<ProviderMetadata> with:
// - id: Provider type identifier
// - name: Display name
// - description: Provider description
// - required_fields: Credential field definitions
// - features: Supported features (e.g., CDN proxy for Cloudflare)
// - limits: Pagination limits
```

## Testing

### Integration Test Framework

Tests are organized in `tests/` with a shared test context:

```rust
pub struct TestContext {
    pub provider: Arc<dyn DnsProvider>,
    pub domain: String,
    pub domain_id: Option<String>,
}

impl TestContext {
    pub fn cloudflare() -> Option<Self>;
    pub fn aliyun() -> Option<Self>;
    pub fn dnspod() -> Option<Self>;
    pub fn huaweicloud() -> Option<Self>;
}
```

### Running Tests

```bash
# Set environment variables
export TEST_DOMAIN=example.com
export CLOUDFLARE_API_TOKEN=xxx
export ALIYUN_ACCESS_KEY_ID=xxx
export ALIYUN_ACCESS_KEY_SECRET=xxx
# ... other provider credentials

# Run all tests
cargo test -p dns-orchestrator-provider

# Run specific provider tests
cargo test -p dns-orchestrator-provider --test cloudflare_test
```

See [TESTING.md](./TESTING.md) for detailed testing documentation.

## Design Principles

### 1. Type Safety

- `RecordData` enum ensures compile-time type checking for DNS records
- `ProviderCredentials` enum prevents credential misuse across providers
- All APIs return strongly-typed `Result<T, ProviderError>`

### 2. Unified Interface

- Single `DnsProvider` trait for all providers
- Consistent pagination, search, and error handling
- Same API regardless of underlying provider differences

### 3. Error Normalization

- Raw API errors mapped to standardized `ProviderError` variants
- Frontend can handle all provider errors uniformly
- Original error messages preserved for debugging

### 4. Performance

- Shared HTTP client with connection pooling
- Configurable retry with exponential backoff
- Feature flags for minimal binary size

### 5. Platform Flexibility

- TLS backend selection for cross-platform support
- `rustls` for Android (avoids OpenSSL cross-compilation)
- `native-tls` for desktop platforms

## Future Work

The following features are currently placeholder implementations (`unimplemented!`):

- **Batch create records**: Cloudflare supports sync batch, DNSPod uses async tasks
- **Batch update records**: Huaweicloud has `BatchUpdateRecordSetWithLine` API
- **Batch delete records**: Under investigation for each provider

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `async-trait` | 0.1 | Async trait support |
| `reqwest` | 0.12 | HTTP client |
| `serde` / `serde_json` | 1.0 | Serialization |
| `tokio` | 1.0 | Async runtime (retry delays) |
| `hmac` / `sha2` | 0.12 / 0.10 | Cryptographic signing |
| `chrono` | 0.4 | DateTime handling |
| `thiserror` | 2.0 | Error definitions |
| `urlencoding` | 2.1 | URL encoding |

## License

This crate is part of the DNS Orchestrator project.
