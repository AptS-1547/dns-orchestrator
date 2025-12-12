//! 平台适配器模块

mod account_repository;
mod credential_store;

pub use account_repository::TauriAccountRepository;
pub use credential_store::TauriCredentialStore;
