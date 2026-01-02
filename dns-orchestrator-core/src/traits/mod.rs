//! 存储层抽象 Trait 定义

mod account_repository;
mod credential_store;
mod domain_metadata_repository;
mod provider_registry;

pub use account_repository::AccountRepository;
pub use credential_store::{CredentialStore, CredentialsMap, LegacyCredentialsMap};
pub use domain_metadata_repository::DomainMetadataRepository;
pub use provider_registry::{InMemoryProviderRegistry, ProviderRegistry};
