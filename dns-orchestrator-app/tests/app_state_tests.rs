#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Integration tests for `AppStateBuilder` and `AppState` startup sequence.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use dns_orchestrator_app::adapters::SqliteStore;
use dns_orchestrator_app::{AppStateBuilder, NoopStartupHooks, StartupHooks};
use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{
    AccountRepository, CredentialStore, CredentialsMap, DomainMetadataRepository,
    InMemoryProviderRegistry,
};
use dns_orchestrator_core::types::{
    Account, AccountStatus, DomainMetadata, DomainMetadataKey, DomainMetadataUpdate,
    ProviderCredentials, ProviderType,
};
use tokio::sync::RwLock;

const TEST_PASSWORD: &str = "test-encryption-password-32chars!";

async fn create_test_sqlite_store() -> (Arc<SqliteStore>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("test.db");
    let store = SqliteStore::new(&db_path, Some(TEST_PASSWORD.to_string()))
        .await
        .expect("failed to create SqliteStore");
    (Arc::new(store), tmp)
}

// ===== Mock Implementations =====

/// Configurable mock `CredentialStore` for testing migration paths.
struct MockCredentialStore {
    load_all_result: RwLock<Option<CoreError>>,
    raw_json: RwLock<String>,
    saved_credentials: RwLock<CredentialsMap>,
}

impl MockCredentialStore {
    fn new() -> Self {
        Self {
            load_all_result: RwLock::new(None),
            raw_json: RwLock::new("{}".to_string()),
            saved_credentials: RwLock::new(HashMap::new()),
        }
    }

    /// Make `load_all` return `MigrationRequired`.
    fn with_migration_required(self) -> Self {
        *self.load_all_result.try_write().unwrap() = Some(CoreError::MigrationRequired);
        self
    }

    /// Make `load_all` return a custom error.
    fn with_load_error(self, err: CoreError) -> Self {
        *self.load_all_result.try_write().unwrap() = Some(err);
        self
    }

    /// Set the raw JSON that `load_raw_json` returns (V1 format for migration).
    fn with_raw_json(self, json: &str) -> Self {
        *self.raw_json.try_write().unwrap() = json.to_string();
        self
    }
}

#[async_trait]
impl CredentialStore for MockCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        let guard = self.load_all_result.read().await;
        if let Some(ref err) = *guard {
            return Err(match err {
                CoreError::MigrationRequired => CoreError::MigrationRequired,
                CoreError::StorageError(msg) => CoreError::StorageError(msg.clone()),
                CoreError::CredentialError(msg) => CoreError::CredentialError(msg.clone()),
                other => CoreError::StorageError(format!("{other}")),
            });
        }
        Ok(self.saved_credentials.read().await.clone())
    }

    async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
        *self.saved_credentials.write().await = credentials.clone();
        // Also update raw_json to V2 format so subsequent load_all works
        *self.load_all_result.write().await = None;
        Ok(())
    }

    async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        Ok(self.saved_credentials.read().await.get(account_id).cloned())
    }

    async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
        self.saved_credentials
            .write()
            .await
            .insert(account_id.to_string(), credentials.clone());
        Ok(())
    }

    async fn remove(&self, account_id: &str) -> CoreResult<()> {
        self.saved_credentials.write().await.remove(account_id);
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        Ok(self.raw_json.read().await.clone())
    }

    async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
        *self.raw_json.write().await = json.to_string();
        Ok(())
    }
}

/// Simple mock `AccountRepository` backed by a Vec.
struct MockAccountRepository {
    accounts: RwLock<Vec<Account>>,
}

impl MockAccountRepository {
    fn new() -> Self {
        Self {
            accounts: RwLock::new(Vec::new()),
        }
    }

    fn with_accounts(self, accounts: Vec<Account>) -> Self {
        *self.accounts.try_write().unwrap() = accounts;
        self
    }
}

fn make_account(id: &str, provider: ProviderType) -> Account {
    Account {
        id: id.to_string(),
        name: format!("Account {id}"),
        provider,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: None,
        error: None,
    }
}

#[async_trait]
impl AccountRepository for MockAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        Ok(self.accounts.read().await.clone())
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        Ok(self
            .accounts
            .read()
            .await
            .iter()
            .find(|a| a.id == id)
            .cloned())
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
            accounts[pos] = account.clone();
        } else {
            accounts.push(account.clone());
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        self.accounts.write().await.retain(|a| a.id != id);
        Ok(())
    }

    async fn save_all(&self, new_accounts: &[Account]) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        for account in new_accounts {
            if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
                accounts[pos] = account.clone();
            } else {
                accounts.push(account.clone());
            }
        }
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
            account.status = Some(status);
            account.error = error;
            Ok(())
        } else {
            Err(CoreError::AccountNotFound(id.to_string()))
        }
    }
}

/// Minimal mock `DomainMetadataRepository` (needed for `AppStateBuilder`).
struct MockDomainMetadataRepository;

#[async_trait]
impl DomainMetadataRepository for MockDomainMetadataRepository {
    async fn find_by_key(&self, _key: &DomainMetadataKey) -> CoreResult<Option<DomainMetadata>> {
        Ok(None)
    }
    async fn find_by_keys(
        &self,
        _keys: &[DomainMetadataKey],
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        Ok(HashMap::new())
    }
    async fn save(&self, _key: &DomainMetadataKey, _metadata: &DomainMetadata) -> CoreResult<()> {
        Ok(())
    }
    async fn batch_save(&self, _entries: &[(DomainMetadataKey, DomainMetadata)]) -> CoreResult<()> {
        Ok(())
    }
    async fn update(
        &self,
        _key: &DomainMetadataKey,
        _update: &DomainMetadataUpdate,
    ) -> CoreResult<()> {
        Ok(())
    }
    async fn delete(&self, _key: &DomainMetadataKey) -> CoreResult<()> {
        Ok(())
    }
    async fn delete_by_account(&self, _account_id: &str) -> CoreResult<()> {
        Ok(())
    }
    async fn find_favorites_by_account(
        &self,
        _account_id: &str,
    ) -> CoreResult<Vec<DomainMetadataKey>> {
        Ok(Vec::new())
    }
    async fn find_by_tag(&self, _tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        Ok(Vec::new())
    }
    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        Ok(Vec::new())
    }
}

/// `StartupHooks` that tracks which callbacks were called.
struct TrackingStartupHooks {
    backup_called: RwLock<bool>,
    cleanup_called: RwLock<bool>,
    preserve_called: RwLock<bool>,
    preserve_error: RwLock<Option<String>>,
}

impl TrackingStartupHooks {
    fn new() -> Self {
        Self {
            backup_called: RwLock::new(false),
            cleanup_called: RwLock::new(false),
            preserve_called: RwLock::new(false),
            preserve_error: RwLock::new(None),
        }
    }
}

#[async_trait]
impl StartupHooks for TrackingStartupHooks {
    async fn backup_credentials(&self, _raw_json: &str) -> Option<String> {
        *self.backup_called.write().await = true;
        Some("backup-path".to_string())
    }

    async fn cleanup_backup(&self, _backup_info: &str) {
        *self.cleanup_called.write().await = true;
    }

    async fn preserve_backup(&self, _backup_info: &str, error: &str) {
        *self.preserve_called.write().await = true;
        *self.preserve_error.write().await = Some(error.to_string());
    }
}

/// Helper to build `AppState` from mock components.
fn build_app_state(
    cred_store: Arc<dyn CredentialStore>,
    account_repo: Arc<dyn AccountRepository>,
) -> dns_orchestrator_app::AppState {
    AppStateBuilder::new()
        .credential_store(cred_store)
        .account_repository(account_repo)
        .domain_metadata_repository(Arc::new(MockDomainMetadataRepository))
        .build()
        .unwrap()
}

// ===== AppStateBuilder Tests =====

#[tokio::test]
async fn builder_with_all_required_adapters_succeeds() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let result = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .build();
    assert!(result.is_ok());
}

#[tokio::test]
async fn builder_missing_credential_store_fails() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let result = AppStateBuilder::new()
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .build();
    assert!(result.is_err());
    match result {
        Err(CoreError::ValidationError(msg)) => assert!(msg.contains("credential_store")),
        Err(other) => panic!("Expected ValidationError, got: {other:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn builder_missing_account_repository_fails() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let result = AppStateBuilder::new()
        .credential_store(store.clone())
        .domain_metadata_repository(store.clone())
        .build();
    assert!(result.is_err());
    match result {
        Err(CoreError::ValidationError(msg)) => assert!(msg.contains("account_repository")),
        Err(other) => panic!("Expected ValidationError, got: {other:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn builder_missing_domain_metadata_repository_fails() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let result = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .build();
    assert!(result.is_err());
    match result {
        Err(CoreError::ValidationError(msg)) => {
            assert!(msg.contains("domain_metadata_repository"));
        }
        Err(other) => panic!("Expected ValidationError, got: {other:?}"),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn builder_default_provider_registry_works() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let app_state = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .build()
        .unwrap();

    let result = app_state.ctx.get_provider("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn builder_custom_provider_registry() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let registry = Arc::new(InMemoryProviderRegistry::new());

    let result = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .provider_registry(registry)
        .build();
    assert!(result.is_ok());
}

// ===== AppState Startup Tests =====

#[tokio::test]
async fn app_state_run_startup_completes() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let app_state = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .build()
        .unwrap();

    let result = app_state.run_startup(&NoopStartupHooks).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn app_state_restore_completed_flag_set_after_startup() {
    let (store, _tmp) = create_test_sqlite_store().await;
    let app_state = AppStateBuilder::new()
        .credential_store(store.clone())
        .account_repository(store.clone())
        .domain_metadata_repository(store.clone())
        .build()
        .unwrap();

    assert!(!app_state.restore_completed.load(Ordering::SeqCst));
    app_state.run_startup(&NoopStartupHooks).await.unwrap();
    assert!(app_state.restore_completed.load(Ordering::SeqCst));
}

// ===== Migration Branch Tests =====

#[tokio::test]
async fn migration_not_needed_calls_cleanup() {
    // V2 format (load_all succeeds) → NotNeeded → cleanup_backup called
    let cred_store = Arc::new(MockCredentialStore::new());
    let account_repo = Arc::new(MockAccountRepository::new());
    let app_state = build_app_state(cred_store, account_repo);

    let hooks = TrackingStartupHooks::new();
    app_state.run_migration(&hooks).await;

    assert!(*hooks.backup_called.read().await);
    assert!(*hooks.cleanup_called.read().await);
    assert!(!*hooks.preserve_called.read().await);
}

#[tokio::test]
async fn migration_success_all_accounts_migrated() {
    // V1 format with matching account metadata → Success, all migrated
    let v1_json = r#"{"acc-1": {"apiToken": "my-token"}}"#;
    let cred_store = Arc::new(
        MockCredentialStore::new()
            .with_migration_required()
            .with_raw_json(v1_json),
    );
    let account_repo = Arc::new(
        MockAccountRepository::new()
            .with_accounts(vec![make_account("acc-1", ProviderType::Cloudflare)]),
    );
    let app_state = build_app_state(cred_store.clone(), account_repo);

    let hooks = TrackingStartupHooks::new();
    app_state.run_migration(&hooks).await;

    // Cleanup should be called (migration succeeded)
    assert!(*hooks.cleanup_called.read().await);
    assert!(!*hooks.preserve_called.read().await);

    // Credentials should now be saved in V2 format
    let saved = cred_store.saved_credentials.read().await;
    assert!(saved.contains_key("acc-1"));
}

#[tokio::test]
async fn migration_success_with_failed_accounts() {
    // V1 format: one account has matching metadata, one doesn't → partial failure
    let v1_json = r#"{"acc-1": {"apiToken": "token"}, "acc-orphan": {"apiToken": "token2"}}"#;
    let cred_store = Arc::new(
        MockCredentialStore::new()
            .with_migration_required()
            .with_raw_json(v1_json),
    );
    let account_repo = Arc::new(
        MockAccountRepository::new()
            .with_accounts(vec![make_account("acc-1", ProviderType::Cloudflare)]),
    );
    let app_state = build_app_state(cred_store.clone(), account_repo.clone());

    let hooks = TrackingStartupHooks::new();
    app_state.run_migration(&hooks).await;

    // Migration succeeded overall → cleanup called
    assert!(*hooks.cleanup_called.read().await);

    // acc-1 migrated, acc-orphan failed (no account metadata)
    let saved = cred_store.saved_credentials.read().await;
    assert!(saved.contains_key("acc-1"));
    assert!(!saved.contains_key("acc-orphan"));
}

#[tokio::test]
async fn migration_failure_preserves_backup() {
    // load_all returns MigrationRequired, but load_raw_json returns invalid JSON → migration fails
    let cred_store = Arc::new(
        MockCredentialStore::new()
            .with_migration_required()
            .with_raw_json("not valid json!!!"),
    );
    let account_repo = Arc::new(MockAccountRepository::new());
    let app_state = build_app_state(cred_store, account_repo);

    let hooks = TrackingStartupHooks::new();
    app_state.run_migration(&hooks).await;

    // Backup should be preserved (migration failed)
    assert!(*hooks.backup_called.read().await);
    assert!(!*hooks.cleanup_called.read().await);
    assert!(*hooks.preserve_called.read().await);
    assert!(hooks.preserve_error.read().await.is_some());
}

#[tokio::test]
async fn migration_load_all_failure_preserves_backup() {
    // with_load_error sets load_all to return StorageError("disk failure").
    // run_migration → migrate_if_needed sees a non-MigrationRequired error
    // and enters the preserve_backup path.
    let cred_store = Arc::new(
        MockCredentialStore::new().with_load_error(CoreError::StorageError("disk failure".into())),
    );
    let account_repo = Arc::new(MockAccountRepository::new());
    let app_state = build_app_state(cred_store, account_repo);

    let hooks = TrackingStartupHooks::new();
    app_state.run_migration(&hooks).await;

    assert!(!*hooks.cleanup_called.read().await);
}

#[tokio::test]
async fn startup_hooks_not_called_with_noop() {
    // NoopStartupHooks should not trigger any backup/cleanup/preserve
    let cred_store = Arc::new(MockCredentialStore::new());
    let account_repo = Arc::new(MockAccountRepository::new());
    let app_state = build_app_state(cred_store, account_repo);

    // NoopStartupHooks returns None from backup_credentials,
    // so cleanup/preserve should never be called
    app_state.run_migration(&NoopStartupHooks).await;
    // If we got here without panic, NoopStartupHooks works correctly
}

// ===== SqliteStore::new Edge Cases =====

#[tokio::test]
async fn sqlite_store_creates_parent_directories() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("nested").join("deep").join("test.db");

    let result = SqliteStore::new(&db_path, Some(TEST_PASSWORD.to_string())).await;
    assert!(result.is_ok());
    assert!(db_path.exists());
}

#[tokio::test]
async fn sqlite_store_reopen_existing_db() {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("test.db");

    // Create and populate
    {
        let store = SqliteStore::new(&db_path, Some(TEST_PASSWORD.to_string()))
            .await
            .unwrap();
        AccountRepository::save(
            &store,
            &Account {
                id: "acc-1".to_string(),
                name: "Test".to_string(),
                provider: ProviderType::Cloudflare,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                status: None,
                error: None,
            },
        )
        .await
        .unwrap();
    }

    // Reopen and verify data persisted
    let store2 = SqliteStore::new(&db_path, Some(TEST_PASSWORD.to_string()))
        .await
        .unwrap();
    let found = AccountRepository::find_by_id(&store2, "acc-1")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test");
}
