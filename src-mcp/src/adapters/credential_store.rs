//! Keyring-based credential store
//!
//! Shares credentials with the desktop Tauri app via system keyring.

use async_trait::async_trait;
use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap};
use dns_orchestrator_provider::ProviderCredentials;
use std::sync::Arc;
use tokio::sync::RwLock;

const SERVICE_NAME: &str = "dns-orchestrator";
const CREDENTIALS_KEY: &str = "all-credentials";

/// Keyring-based credential store that shares credentials with the desktop app.
///
/// Uses the system keychain (Keychain on macOS, Credential Manager on Windows,
/// secret-service on Linux) to store credentials securely.
pub struct KeyringCredentialStore {
    /// In-memory cache to reduce keyring access frequency.
    cache: Arc<RwLock<Option<CredentialsMap>>>,
}

impl KeyringCredentialStore {
    /// Create a new credential store instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }

    fn get_entry() -> CoreResult<keyring::Entry> {
        keyring::Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// Read raw JSON from keyring (synchronous).
    fn read_raw_sync() -> CoreResult<String> {
        let entry = Self::get_entry()?;

        match entry.get_password() {
            Ok(json) => Ok(json),
            Err(keyring::Error::NoEntry) => Ok("{}".to_string()),
            Err(e) => Err(CoreError::CredentialError(e.to_string())),
        }
    }

    /// Write raw JSON to keyring (synchronous).
    fn write_raw_sync(json: &str) -> CoreResult<()> {
        let entry = Self::get_entry()?;
        entry
            .set_password(json)
            .map_err(|e| CoreError::CredentialError(e.to_string()))?;
        Ok(())
    }

    /// Read all credentials from keyring (synchronous).
    fn read_all_sync() -> CoreResult<CredentialsMap> {
        let json = Self::read_raw_sync()?;

        if json.trim().is_empty() || json.trim() == "{}" {
            return Ok(std::collections::HashMap::new());
        }

        serde_json::from_str(&json).map_err(|e| CoreError::SerializationError(e.to_string()))
    }

    /// Write all credentials to keyring (synchronous).
    fn write_all_sync(credentials: &CredentialsMap) -> CoreResult<()> {
        let json = serde_json::to_string(credentials)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        Self::write_raw_sync(&json)
    }

    /// Update cache (helper method).
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
        // Check cache first (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(ref creds) = *cache {
                return Ok(creds.clone());
            }
        }

        // Cache empty, acquire write lock and load (double-check)
        let mut cache = self.cache.write().await;
        if let Some(ref creds) = *cache {
            return Ok(creds.clone());
        }

        // Load from keyring
        let credentials = tokio::task::spawn_blocking(|| {
            tracing::debug!("Loading all credentials from Keyring");
            Self::read_all_sync()
        })
        .await
        .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??;

        *cache = Some(credentials.clone());
        tracing::info!("Loaded {} accounts from Keyring", credentials.len());
        Ok(credentials)
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        let creds_clone = credentials.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_clone))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??
;

        // Update cache
        self.update_cache(credentials.clone()).await;

        tracing::info!("Saved {} accounts to Keyring", credentials.len());
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        let all_creds = self.load_all().await?;
        Ok(all_creds.get(account_id).cloned())
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        // Load from cache or keyring
        let mut all_creds = match cache.take() {
            Some(creds) => creds,
            None => tokio::task::spawn_blocking(Self::read_all_sync)
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??
,
        };

        all_creds.insert(account_id.to_string(), credentials.clone());

        // Write to keyring
        let creds_for_save = all_creds.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_for_save))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??
;

        *cache = Some(all_creds);
        tracing::info!("Credentials saved for account: {account_id}");
        Ok(())
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        // Load from cache or keyring
        let mut all_creds = match cache.take() {
            Some(creds) => creds,
            None => tokio::task::spawn_blocking(Self::read_all_sync)
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??
,
        };

        all_creds.remove(account_id);

        // Write to keyring
        let creds_for_save = all_creds.clone();
        tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_for_save))
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??
;

        *cache = Some(all_creds);
        tracing::info!("Credentials deleted for account: {account_id}");
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
