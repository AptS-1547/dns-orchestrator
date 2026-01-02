//! 平台适配器模块

mod account_repository;
mod credential_store;
mod domain_metadata_repository;

pub use account_repository::TauriAccountRepository;
pub use credential_store::TauriCredentialStore;
pub use domain_metadata_repository::TauriDomainMetadataRepository;
