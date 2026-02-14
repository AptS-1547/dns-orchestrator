//! Platform adapters for MCP Server
//!
//! Provides trait implementations that bridge the MCP layer with the core business logic.

mod account_repository;
mod credential_store;
mod domain_metadata_repository;

pub use account_repository::TauriStoreAccountRepository;
pub use credential_store::KeyringCredentialStore;
pub use domain_metadata_repository::NoOpDomainMetadataRepository;
