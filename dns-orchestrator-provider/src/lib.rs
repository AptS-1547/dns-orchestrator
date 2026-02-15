//! # dns-orchestrator-provider
//!
//! A unified DNS provider abstraction library for managing DNS records across
//! multiple cloud platforms.
//!
//! ## Supported Providers
//!
//! | Provider | Feature Flag | Auth Method |
//! |----------|-------------|-------------|
//! | [Cloudflare](https://www.cloudflare.com/) | `cloudflare` | Bearer Token |
//! | [Aliyun DNS](https://www.aliyun.com/product/dns) | `aliyun` | HMAC-SHA256 (V3) |
//! | [DNSPod (Tencent Cloud)](https://www.dnspod.cn/) | `dnspod` | TC3-HMAC-SHA256 |
//! | [Huawei Cloud DNS](https://www.huaweicloud.com/product/dns.html) | `huaweicloud` | AK/SK Signing |
//!
//! ## Feature Flags
//!
//! ### Provider Selection
//!
//! - **`all-providers`** *(default)* — Enable all providers listed above.
//! - **`cloudflare`** — Enable only the Cloudflare provider.
//! - **`aliyun`** — Enable only the Aliyun DNS provider.
//! - **`dnspod`** — Enable only the Tencent Cloud `DNSPod` provider.
//! - **`huaweicloud`** — Enable only the Huawei Cloud DNS provider.
//!
//! ### TLS Backend
//!
//! - **`native-tls`** — Use the platform's native TLS implementation.
//! - **`rustls`** *(default)* — Use rustls, a pure-Rust TLS implementation.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! dns-orchestrator-provider = { version = "0.1", features = ["all-providers"] }
//! ```
//!
//! Or enable only the providers you need:
//!
//! ```toml
//! [dependencies]
//! dns-orchestrator-provider = { version = "0.1", default-features = false, features = ["cloudflare", "rustls"] }
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dns_orchestrator_provider::{
//!     create_provider, DnsProvider, PaginationParams, ProviderCredentials,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Create a provider from credentials
//!     let credentials = ProviderCredentials::Cloudflare {
//!         api_token: "your-token".to_string(),
//!     };
//!     let provider = create_provider(credentials)?;
//!
//!     // 2. Validate credentials against the remote API
//!     provider.validate_credentials().await?;
//!
//!     // 3. List domains
//!     let domains = provider.list_domains(&PaginationParams::default()).await?;
//!     for domain in &domains.items {
//!         println!("{} ({:?})", domain.name, domain.status);
//!     }
//!
//!     // 4. List DNS records for the first domain
//!     let records = provider
//!         .list_records(&domains.items[0].id, &Default::default())
//!         .await?;
//!     for record in &records.items {
//!         println!(
//!             "{} {:?} -> {}",
//!             record.name,
//!             record.data.record_type(),
//!             record.data.display_value()
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Creating Records
//!
//! ```rust,no_run
//! # use dns_orchestrator_provider::*;
//! # async fn example(provider: std::sync::Arc<dyn DnsProvider>) -> Result<()> {
//! let request = CreateDnsRecordRequest {
//!     domain_id: "example.com".to_string(),
//!     name: "www".to_string(),
//!     ttl: 600,
//!     data: RecordData::A { address: "1.2.3.4".to_string() },
//!     proxied: None,
//! };
//! let record = provider.create_record(&request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All provider operations return [`Result<T, ProviderError>`](ProviderError).
//! The error enum provides structured variants for common failure modes:
//!
//! - [`ProviderError::InvalidCredentials`] — authentication failed
//! - [`ProviderError::RecordNotFound`] — DNS record not found
//! - [`ProviderError::RateLimited`] — API rate limit exceeded (retryable)
//! - [`ProviderError::NetworkError`] — network connectivity issue (retryable)
//!
//! Transient errors (`NetworkError`, `Timeout`, `RateLimited`) are automatically
//! retried with exponential backoff. See [`ProviderError`] for the full list.

mod error;
mod factory;
mod http_client;
mod providers;
mod traits;
mod types;
mod utils;

// Re-export error types
pub use error::{ProviderError, Result};

// Re-export factory functions
pub use factory::{create_provider, get_all_provider_metadata};

// Re-export core trait only (internal traits are not exported)
pub use traits::DnsProvider;

// Re-export types
pub use types::{
    BatchCreateFailure, BatchCreateResult, BatchDeleteFailure, BatchDeleteResult,
    BatchUpdateFailure, BatchUpdateItem, BatchUpdateResult, CreateDnsRecordRequest,
    CredentialValidationError, DnsRecord, DnsRecordType, DomainStatus, FieldType,
    PaginatedResponse, PaginationParams, ProviderCredentialField, ProviderCredentials,
    ProviderDomain, ProviderFeatures, ProviderLimits, ProviderMetadata, ProviderType, RecordData,
    RecordQueryParams, UpdateDnsRecordRequest,
};

// Re-export utils module
pub use utils::datetime;

// Re-export concrete providers (behind feature flags)
#[cfg(feature = "cloudflare")]
pub use providers::CloudflareProvider;

#[cfg(feature = "aliyun")]
pub use providers::AliyunProvider;

#[cfg(feature = "dnspod")]
pub use providers::DnspodProvider;

#[cfg(feature = "huaweicloud")]
pub use providers::HuaweicloudProvider;
