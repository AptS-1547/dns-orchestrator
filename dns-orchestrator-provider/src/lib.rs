//! DNS Provider abstraction library for multiple cloud platforms
//!
//! This library provides a unified interface for managing DNS records across
//! different cloud providers including Cloudflare, Aliyun, DNSPod, and Huaweicloud.
//!
//! # Features
//!
//! - `cloudflare` - Enable Cloudflare provider
//! - `aliyun` - Enable Aliyun DNS provider
//! - `dnspod` - Enable Tencent Cloud DNSPod provider
//! - `huaweicloud` - Enable Huawei Cloud DNS provider
//! - `all-providers` - Enable all providers
//! - `native-tls` - Use native TLS backend (default)
//! - `rustls` - Use rustls TLS backend (recommended for Android)
//!
//! # Example
//!
//! ```rust,no_run
//! use dns_orchestrator_provider::{create_provider, DnsProvider, PaginationParams, ProviderCredentials};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let credentials = ProviderCredentials::Cloudflare {
//!         api_token: "your-token".to_string(),
//!     };
//!
//!     let provider = create_provider(credentials)?;
//!     let domains = provider.list_domains(&PaginationParams::default()).await?;
//!
//!     for domain in domains.items {
//!         println!("Domain: {}", domain.name);
//!     }
//!
//!     Ok(())
//! }
//! ```

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
