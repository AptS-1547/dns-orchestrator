//! Unified account service
//!
//! Consolidates the responsibilities of:
//! `AccountMetadataService`, `CredentialManagementService`,
//! `AccountLifecycleService`, and `AccountBootstrapService`.

use std::sync::Arc;

use chrono::Utc;
use dns_orchestrator_provider::{DnsProvider, ProviderCredentials, ProviderType, create_provider};

use crate::error::{CoreError, CoreResult};
use crate::services::ServiceContext;
use crate::traits::CredentialsMap;
use crate::types::{
    Account, AccountStatus, BatchDeleteFailure, BatchDeleteResult, CreateAccountRequest,
    UpdateAccountRequest,
};

/// Account restore result summary.
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Number of accounts restored successfully.
    pub success_count: usize,
    /// Number of accounts that failed to restore.
    pub error_count: usize,
}

/// Unified account management service.
pub struct AccountService {
    ctx: Arc<ServiceContext>,
}

impl AccountService {
    /// Creates an account service.
    #[must_use]
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    // ===== CRUD operations =====

    /// Lists all accounts.
    pub async fn list_accounts(&self) -> CoreResult<Vec<Account>> {
        self.ctx.account_repository().find_all().await
    }

    /// Returns one account by ID.
    pub async fn get_account(&self, account_id: &str) -> CoreResult<Option<Account>> {
        self.ctx.account_repository().find_by_id(account_id).await
    }

    /// Updates account status.
    pub async fn update_status(
        &self,
        account_id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        self.ctx
            .account_repository()
            .update_status(account_id, status, error)
            .await
    }

    // ===== Lifecycle operations =====

    /// Creates an account.
    ///
    /// Workflow:
    /// verify credentials -> save credentials -> register provider -> save metadata.
    ///
    /// If metadata save fails, saved credentials and registered providers are rolled back.
    pub async fn create_account(&self, request: CreateAccountRequest) -> CoreResult<Account> {
        // 1. Validate credentials.
        let provider = self
            .validate_and_create_provider(&request.credentials)
            .await?;

        // 2. Generate account ID.
        let account_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // 3. Persist credentials.
        log::info!("Saving credentials for account: {account_id}");
        self.save_credentials(&account_id, &request.credentials)
            .await?;
        log::info!("Credentials saved successfully");

        // 4. Register provider instance.
        self.register_provider(account_id.clone(), provider).await;

        // 5. Build account metadata.
        let account = Account {
            id: account_id.clone(),
            name: request.name,
            provider: request.provider,
            created_at: now,
            updated_at: now,
            status: Some(AccountStatus::Active),
            error: None,
        };

        // 6. Persist metadata; rollback side effects on failure.
        if let Err(e) = self.ctx.account_repository().save(&account).await {
            log::error!("Failed to save account metadata, cleaning up: {e}");
            if let Err(cleanup_err) = self.delete_credentials(&account_id).await {
                log::warn!("Cleanup: failed to delete credentials for {account_id}: {cleanup_err}");
            }
            self.unregister_provider(&account_id).await;
            return Err(e);
        }

        Ok(account)
    }

    /// Creates an account from imported data (skips credential validation).
    pub async fn create_account_from_import(
        &self,
        name: String,
        provider_type: ProviderType,
        credentials: ProviderCredentials,
    ) -> CoreResult<Account> {
        let provider = create_provider(credentials.clone())?;
        let account_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        self.save_credentials(&account_id, &credentials).await?;
        self.register_provider(account_id.clone(), provider).await;

        let account = Account {
            id: account_id.clone(),
            name,
            provider: provider_type,
            created_at: now,
            updated_at: now,
            status: Some(AccountStatus::Active),
            error: None,
        };

        if let Err(e) = self.ctx.account_repository().save(&account).await {
            log::error!("Failed to save account metadata, cleaning up: {e}");
            if let Err(cleanup_err) = self.delete_credentials(&account_id).await {
                log::warn!("Cleanup: failed to delete credentials for {account_id}: {cleanup_err}");
            }
            self.unregister_provider(&account_id).await;
            return Err(e);
        }

        Ok(account)
    }

    /// Update account
    ///
    /// Supports updating account names and/or credentials.
    /// If the credentials are updated, the provider is revalidated and reregistered.
    pub async fn update_account(&self, request: UpdateAccountRequest) -> CoreResult<Account> {
        // 1. Load the existing account.
        let mut account = self
            .ctx
            .account_repository()
            .find_by_id(&request.id)
            .await?
            .ok_or_else(|| CoreError::AccountNotFound(request.id.clone()))?;

        // 2. If credentials are provided, validate and update them.
        let old_credentials = if let Some(ref new_credentials) = request.credentials {
            let new_provider = self.validate_and_create_provider(new_credentials).await?;

            // Keep old credentials for rollback.
            let old_creds = self.load_credentials(&request.id).await.ok();

            log::info!("Updating credentials for account: {}", request.id);
            self.save_credentials(&request.id, new_credentials).await?;
            self.register_provider(request.id.clone(), new_provider)
                .await;

            account.status = Some(AccountStatus::Active);
            account.error = None;

            old_creds
        } else {
            None
        };

        // 3. Update name if provided.
        if let Some(new_name) = request.name {
            account.name = new_name;
        }

        // 4. Refresh timestamp.
        account.updated_at = Utc::now();

        // 5. Persist account and rollback credentials/provider if this fails.
        if let Err(e) = self.ctx.account_repository().save(&account).await {
            if let Some(old_creds) = old_credentials {
                log::warn!("Rolling back credentials for account: {}", request.id);
                if let Err(rollback_err) = self.save_credentials(&request.id, &old_creds).await {
                    log::error!(
                        "Failed to rollback credentials for {}: {rollback_err}",
                        request.id
                    );
                }
                // Roll back provider registration.
                if let Ok(old_provider) = create_provider(old_creds) {
                    self.register_provider(request.id.clone(), old_provider)
                        .await;
                }
            }
            return Err(e);
        }

        Ok(account)
    }

    /// Delete account
    ///
    /// Order of operations:
    /// - Reversible operations first.
    /// - Irreversible operations last.
    ///
    /// Credential deletion failure aborts the whole operation, so users can retry safely.
    /// Domain metadata cleanup failure is non-fatal and logged as warning.
    /// Account metadata deletion is executed last.
    pub async fn delete_account(&self, account_id: &str) -> CoreResult<()> {
        // 1. Ensure account exists.
        self.ctx
            .account_repository()
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))?;

        // 2. Delete credentials (abort on failure to prevent orphan credentials).
        self.delete_credentials(account_id).await?;

        // 3. Clean domain metadata (soft failure).
        if let Err(e) = self
            .ctx
            .domain_metadata_repository()
            .delete_by_account(account_id)
            .await
        {
            log::warn!("Failed to delete domain metadata for {account_id}: {e}");
        }

        // 4. Unregister provider (in-memory operation).
        self.unregister_provider(account_id).await;

        // 5. Delete account metadata (irreversible).
        self.ctx.account_repository().delete(account_id).await?;

        Ok(())
    }

    /// Deletes accounts in batch and aggregates per-item failures.
    pub async fn batch_delete_accounts(
        &self,
        account_ids: Vec<String>,
    ) -> CoreResult<BatchDeleteResult> {
        let mut success_count = 0;
        let mut failures = Vec::new();

        for account_id in account_ids {
            match self.delete_account(&account_id).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    failures.push(BatchDeleteFailure {
                        record_id: account_id,
                        reason: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchDeleteResult {
            success_count,
            failed_count: failures.len(),
            failures,
        })
    }

    // ===== Credential operations =====

    /// Validates credentials and creates a provider instance.
    pub async fn validate_and_create_provider(
        &self,
        credentials: &ProviderCredentials,
    ) -> CoreResult<Arc<dyn DnsProvider>> {
        let provider = create_provider(credentials.clone())?;

        let is_valid = provider.validate_credentials().await?;
        if !is_valid {
            return Err(CoreError::InvalidCredentials(
                credentials.provider_type().to_string(),
            ));
        }

        Ok(provider)
    }

    /// Saves credentials.
    pub async fn save_credentials(
        &self,
        account_id: &str,
        credentials: &ProviderCredentials,
    ) -> CoreResult<()> {
        self.ctx
            .credential_store()
            .set(account_id, credentials)
            .await
    }

    /// Loads credentials.
    pub async fn load_credentials(&self, account_id: &str) -> CoreResult<ProviderCredentials> {
        self.ctx
            .credential_store()
            .get(account_id)
            .await?
            .ok_or_else(|| {
                CoreError::CredentialError(format!(
                    "No credentials found for account: {account_id}"
                ))
            })
    }

    /// Deletes credentials.
    pub async fn delete_credentials(&self, account_id: &str) -> CoreResult<()> {
        self.ctx.credential_store().remove(account_id).await
    }

    /// Loads all credentials.
    pub async fn load_all_credentials(&self) -> CoreResult<CredentialsMap> {
        self.ctx.credential_store().load_all().await
    }

    // ===== Provider registration =====

    /// Registers a provider in the runtime registry.
    pub async fn register_provider(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.ctx
            .provider_registry()
            .register(account_id, provider)
            .await;
    }

    /// Unregisters a provider from the runtime registry.
    pub async fn unregister_provider(&self, account_id: &str) {
        self.ctx.provider_registry().unregister(account_id).await;
    }

    // ===== Startup restore =====

    /// Restores all accounts at startup.
    pub async fn restore_accounts(&self) -> CoreResult<RestoreResult> {
        let mut success_count = 0;
        let mut error_count = 0;

        // 1. Load all account metadata.
        let accounts = self.list_accounts().await?;

        // 2. Load all credentials.
        let all_credentials = match self.load_all_credentials().await {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("Failed to load credentials: {e}");
                for account in &accounts {
                    if let Err(update_err) = self
                        .update_status(&account.id, AccountStatus::Error, Some(e.to_string()))
                        .await
                    {
                        log::warn!(
                            "Failed to update status for account {}: {update_err}",
                            account.id
                        );
                    }
                }
                return Ok(RestoreResult {
                    success_count: 0,
                    error_count: accounts.len(),
                });
            }
        };

        // 3. Restore providers account by account.
        for account in &accounts {
            let Some(credentials) = all_credentials.get(&account.id) else {
                log::warn!("No credentials found for account: {}", account.id);
                if let Err(e) = self
                    .update_status(
                        &account.id,
                        AccountStatus::Error,
                        Some("Credentials not found".to_string()),
                    )
                    .await
                {
                    log::warn!("Failed to update status for account {}: {e}", account.id);
                }
                error_count += 1;
                continue;
            };

            let provider = match create_provider(credentials.clone()) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!(
                        "Failed to create provider for account {}: {}",
                        account.id,
                        e
                    );
                    if let Err(update_err) = self
                        .update_status(
                            &account.id,
                            AccountStatus::Error,
                            Some(format!("Failed to create provider: {e}")),
                        )
                        .await
                    {
                        log::warn!(
                            "Failed to update status for account {}: {update_err}",
                            account.id
                        );
                    }
                    error_count += 1;
                    continue;
                }
            };

            self.register_provider(account.id.clone(), provider).await;

            if let Err(e) = self
                .update_status(&account.id, AccountStatus::Active, None)
                .await
            {
                log::warn!("Failed to update status for account {}: {e}", account.id);
            }

            success_count += 1;
        }

        Ok(RestoreResult {
            success_count,
            error_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_account_service, test_credentials};
    use crate::traits::{AccountRepository, CredentialStore, DomainMetadataRepository};
    use dns_orchestrator_provider::ProviderType;

    #[tokio::test]
    async fn create_account_from_import_success() {
        let (svc, account_repo, credential_store, _) = create_test_account_service();

        let account = svc
            .create_account_from_import(
                "Test CF".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();

        assert_eq!(account.name, "Test CF");
        assert_eq!(account.provider, ProviderType::Cloudflare);
        assert_eq!(account.status, Some(AccountStatus::Active));

        // Verify credentials were saved.
        let creds = credential_store.get(&account.id).await.unwrap();
        assert!(creds.is_some());

        // Verify account metadata was saved.
        let saved = account_repo.find_by_id(&account.id).await.unwrap();
        assert!(saved.is_some());

        // Verify provider was registered.
        let provider = svc.ctx.provider_registry().get(&account.id).await;
        assert!(provider.is_some());
    }

    #[tokio::test]
    async fn create_account_from_import_save_failure_cleanup() {
        let (svc, account_repo, credential_store, _) = create_test_account_service();

        // Configure save to fail.
        account_repo
            .set_save_error(Some("disk full".to_string()))
            .await;

        let result = svc
            .create_account_from_import(
                "Fail".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await;

        assert!(result.is_err());

        // Verify cleanup: credentials were removed.
        let all_creds = credential_store.load_all().await.unwrap();
        assert!(all_creds.is_empty());

        // Verify cleanup: provider was unregistered.
        let ids = svc.ctx.provider_registry().list_account_ids().await;
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn delete_account_success() {
        let (svc, account_repo, credential_store, _) = create_test_account_service();

        let account = svc
            .create_account_from_import(
                "To Delete".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();
        let id = account.id.clone();

        svc.delete_account(&id).await.unwrap();

        // Account metadata deleted.
        assert!(account_repo.find_by_id(&id).await.unwrap().is_none());
        // Credentials deleted.
        assert!(credential_store.get(&id).await.unwrap().is_none());
        // Provider unregistered.
        assert!(svc.ctx.provider_registry().get(&id).await.is_none());
    }

    #[tokio::test]
    async fn delete_account_not_found() {
        let (svc, _, _, _) = create_test_account_service();
        let result = svc.delete_account("nonexistent").await;
        assert!(matches!(result, Err(CoreError::AccountNotFound(_))));
    }

    #[tokio::test]
    async fn delete_account_cleans_domain_metadata() {
        use crate::types::{DomainMetadata, DomainMetadataKey};

        let (svc, _, _, domain_meta_repo) = create_test_account_service();

        let account = svc
            .create_account_from_import(
                "Meta Test".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();
        let id = account.id.clone();

        // Add metadata under this account.
        let key = DomainMetadataKey::new(id.clone(), "example.com".to_string());
        let meta = DomainMetadata {
            is_favorite: true,
            favorited_at: Some(chrono::Utc::now()),
            ..DomainMetadata::default()
        };
        domain_meta_repo.save(&key, &meta).await.unwrap();

        // Delete account.
        svc.delete_account(&id).await.unwrap();

        // Domain metadata was cleaned as well.
        let found = domain_meta_repo.find_by_key(&key).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn batch_delete_accounts_partial_failure() {
        let (svc, _, _, _) = create_test_account_service();

        let acc = svc
            .create_account_from_import(
                "Keep".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();

        let result = svc
            .batch_delete_accounts(vec![acc.id.clone(), "nonexistent".to_string()])
            .await
            .unwrap();

        assert_eq!(result.success_count, 1);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.failures[0].record_id, "nonexistent");
    }

    #[tokio::test]
    async fn list_accounts() {
        let (svc, _, _, _) = create_test_account_service();

        svc.create_account_from_import(
            "A".to_string(),
            ProviderType::Cloudflare,
            test_credentials(),
        )
        .await
        .unwrap();
        svc.create_account_from_import(
            "B".to_string(),
            ProviderType::Cloudflare,
            test_credentials(),
        )
        .await
        .unwrap();

        let accounts = svc.list_accounts().await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn get_account_found() {
        let (svc, _, _, _) = create_test_account_service();

        let acc = svc
            .create_account_from_import(
                "Find Me".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();

        let found = svc.get_account(&acc.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Find Me");
    }

    #[tokio::test]
    async fn get_account_not_found() {
        let (svc, _, _, _) = create_test_account_service();
        let found = svc.get_account("ghost").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn update_status() {
        let (svc, _, _, _) = create_test_account_service();

        let acc = svc
            .create_account_from_import(
                "Status".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();

        svc.update_status(&acc.id, AccountStatus::Error, Some("broken".to_string()))
            .await
            .unwrap();

        let updated = svc.get_account(&acc.id).await.unwrap().unwrap();
        assert_eq!(updated.status, Some(AccountStatus::Error));
        assert_eq!(updated.error, Some("broken".to_string()));
    }

    #[tokio::test]
    async fn delete_account_credential_failure_aborts() {
        let (svc, account_repo, credential_store, _) = create_test_account_service();

        let account = svc
            .create_account_from_import(
                "Cred Fail".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();
        let id = account.id.clone();

        // Configure credential deletion to fail.
        credential_store
            .set_remove_error(Some("keychain locked".to_string()))
            .await;

        // Delete should fail.
        let result = svc.delete_account(&id).await;
        assert!(result.is_err());

        // Account metadata should still exist (irreversible step not reached).
        let still_exists = account_repo.find_by_id(&id).await.unwrap();
        assert!(
            still_exists.is_some(),
            "account should still exist when credential deletion fails"
        );

        // Credentials should still exist.
        let creds = credential_store.get(&id).await.unwrap();
        assert!(creds.is_some(), "credentials should still exist");

        // Provider should still be registered.
        let provider = svc.ctx.provider_registry().get(&id).await;
        assert!(provider.is_some(), "provider should still be registered");
    }

    #[tokio::test]
    async fn delete_account_credential_success_then_completes() {
        let (svc, account_repo, credential_store, _) = create_test_account_service();

        let account = svc
            .create_account_from_import(
                "Clean Delete".to_string(),
                ProviderType::Cloudflare,
                test_credentials(),
            )
            .await
            .unwrap();
        let id = account.id.clone();

        // Delete normally.
        svc.delete_account(&id).await.unwrap();

        // Verify all related data is removed.
        assert!(account_repo.find_by_id(&id).await.unwrap().is_none());
        assert!(credential_store.get(&id).await.unwrap().is_none());
        assert!(svc.ctx.provider_registry().get(&id).await.is_none());
    }
}
