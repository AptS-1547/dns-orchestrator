//! Provider registry abstraction.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use dns_orchestrator_provider::DnsProvider;

/// Registry of runtime provider instances.
///
/// Stores provider instances keyed by `account_id`.
/// Includes an in-memory implementation: [`InMemoryProviderRegistry`].
#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    /// Registers a provider instance.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    /// * `provider` - Provider instance.
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);

    /// Unregisters a provider.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    async fn unregister(&self, account_id: &str);

    /// Returns a provider instance.
    ///
    /// # Arguments
    /// * `account_id` - Account ID.
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;

    /// Lists all registered account IDs.
    async fn list_account_ids(&self) -> Vec<String>;
}

/// In-memory provider registry.
///
/// Default implementation, available on all platforms.
#[derive(Clone)]
pub struct InMemoryProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl InMemoryProviderRegistry {
    /// Creates a new in-memory registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderRegistry for InMemoryProviderRegistry {
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.providers.write().await.insert(account_id, provider);
    }

    async fn unregister(&self, account_id: &str) {
        self.providers.write().await.remove(account_id);
    }

    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>> {
        self.providers.read().await.get(account_id).cloned()
    }

    async fn list_account_ids(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }
}
