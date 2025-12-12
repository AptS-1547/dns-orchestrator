//! DNS Orchestrator Core Library
//!
//! 提供 DNS 管理应用的核心业务逻辑，包括：
//! - 账户管理 (Account Service)
//! - 域名管理 (Domain Service)
//! - DNS 记录管理 (DNS Service)
//! - 工具箱 (Toolbox Service)
//!
//! 此库设计为平台无关，通过 trait 抽象存储层，
//! 支持 Tauri (Desktop/Android) 和 Actix-Web 后端。

pub mod crypto;
pub mod error;
pub mod services;
pub mod traits;
pub mod types;

// Re-export 常用类型
pub use error::{CoreError, CoreResult};
pub use services::ServiceContext;
pub use traits::{AccountRepository, CredentialStore, ProviderRegistry};

// Re-export provider 库的公共类型
pub use dns_orchestrator_provider::{
    CreateDnsRecordRequest, DnsProvider, DnsRecord, DnsRecordType, Domain as LibDomain,
    DomainStatus, PaginatedResponse, PaginationParams, ProviderCredentials, ProviderError,
    ProviderMetadata, ProviderType, RecordQueryParams, UpdateDnsRecordRequest,
};
