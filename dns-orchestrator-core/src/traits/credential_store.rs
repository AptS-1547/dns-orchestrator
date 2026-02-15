//! Credential storage abstraction.

use async_trait::async_trait;
use dns_orchestrator_provider::ProviderCredentials;
use std::collections::HashMap;

use crate::error::CoreResult;

/// Credential map type: `account_id` -> `ProviderCredentials` (type-safe).
pub type CredentialsMap = HashMap<String, ProviderCredentials>;

/// Legacy credential map used during migration.
pub type LegacyCredentialsMap = HashMap<String, HashMap<String, String>>;

/// Credential storage interface.
///
/// Platform implementations:
/// - Tauri Desktop: `TauriCredentialStore` (`keyring` crate)
/// - Tauri Android: `TauriCredentialStore` (tauri-plugin-store)
/// - Actix-Web: `DatabaseCredentialStore` (`SeaORM` + AES encryption)
///
/// # v1.7.0 changes
///
/// - Use type-safe `ProviderCredentials` instead of `HashMap<String, String>`
/// - Methods renamed to match Rust idioms: `load()` → `get()`, `save()` → `set()`, `delete()` → `remove()`
/// - Added `save_all()` for batch saving (migration scenario)
/// - Added `load_raw_json()` and `save_raw_json()` for migration detection
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Loads all credentials in the new format.
    ///
    /// Used at startup to reduce storage round-trips.
    async fn load_all(&self) -> CoreResult<CredentialsMap>;

    /// Saves credentials in batch in the new format.
    ///
    /// Used during migration to persist all credentials in one write.
    ///
    /// # Arguments
    /// * `credentials` - Credential map.
    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()>;

    /// Loads credentials for one account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    ///
    /// # Returns
    /// * `Ok(Some(credentials))` - Credentials exist.
    /// * `Ok(None)` - Credentials do not exist.
    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>>;

    /// Saves credentials for one account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    /// * `credentials` - Type-safe credential payload.
    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()>;

    /// Removes credentials for one account.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    async fn remove(&self, account_id: &str) -> CoreResult<()>;

    // === Migration helper methods ===

    /// Loads raw JSON for format detection and migration.
    ///
    /// Returns the raw JSON string without deserialization.
    async fn load_raw_json(&self) -> CoreResult<String>;

    /// Saves raw JSON during migration writes.
    ///
    /// Writes JSON directly to storage without (de)serialization.
    ///
    /// # Arguments
    /// * `json` - JSON string.
    async fn save_raw_json(&self, json: &str) -> CoreResult<()>;
}
