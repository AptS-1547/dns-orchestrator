//! Platform adapters for MCP Server (Read-Only)
//!
//! Provides trait implementations that bridge the MCP layer with the core business logic.
//!
//! # Read-Only Design
//!
//! All adapters operate in read-only mode:
//! - **`TauriStoreAccountRepository`**: Reads account data from Tauri store files.
//!   Write operations only update the in-memory cache without persisting to disk.
//! - **`KeyringCredentialStore`**: Reads credentials from system keyring.
//!   Write operations silently succeed without modifying the keyring.
//! - **`NoOpDomainMetadataRepository`**: No-op implementation, returns empty/default values.
//!
//! This design ensures the MCP server acts as a read-only view of the desktop app's data,
//! preventing accidental modifications while allowing full read access to accounts and domains.

mod account_repository;
mod credential_store;
mod domain_metadata_repository;

pub use account_repository::TauriStoreAccountRepository;
pub use credential_store::KeyringCredentialStore;
pub use domain_metadata_repository::NoOpDomainMetadataRepository;
