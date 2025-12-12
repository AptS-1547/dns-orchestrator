//! 类型定义模块

mod account;
mod domain;
mod export;
mod response;
mod toolbox;

pub use account::{Account, AccountStatus, CreateAccountRequest};
pub use domain::Domain;
pub use export::{
    ExportAccountsRequest, ExportAccountsResponse, ExportFile, ExportFileHeader, ExportedAccount,
    ImportAccountsRequest, ImportFailure, ImportPreview, ImportPreviewAccount, ImportResult,
};
pub use response::{ApiResponse, BatchDeleteFailure, BatchDeleteRequest, BatchDeleteResult};
pub use toolbox::{
    CertChainItem, DnsLookupRecord, DnsLookupResult, IpGeoInfo, IpLookupResult, SslCertInfo,
    SslCheckResult, WhoisResult,
};

// Re-export provider 库的公共类型
pub use dns_orchestrator_provider::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, Domain as LibDomain, DomainStatus,
    PaginatedResponse, PaginationParams, ProviderCredentials, ProviderMetadata, ProviderType,
    RecordQueryParams, UpdateDnsRecordRequest,
};
