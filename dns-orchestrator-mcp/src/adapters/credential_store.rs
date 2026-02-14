//! Keyring-based credential store (Read-Only)
//!
//! Shares credentials with the desktop Tauri app via system keyring.
//! This is a read-only implementation - all write operations are no-ops.

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
///
/// **Read-Only Mode**: All write operations (`save_all`, `set`, `remove`,
/// `save_raw_json`) silently succeed without modifying the keyring.
/// This prevents the MCP server from modifying the desktop app's credentials.
pub struct KeyringCredentialStore {
    /// In-memory cache to reduce keyring access frequency.
    cache: Arc<RwLock<Option<CredentialsMap>>>,
}

impl KeyringCredentialStore {
    /// Create a new read-only credential store instance.
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

    /// Read all credentials from keyring (synchronous).
    fn read_all_sync() -> CoreResult<CredentialsMap> {
        let json = Self::read_raw_sync()?;

        if json.trim().is_empty() || json.trim() == "{}" {
            return Ok(std::collections::HashMap::new());
        }

        serde_json::from_str(&json).map_err(|e| CoreError::SerializationError(e.to_string()))
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
        // Read-only mode: only update in-memory cache
        self.update_cache(credentials.clone()).await;
        tracing::debug!("Saved {} accounts to cache (read-only mode)", credentials.len());
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        let all_creds = self.load_all().await?;
        Ok(all_creds.get(account_id).cloned())
    }

    async fn set(&self, _account_id: &str, _credentials: &ProviderCredentials) -> CoreResult<()> {
        // Read-only mode: silently succeed
        tracing::debug!("Set operation ignored (read-only mode)");
        Ok(())
    }

    async fn remove(&self, _account_id: &str) -> CoreResult<()> {
        // Read-only mode: silently succeed
        tracing::debug!("Remove operation ignored (read-only mode)");
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        tokio::task::spawn_blocking(Self::read_raw_sync)
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
    }

    async fn save_raw_json(&self, _json: &str) -> CoreResult<()> {
        // Read-only mode: silently succeed
        tracing::debug!("Save raw JSON operation ignored (read-only mode)");
        Ok(())
    }
}
