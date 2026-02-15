//! DNS Orchestrator core library.
//!
//! Provides platform-agnostic DNS management business logic, including:
//! - Account management
//! - Domain listing and lookup
//! - DNS record CRUD
//! - Import/export and migration workflows
//!
//! Storage and runtime integrations are injected via traits, which allows the same core logic
//! to run on Tauri (desktop/mobile) and Actix-Web backends.

/// Cryptography helpers for import/export encryption.
pub mod crypto;
/// Unified error types used by the core library.
pub mod error;
/// Business services that orchestrate repositories and providers.
pub mod services;
/// Storage/runtime abstraction traits.
pub mod traits;
/// Shared core data structures and provider re-exports.
pub mod types;
/// Utility helpers.
pub mod utils;

#[cfg(test)]
mod test_utils;

// Re-export common entry points.
pub use error::{CoreError, CoreResult};
pub use services::ServiceContext;
pub use traits::{AccountRepository, CredentialStore, DomainMetadataRepository, ProviderRegistry};
