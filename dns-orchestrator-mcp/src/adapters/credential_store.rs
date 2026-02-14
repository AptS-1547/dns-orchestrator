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

trait KeyringBackend: Send + Sync {
    fn read_raw_json(&self) -> CoreResult<String>;
}

struct SystemKeyringBackend;

impl KeyringBackend for SystemKeyringBackend {
    fn read_raw_json(&self) -> CoreResult<String> {
        let entry = keyring::Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
            .map_err(|e| CoreError::CredentialError(e.to_string()))?;

        match entry.get_password() {
            Ok(json) => Ok(json),
            Err(keyring::Error::NoEntry) => Ok("{}".to_string()),
            Err(e) => Err(CoreError::CredentialError(e.to_string())),
        }
    }
}

fn parse_credentials_json(json: &str) -> CoreResult<CredentialsMap> {
    if json.trim().is_empty() || json.trim() == "{}" {
        return Ok(std::collections::HashMap::new());
    }

    serde_json::from_str(json).map_err(|e| CoreError::SerializationError(e.to_string()))
}

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
    /// Keyring access backend.
    backend: Arc<dyn KeyringBackend>,
}

impl KeyringCredentialStore {
    /// Create a new read-only credential store instance.
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_backend(Arc::new(SystemKeyringBackend))
    }

    fn new_with_backend(backend: Arc<dyn KeyringBackend>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            backend,
        }
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
        let backend = Arc::clone(&self.backend);
        let credentials = tokio::task::spawn_blocking(move || {
            tracing::debug!("Loading all credentials from Keyring");
            backend
                .read_raw_json()
                .and_then(|json| parse_credentials_json(&json))
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
        tracing::debug!(
            "Saved {} accounts to cache (read-only mode)",
            credentials.len()
        );
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
        let backend = Arc::clone(&self.backend);
        tokio::task::spawn_blocking(move || backend.read_raw_json())
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
    }

    async fn save_raw_json(&self, _json: &str) -> CoreResult<()> {
        // Read-only mode: silently succeed
        tracing::debug!("Save raw JSON operation ignored (read-only mode)");
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use dns_orchestrator_core::types::ProviderType;

    struct FakeBackend {
        raw_json: Mutex<String>,
        error: Mutex<Option<String>>,
        read_calls: AtomicUsize,
    }

    impl FakeBackend {
        fn new(raw_json: &str) -> Self {
            Self {
                raw_json: Mutex::new(raw_json.to_string()),
                error: Mutex::new(None),
                read_calls: AtomicUsize::new(0),
            }
        }

        fn with_error(message: &str) -> Self {
            Self {
                raw_json: Mutex::new("{}".to_string()),
                error: Mutex::new(Some(message.to_string())),
                read_calls: AtomicUsize::new(0),
            }
        }

        fn set_raw_json(&self, raw_json: &str) {
            *self.raw_json.lock().unwrap() = raw_json.to_string();
        }

        fn read_calls(&self) -> usize {
            self.read_calls.load(Ordering::SeqCst)
        }
    }

    impl KeyringBackend for FakeBackend {
        fn read_raw_json(&self) -> CoreResult<String> {
            self.read_calls.fetch_add(1, Ordering::SeqCst);

            if let Some(message) = self.error.lock().unwrap().clone() {
                return Err(CoreError::CredentialError(message));
            }

            Ok(self.raw_json.lock().unwrap().clone())
        }
    }

    fn sample_credentials_map() -> CredentialsMap {
        let mut map = HashMap::new();
        map.insert(
            "acc-1".to_string(),
            ProviderCredentials::Cloudflare {
                api_token: "token-1".to_string(),
            },
        );
        map
    }

    fn sample_credentials_json() -> String {
        serde_json::to_string(&sample_credentials_map()).unwrap()
    }

    #[tokio::test]
    async fn load_all_returns_empty_for_blank_json() {
        let backend = Arc::new(FakeBackend::new("   "));
        let store = KeyringCredentialStore::new_with_backend(backend);

        let credentials = store.load_all().await.unwrap();
        assert!(credentials.is_empty());
    }

    #[tokio::test]
    async fn load_all_returns_serialization_error_for_invalid_json() {
        let backend = Arc::new(FakeBackend::new("not-json"));
        let store = KeyringCredentialStore::new_with_backend(backend);

        let error = store.load_all().await.unwrap_err();
        assert!(matches!(error, CoreError::SerializationError(_)));
    }

    #[tokio::test]
    async fn load_all_uses_cache_after_first_load() {
        let backend = Arc::new(FakeBackend::new(&sample_credentials_json()));
        let store = KeyringCredentialStore::new_with_backend(backend.clone());

        let first = store.load_all().await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(backend.read_calls(), 1);

        backend.set_raw_json("{}");

        let second = store.load_all().await.unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(backend.read_calls(), 1);
    }

    #[tokio::test]
    async fn save_all_updates_cache_and_get_reads_from_cache() {
        let backend = Arc::new(FakeBackend::new("{}"));
        let store = KeyringCredentialStore::new_with_backend(backend);

        let credentials = sample_credentials_map();
        store.save_all(&credentials).await.unwrap();

        let loaded = store.load_all().await.unwrap();
        assert_eq!(loaded.len(), 1);

        let one = store.get("acc-1").await.unwrap().unwrap();
        assert_eq!(one.provider_type(), ProviderType::Cloudflare);
    }

    #[tokio::test]
    async fn load_raw_json_propagates_backend_errors() {
        let backend = Arc::new(FakeBackend::with_error("backend read failed"));
        let store = KeyringCredentialStore::new_with_backend(backend);

        let error = store.load_raw_json().await.unwrap_err();
        assert!(matches!(error, CoreError::CredentialError(_)));
        assert!(error.to_string().contains("backend read failed"));
    }

    #[tokio::test]
    async fn set_remove_and_save_raw_json_are_noop_success() {
        let backend = Arc::new(FakeBackend::new("{}"));
        let store = KeyringCredentialStore::new_with_backend(backend);

        store
            .set(
                "acc-1",
                &ProviderCredentials::Cloudflare {
                    api_token: "token".to_string(),
                },
            )
            .await
            .unwrap();
        store.remove("acc-1").await.unwrap();
        store.save_raw_json("{\"anything\":true}").await.unwrap();
    }
}
