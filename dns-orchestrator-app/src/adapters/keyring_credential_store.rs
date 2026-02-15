//! Keyring-based credential store.
//!
//! Uses the system keychain (macOS Keychain, Windows Credential Manager,
//! Linux Secret Service) via the `keyring` crate. Same logic as the Tauri
//! desktop `TauriCredentialStore`, but without any Tauri dependency.

use async_trait::async_trait;
use dns_orchestrator_core::types::ProviderCredentials;
use keyring::Entry;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap};

const SERVICE_NAME: &str = "dns-orchestrator";
const CREDENTIALS_KEY: &str = "all-credentials";

/// Storage format detection (dual-format support for migration).
#[derive(Deserialize)]
#[serde(untagged)]
enum StorageFormat {
    V2(HashMap<String, ProviderCredentials>),
    #[allow(dead_code)]
    V1(HashMap<String, HashMap<String, String>>),
}

/// Keyring-based credential store.
///
/// Stores all credentials as a single JSON blob in the system keychain.
/// Compatible with the Tauri desktop app's credential storage.
pub struct KeyringCredentialStore {
    /// In-memory cache to avoid repeated keychain reads during one process lifetime.
    cache: Arc<RwLock<Option<CredentialsMap>>>,
}

impl KeyringCredentialStore {
    /// Create a new keyring-backed credential store with an empty cache.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Build a keyring entry handle for the shared credentials record.
    fn get_entry() -> CoreResult<Entry> {
        Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// Read the raw credential JSON from the keychain.
    fn read_raw_sync() -> CoreResult<String> {
        let entry = Self::get_entry()?;
        match entry.get_password() {
            Ok(json) => Ok(json),
            Err(keyring::Error::NoEntry) => Ok("{}".to_string()),
            Err(e) => Err(CoreError::CredentialError(e.to_string())),
        }
    }

    /// Write raw credential JSON to the keychain.
    fn write_raw_sync(json: &str) -> CoreResult<()> {
        let entry = Self::get_entry()?;
        entry
            .set_password(json)
            .map_err(|e| CoreError::CredentialError(e.to_string()))?;
        Ok(())
    }

    /// Read and parse all credentials, validating whether migration is required.
    fn read_all_sync() -> CoreResult<CredentialsMap> {
        let json = Self::read_raw_sync()?;
        match serde_json::from_str::<StorageFormat>(&json) {
            Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
            Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
            Err(_) if json.trim().is_empty() || json.trim() == "{}" => Ok(HashMap::new()),
            Err(e) => Err(CoreError::SerializationError(e.to_string())),
        }
    }

    /// Serialize and write all credentials to the keychain.
    fn write_all_sync(credentials: &CredentialsMap) -> CoreResult<()> {
        let json = serde_json::to_string(credentials)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        Self::write_raw_sync(&json)
    }

    /// Replace the in-memory credential cache.
    async fn update_cache(&self, credentials: CredentialsMap) {
        let mut cache = self.cache.write().await;
        *cache = Some(credentials);
    }
}

impl Default for KeyringCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CredentialStore for KeyringCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        {
            let cache = self.cache.read().await;
            if let Some(ref creds) = *cache {
                return Ok(creds.clone());
            }
        }

        let mut cache = self.cache.write().await;
        if let Some(ref creds) = *cache {
            return Ok(creds.clone());
        }

        let credentials = tokio::task::spawn_blocking(|| {
            log::debug!("Loading all credentials from Keychain");
            Self::read_all_sync()
        })
        .await
        .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?;

        let credentials = credentials?;
        *cache = Some(credentials.clone());
        log::info!("Loaded {} accounts from Keychain", credentials.len());
        Ok(credentials)
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        let creds_clone = credentials.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_clone))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??;

        self.update_cache(credentials.clone()).await;
        log::info!("Saved {} accounts to Keychain", credentials.len());
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        let all_creds = self.load_all().await?;
        Ok(all_creds.get(account_id).cloned())
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        let mut all_creds = match cache.take() {
            Some(creds) => creds,
            None => tokio::task::spawn_blocking(Self::read_all_sync)
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??,
        };

        all_creds.insert(account_id.to_string(), credentials.clone());

        let creds_for_save = all_creds.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_for_save))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??;

        *cache = Some(all_creds);
        log::info!("Credentials saved for account: {account_id}");
        Ok(())
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        let mut all_creds = match cache.take() {
            Some(creds) => creds,
            None => tokio::task::spawn_blocking(Self::read_all_sync)
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??,
        };

        all_creds.remove(account_id);

        let creds_for_save = all_creds.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_for_save))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??;

        *cache = Some(all_creds);
        log::info!("Credentials deleted for account: {account_id}");
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        tokio::task::spawn_blocking(Self::read_raw_sync)
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
    }

    async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
        let json_clone = json.to_string();
        tokio::task::spawn_blocking(move || Self::write_raw_sync(&json_clone))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
    }
}
