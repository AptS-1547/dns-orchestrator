//! 类型定义模块

mod account;
mod domain;
mod domain_metadata;
mod export;
mod response;
mod toolbox;

pub use account::{Account, AccountStatus, CreateAccountRequest, UpdateAccountRequest};
pub use domain::AppDomain;
pub use domain_metadata::{
    BatchTagFailure, BatchTagRequest, BatchTagResult, DomainMetadata, DomainMetadataKey,
    DomainMetadataUpdate,
};
pub use export::{
    ExportAccountsRequest, ExportAccountsResponse, ExportFile, ExportFileHeader, ExportedAccount,
    ImportAccountsRequest, ImportFailure, ImportPreview, ImportPreviewAccount, ImportResult,
};
pub use response::{ApiResponse, BatchDeleteFailure, BatchDeleteRequest, BatchDeleteResult};
pub use toolbox::{
    CertChainItem, DnsLookupRecord, DnsLookupResult, DnsPropagationResult, DnsPropagationServer,
    DnsPropagationServerResult, DnskeyRecord, DnssecResult, DsRecord, HttpHeader,
    HttpHeaderCheckRequest, HttpHeaderCheckResult, HttpMethod, IpGeoInfo, IpLookupResult,
    RrsigRecord, SecurityHeaderAnalysis, SslCertInfo, SslCheckResult, WhoisResult,
};

// Re-export provider 库的公共类型
pub use dns_orchestrator_provider::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, DomainStatus, PaginatedResponse,
    PaginationParams, ProviderCredentials, ProviderDomain, ProviderMetadata, ProviderType,
    RecordQueryParams, UpdateDnsRecordRequest,
};
