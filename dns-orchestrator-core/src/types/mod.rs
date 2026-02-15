//! Core type definitions.

mod account;
mod dns;
mod domain;
mod domain_metadata;
mod export;

pub use account::{Account, AccountStatus, CreateAccountRequest, UpdateAccountRequest};
pub use dns::BatchDeleteRequest;
pub use domain::AppDomain;
pub use domain_metadata::{
    BatchTagFailure, BatchTagRequest, BatchTagResult, DomainMetadata, DomainMetadataKey,
    DomainMetadataUpdate,
};
pub use export::{
    ExportAccountsRequest, ExportAccountsResponse, ExportFile, ExportFileHeader, ExportedAccount,
    ImportAccountsRequest, ImportFailure, ImportPreview, ImportPreviewAccount, ImportResult,
};

// Re-export commonly used types from `dns-orchestrator-provider`.
pub use dns_orchestrator_provider::{
    BatchCreateFailure, BatchCreateResult, BatchDeleteFailure, BatchDeleteResult,
    BatchUpdateFailure, BatchUpdateItem, BatchUpdateResult, CreateDnsRecordRequest,
    CredentialValidationError, DnsRecord, DnsRecordType, DomainStatus, PaginatedResponse,
    PaginationParams, ProviderCredentials, ProviderDomain, ProviderMetadata, ProviderType,
    RecordData, RecordQueryParams, UpdateDnsRecordRequest,
};
