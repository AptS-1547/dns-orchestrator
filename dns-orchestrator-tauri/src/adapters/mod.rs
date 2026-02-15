//! 平台适配器模块

// CredentialStore（桌面端复用 app 层，Android 用 tauri-plugin-store）
mod credential_store;
pub use credential_store::TauriCredentialStore;

// 桌面端：AccountRepository + DomainMetadataRepository 复用 app 层 SqliteStore
#[cfg(not(target_os = "android"))]
pub use dns_orchestrator_app::adapters::SqliteStore;

// Android 端：保留 tauri-plugin-store 实现
#[cfg(target_os = "android")]
mod account_repository;
#[cfg(target_os = "android")]
pub use account_repository::TauriAccountRepository;

#[cfg(target_os = "android")]
mod domain_metadata_repository;
#[cfg(target_os = "android")]
pub use domain_metadata_repository::TauriDomainMetadataRepository;
