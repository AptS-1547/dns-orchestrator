//! Test utility module.
//!
//! Provides mock implementations and factory helpers for tests.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dns_orchestrator_provider::ProviderCredentials;
use tokio::sync::RwLock;

use crate::error::{CoreError, CoreResult};
use crate::services::{AccountService, ServiceContext};
use crate::traits::{
    AccountRepository, CredentialStore, CredentialsMap, DomainMetadataRepository,
    InMemoryProviderRegistry,
};
use crate::types::{
    Account, AccountStatus, DomainMetadata, DomainMetadataKey, DomainMetadataUpdate,
};

// ===== MockAccountRepository =====

/// In-memory mock for [`AccountRepository`].
pub struct MockAccountRepository {
    accounts: RwLock<HashMap<String, Account>>,
    /// If Some, this error is returned when saving (used to test the cleanup path)
    save_error: RwLock<Option<String>>,
}

impl MockAccountRepository {
    /// Creates a new mock repository.
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
            save_error: RwLock::new(None),
        }
    }

    /// Configures an artificial save error for testing.
    pub async fn set_save_error(&self, err: Option<String>) {
        *self.save_error.write().await = err;
    }
}

#[async_trait]
impl AccountRepository for MockAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        Ok(self.accounts.read().await.values().cloned().collect())
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        Ok(self.accounts.read().await.get(id).cloned())
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        if let Some(ref msg) = *self.save_error.read().await {
            return Err(CoreError::StorageError(msg.clone()));
        }
        self.accounts
            .write()
            .await
            .insert(account.id.clone(), account.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        self.accounts.write().await.remove(id);
        Ok(())
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        let mut store = self.accounts.write().await;
        for account in accounts {
            store.insert(account.id.clone(), account.clone());
        }
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let mut store = self.accounts.write().await;
        if let Some(account) = store.get_mut(id) {
            account.status = Some(status);
            account.error = error;
        }
        Ok(())
    }
}

// ===== MockCredentialStore =====

/// In-memory mock for [`CredentialStore`].
pub struct MockCredentialStore {
    credentials: RwLock<HashMap<String, ProviderCredentials>>,
    /// If Some, this error is returned when remove (used to test the credential removal failure path)
    remove_error: RwLock<Option<String>>,
}

impl MockCredentialStore {
    /// Creates a new mock credential store.
    pub fn new() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
            remove_error: RwLock::new(None),
        }
    }

    /// Configures an artificial remove error for testing.
    pub async fn set_remove_error(&self, err: Option<String>) {
        *self.remove_error.write().await = err;
    }
}

#[async_trait]
impl CredentialStore for MockCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        Ok(self.credentials.read().await.clone())
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        *self.credentials.write().await = credentials.clone();
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        Ok(self.credentials.read().await.get(account_id).cloned())
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        self.credentials
            .write()
            .await
            .insert(account_id.to_string(), credentials.clone());
        Ok(())
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        if let Some(ref msg) = *self.remove_error.read().await {
            return Err(CoreError::StorageError(msg.clone()));
        }
        self.credentials.write().await.remove(account_id);
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        let creds = self.credentials.read().await;
        serde_json::to_string(&*creds).map_err(|e| CoreError::SerializationError(e.to_string()))
    }

    async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
        let creds: CredentialsMap =
            serde_json::from_str(json).map_err(|e| CoreError::SerializationError(e.to_string()))?;
        *self.credentials.write().await = creds;
        Ok(())
    }
}

// ===== MockDomainMetadataRepository =====

/// In-memory mock for [`DomainMetadataRepository`].
pub struct MockDomainMetadataRepository {
    metadata: RwLock<HashMap<String, DomainMetadata>>,
}

impl MockDomainMetadataRepository {
    /// Creates a new mock metadata repository.
    pub fn new() -> Self {
        Self {
            metadata: RwLock::new(HashMap::new()),
        }
    }

    fn make_key(key: &DomainMetadataKey) -> String {
        key.to_storage_key()
    }
}

#[async_trait]
impl DomainMetadataRepository for MockDomainMetadataRepository {
    async fn find_by_key(&self, key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        Ok(self
            .metadata
            .read()
            .await
            .get(&Self::make_key(key))
            .cloned())
    }

    async fn find_by_keys(
        &self,
        keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        let store = self.metadata.read().await;
        let mut result = HashMap::new();
        for key in keys {
            if let Some(m) = store.get(&Self::make_key(key)) {
                result.insert(key.clone(), m.clone());
            }
        }
        Ok(result)
    }

    async fn save(&self, key: &DomainMetadataKey, metadata: &DomainMetadata) -> CoreResult<()> {
        if metadata.is_empty() {
            self.metadata.write().await.remove(&Self::make_key(key));
        } else {
            self.metadata
                .write()
                .await
                .insert(Self::make_key(key), metadata.clone());
        }
        Ok(())
    }

    async fn batch_save(&self, entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()> {
        let mut store = self.metadata.write().await;
        for (key, metadata) in entries {
            if metadata.is_empty() {
                store.remove(&Self::make_key(key));
            } else {
                store.insert(Self::make_key(key), metadata.clone());
            }
        }
        Ok(())
    }

    async fn update(
        &self,
        key: &DomainMetadataKey,
        update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        let mut store = self.metadata.write().await;
        let storage_key = Self::make_key(key);
        let mut metadata = store.get(&storage_key).cloned().unwrap_or_default();
        update.apply_to(&mut metadata);
        if metadata.is_empty() {
            store.remove(&storage_key);
        } else {
            store.insert(storage_key, metadata);
        }
        Ok(())
    }

    async fn delete(&self, key: &DomainMetadataKey) -> CoreResult<()> {
        self.metadata.write().await.remove(&Self::make_key(key));
        Ok(())
    }

    async fn delete_by_account(&self, account_id: &str) -> CoreResult<()> {
        let prefix = format!("{account_id}::");
        self.metadata
            .write()
            .await
            .retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }

    async fn find_favorites_by_account(
        &self,
        account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        let prefix = format!("{account_id}::");
        let store = self.metadata.read().await;
        Ok(store
            .iter()
            .filter(|(k, v)| k.starts_with(&prefix) && v.is_favorite)
            .filter_map(|(k, _)| DomainMetadataKey::from_storage_key(k))
            .collect())
    }

    async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        let store = self.metadata.read().await;
        Ok(store
            .iter()
            .filter(|(_, v)| v.tags.contains(&tag.to_string()))
            .filter_map(|(k, _)| DomainMetadataKey::from_storage_key(k))
            .collect())
    }

    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        let store = self.metadata.read().await;
        let mut tags: Vec<String> = store.values().flat_map(|v| v.tags.clone()).collect();
        tags.sort();
        tags.dedup();
        Ok(tags)
    }
}

// ===== Factory method =====

/// Creates a [`ServiceContext`] wired with mock dependencies.
pub fn create_test_context() -> (
    Arc<ServiceContext>,
    Arc<MockAccountRepository>,
    Arc<MockCredentialStore>,
    Arc<MockDomainMetadataRepository>,
) {
    let account_repo = Arc::new(MockAccountRepository::new());
    let credential_store = Arc::new(MockCredentialStore::new());
    let provider_registry = Arc::new(InMemoryProviderRegistry::new());
    let domain_metadata_repo = Arc::new(MockDomainMetadataRepository::new());

    let ctx = Arc::new(ServiceContext::new(
        credential_store.clone(),
        account_repo.clone(),
        provider_registry,
        domain_metadata_repo.clone(),
    ));

    (ctx, account_repo, credential_store, domain_metadata_repo)
}

/// Creates an [`AccountService`] wired with mock dependencies.
pub fn create_test_account_service() -> (
    AccountService,
    Arc<MockAccountRepository>,
    Arc<MockCredentialStore>,
    Arc<MockDomainMetadataRepository>,
) {
    let (ctx, account_repo, credential_store, domain_metadata_repo) = create_test_context();
    let service = AccountService::new(ctx);
    (
        service,
        account_repo,
        credential_store,
        domain_metadata_repo,
    )
}

/// Creates default credentials for tests.
///
/// Cloudflare is used because it requires only one field.
pub fn test_credentials() -> ProviderCredentials {
    ProviderCredentials::Cloudflare {
        api_token: "test-token-12345".to_string(),
    }
}
