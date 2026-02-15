//! Core business service layer.

mod account_service;
mod dns_service;
mod domain_metadata_service;
mod domain_service;
mod import_export_service;
mod migration_service;
mod provider_metadata_service;

pub use account_service::{AccountService, RestoreResult};
pub use dns_service::DnsService;
pub use domain_metadata_service::DomainMetadataService;
pub use domain_service::DomainService;
pub use import_export_service::ImportExportService;
pub use migration_service::{MigrationResult, MigrationService};
pub use provider_metadata_service::ProviderMetadataService;

use std::sync::Arc;

use dns_orchestrator_provider::DnsProvider;

use dns_orchestrator_provider::ProviderError;

use crate::error::{CoreError, CoreResult};
use crate::traits::{
    AccountRepository, CredentialStore, DomainMetadataRepository, ProviderRegistry,
};
use crate::types::AccountStatus;

/// Service context that holds all runtime dependencies.
///
/// The platform layer creates this context and injects platform-specific implementations.
/// Fields are intentionally accessed through getters so callers do not bypass service orchestration.
pub struct ServiceContext {
    pub(crate) credential_store: Arc<dyn CredentialStore>,
    pub(crate) account_repository: Arc<dyn AccountRepository>,
    pub(crate) provider_registry: Arc<dyn ProviderRegistry>,
    pub(crate) domain_metadata_repository: Arc<dyn DomainMetadataRepository>,
}

impl ServiceContext {
    /// Creates a new service context.
    #[must_use]
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
        provider_registry: Arc<dyn ProviderRegistry>,
        domain_metadata_repository: Arc<dyn DomainMetadataRepository>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
            provider_registry,
            domain_metadata_repository,
        }
    }

    /// Returns the credential store.
    pub fn credential_store(&self) -> &Arc<dyn CredentialStore> {
        &self.credential_store
    }

    /// Returns the account repository.
    pub fn account_repository(&self) -> &Arc<dyn AccountRepository> {
        &self.account_repository
    }

    /// Returns the provider registry.
    pub fn provider_registry(&self) -> &Arc<dyn ProviderRegistry> {
        &self.provider_registry
    }

    /// Returns the domain metadata repository.
    pub fn domain_metadata_repository(&self) -> &Arc<dyn DomainMetadataRepository> {
        &self.domain_metadata_repository
    }

    /// Returns the provider instance for an account.
    pub async fn get_provider(&self, account_id: &str) -> CoreResult<Arc<dyn DnsProvider>> {
        self.provider_registry
            .get(account_id)
            .await
            .ok_or_else(|| CoreError::AccountNotFound(account_id.to_string()))
    }

    /// Marks an account as invalid.
    ///
    /// Called when invalid credentials are detected during provider operations.
    pub async fn mark_account_invalid(&self, account_id: &str, error_msg: &str) {
        if let Err(e) = self
            .account_repository
            .update_status(
                account_id,
                AccountStatus::Error,
                Some(error_msg.to_string()),
            )
            .await
        {
            log::error!("Failed to mark account {account_id} as invalid: {e}");
            return;
        }
        log::warn!("Account {account_id} marked as invalid: {error_msg}");
    }

    /// Maps provider errors to `CoreError` and updates account status when needed.
    pub async fn handle_provider_error(&self, account_id: &str, err: ProviderError) -> CoreError {
        if let ProviderError::InvalidCredentials { .. } = &err {
            self.mark_account_invalid(account_id, "Credentials have expired")
                .await;
        }
        CoreError::Provider(err)
    }
}
