//! Platform-agnostic application bootstrap for DNS Orchestrator.
//!
//! Provides `AppState` (service container), `AppStateBuilder` (adapter injection),
//! `StartupHooks` (platform-specific startup callbacks), and generic file-based
//! storage adapters for non-Tauri frontends.
//!
//! # Startup Lifecycle
//! 1. `AppState::run_migration` checks credential format and performs migration if needed.
//! 2. `AppState::run_account_restore` restores account runtime state from stored credentials.
//! 3. `AppState::restore_completed` is set to `true` after restore finishes.
//!
//! # Feature Flags
//! - `keyring-store`: enables `adapters::KeyringCredentialStore`.
//! - `sqlite-store`: enables `adapters::SqliteStore`.

pub mod adapters;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::services::{
    AccountService, DnsService, DomainMetadataService, DomainService, ImportExportService,
    MigrationResult, MigrationService, ProviderMetadataService, ServiceContext,
};
use dns_orchestrator_core::traits::{
    AccountRepository, CredentialStore, DomainMetadataRepository, InMemoryProviderRegistry,
    ProviderRegistry,
};
use dns_orchestrator_core::types::AccountStatus;

/// Platform-specific hooks for the startup sequence.
///
/// Frontends implement this to handle credential backup before migration.
/// Use `NoopStartupHooks` if no backup is needed (e.g., database-backed storage).
///
/// The `backup_info` returned by [`StartupHooks::backup_credentials`] is passed to
/// [`StartupHooks::cleanup_backup`] on success paths and to
/// [`StartupHooks::preserve_backup`] on migration failure.
#[async_trait::async_trait]
pub trait StartupHooks: Send + Sync {
    /// Called before migration to backup credentials.
    ///
    /// Returns a backup identifier (e.g., file path) or `None` to skip backup.
    async fn backup_credentials(&self, _raw_json: &str) -> Option<String> {
        None
    }

    /// Called after migration succeeds (or is not needed) to clean up the backup.
    async fn cleanup_backup(&self, _backup_info: &str) {}

    /// Called when migration fails, to preserve the backup for manual recovery.
    async fn preserve_backup(&self, _backup_info: &str, _error: &str) {}
}

/// No-op startup hooks for frontends that don't need credential backup.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopStartupHooks;

#[async_trait::async_trait]
impl StartupHooks for NoopStartupHooks {}

/// Platform-agnostic application state.
///
/// Holds all services and the `ServiceContext`. Every frontend constructs this
/// once at startup via `AppStateBuilder`.
pub struct AppState {
    /// Shared service context containing all repository/provider adapters.
    pub ctx: Arc<ServiceContext>,
    /// Service for account CRUD, validation, and restoration.
    pub account_service: Arc<AccountService>,
    /// Service for provider metadata (capabilities and provider information).
    pub provider_metadata_service: ProviderMetadataService,
    /// Service for importing/exporting account snapshots.
    pub import_export_service: ImportExportService,
    /// Service for domain listing and domain-level orchestration logic.
    pub domain_service: DomainService,
    /// Service for per-domain UI metadata (favorite/tag/color/note).
    pub domain_metadata_service: Arc<DomainMetadataService>,
    /// Service for DNS record operations.
    pub dns_service: DnsService,
    /// Whether account restoration has completed for this process lifecycle.
    pub restore_completed: AtomicBool,
}

impl AppState {
    /// Run the full startup sequence: migration → account restoration.
    ///
    /// This method always completes migration first, then account restoration.
    /// It returns `CoreResult` for forward compatibility with future startup
    /// stages that may become fallible.
    pub async fn run_startup(&self, hooks: &dyn StartupHooks) -> CoreResult<()> {
        self.run_migration(hooks).await;
        self.run_account_restore().await;
        Ok(())
    }

    /// Run credential migration.
    ///
    /// This should be called before the app is ready to serve requests.
    /// Failed accounts are marked as `AccountStatus::Error`.
    ///
    /// Migration failures are logged instead of being returned so startup can
    /// continue to the restore stage.
    pub async fn run_migration(&self, hooks: &dyn StartupHooks) {
        // 1. Backup
        let backup_info = match self.ctx.credential_store().load_raw_json().await {
            Ok(raw_json) => hooks.backup_credentials(&raw_json).await,
            Err(e) => {
                log::warn!("Failed to load raw credentials for backup: {e}");
                None
            }
        };

        // 2. Migrate
        let migration_service = MigrationService::new(
            Arc::clone(self.ctx.credential_store()),
            Arc::clone(self.ctx.account_repository()),
        );

        match migration_service.migrate_if_needed().await {
            Ok(MigrationResult::NotNeeded) => {
                log::info!("Credential format check complete: no migration needed");
                if let Some(ref info) = backup_info {
                    hooks.cleanup_backup(info).await;
                }
            }
            Ok(MigrationResult::Success {
                migrated_count,
                failed_accounts,
            }) => {
                log::info!("Credential migration succeeded: migrated {migrated_count} account(s)");
                if !failed_accounts.is_empty() {
                    log::warn!(
                        "Credential migration partially failed ({} account(s)): {:?}",
                        failed_accounts.len(),
                        failed_accounts
                    );
                    for (account_id, error_msg) in &failed_accounts {
                        if let Err(e) = self
                            .account_service
                            .update_status(
                                account_id,
                                AccountStatus::Error,
                                Some(format!("Credential migration failed: {error_msg}")),
                            )
                            .await
                        {
                            log::error!("Failed to update status for account {account_id}: {e}");
                        }
                    }
                }
                if let Some(ref info) = backup_info {
                    hooks.cleanup_backup(info).await;
                }
            }
            Err(e) => {
                log::error!("Credential migration failed: {e}");
                if let Some(ref info) = backup_info {
                    hooks.preserve_backup(info, &e.to_string()).await;
                }
            }
        }
    }

    /// Run account restoration and set `restore_completed` to `true` when done.
    ///
    /// Restoration errors are logged and do not panic.
    pub async fn run_account_restore(&self) {
        match self.account_service.restore_accounts().await {
            Ok(result) => {
                log::info!(
                    "Account restoration complete: {} succeeded, {} failed",
                    result.success_count,
                    result.error_count
                );
            }
            Err(e) => {
                log::error!("Failed to restore accounts: {e}");
            }
        }
        self.restore_completed.store(true, Ordering::SeqCst);
    }
}

/// Builder for constructing `AppState` with platform-specific adapters.
///
/// The builder validates required adapters, then wires all core services with
/// a shared `ServiceContext`.
///
/// # Required adapters
/// - `credential_store` — how credentials are stored
/// - `account_repository` — how account metadata is stored
/// - `domain_metadata_repository` — how domain metadata is stored
///
/// # Optional
/// - `provider_registry` — defaults to `InMemoryProviderRegistry`
pub struct AppStateBuilder {
    credential_store: Option<Arc<dyn CredentialStore>>,
    account_repository: Option<Arc<dyn AccountRepository>>,
    provider_registry: Option<Arc<dyn ProviderRegistry>>,
    domain_metadata_repository: Option<Arc<dyn DomainMetadataRepository>>,
}

impl AppStateBuilder {
    /// Create an empty builder.
    ///
    /// Required adapters must be injected before calling [`Self::build`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            credential_store: None,
            account_repository: None,
            provider_registry: None,
            domain_metadata_repository: None,
        }
    }

    /// Set the credential storage adapter.
    #[must_use]
    pub fn credential_store(mut self, store: Arc<dyn CredentialStore>) -> Self {
        self.credential_store = Some(store);
        self
    }

    /// Set the account metadata repository adapter.
    #[must_use]
    pub fn account_repository(mut self, repo: Arc<dyn AccountRepository>) -> Self {
        self.account_repository = Some(repo);
        self
    }

    /// Override the provider registry implementation.
    ///
    /// If not provided, an [`InMemoryProviderRegistry`] will be used.
    #[must_use]
    pub fn provider_registry(mut self, registry: Arc<dyn ProviderRegistry>) -> Self {
        self.provider_registry = Some(registry);
        self
    }

    /// Set the domain metadata repository adapter.
    #[must_use]
    pub fn domain_metadata_repository(mut self, repo: Arc<dyn DomainMetadataRepository>) -> Self {
        self.domain_metadata_repository = Some(repo);
        self
    }

    /// Build the `AppState`.
    ///
    /// # Errors
    /// Returns `CoreError::ValidationError` if required adapters are missing.
    ///
    /// # Behavior
    /// If no provider registry is injected, this method uses
    /// `InMemoryProviderRegistry`.
    pub fn build(self) -> CoreResult<AppState> {
        let credential_store = self.credential_store.ok_or_else(|| {
            CoreError::ValidationError("credential_store is required".to_string())
        })?;
        let account_repository = self.account_repository.ok_or_else(|| {
            CoreError::ValidationError("account_repository is required".to_string())
        })?;
        let provider_registry = self
            .provider_registry
            .unwrap_or_else(|| Arc::new(InMemoryProviderRegistry::new()));
        let domain_metadata_repository = self.domain_metadata_repository.ok_or_else(|| {
            CoreError::ValidationError("domain_metadata_repository is required".to_string())
        })?;

        let ctx = Arc::new(ServiceContext::new(
            credential_store,
            account_repository,
            provider_registry,
            domain_metadata_repository.clone(),
        ));

        let account_service = Arc::new(AccountService::new(Arc::clone(&ctx)));
        let provider_metadata_service = ProviderMetadataService::new();
        let import_export_service = ImportExportService::new(Arc::clone(&account_service));
        let domain_metadata_service =
            Arc::new(DomainMetadataService::new(domain_metadata_repository));
        let domain_service =
            DomainService::new(Arc::clone(&ctx), Arc::clone(&domain_metadata_service));
        let dns_service = DnsService::new(Arc::clone(&ctx));

        Ok(AppState {
            ctx,
            account_service,
            provider_metadata_service,
            import_export_service,
            domain_service,
            domain_metadata_service,
            dns_service,
            restore_completed: AtomicBool::new(false),
        })
    }
}

impl Default for AppStateBuilder {
    /// Equivalent to [`AppStateBuilder::new`].
    fn default() -> Self {
        Self::new()
    }
}
