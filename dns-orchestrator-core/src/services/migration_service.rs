//! Credential format migration service (v1.7.0)
//!
//! Migrates legacy credentials
//! (`HashMap<String, HashMap<String, String>>`)
//! into typed `ProviderCredentials`.

use dns_orchestrator_provider::{ProviderCredentials, ProviderType};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{CoreError, CoreResult};
use crate::traits::{AccountRepository, CredentialStore};

/// Service for credential format migration.
pub struct MigrationService {
    credential_store: Arc<dyn CredentialStore>,
    account_repository: Arc<dyn AccountRepository>,
}

impl MigrationService {
    /// Creates a migration service.
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        account_repository: Arc<dyn AccountRepository>,
    ) -> Self {
        Self {
            credential_store,
            account_repository,
        }
    }

    /// Checks credential format and runs migration if needed.
    ///
    /// Returns [`MigrationResult::NotNeeded`] when data is already in the new format.
    pub async fn migrate_if_needed(&self) -> CoreResult<MigrationResult> {
        // Attempt to load credentials using the new format path.
        match self.credential_store.load_all().await {
            Ok(_) => {
                log::info!("Credentials are already in the new format; migration not needed");
                Ok(MigrationResult::NotNeeded)
            }
            Err(CoreError::MigrationRequired) => {
                log::info!("Legacy credential format detected, starting migration...");
                self.perform_migration().await
            }
            Err(e) => Err(e),
        }
    }

    /// Executes migration from legacy JSON format to typed credentials.
    ///
    /// Backup logic is implemented in the Tauri layer (`src-tauri/src/lib.rs`)
    /// and should run before calling `migrate_if_needed()`.
    async fn perform_migration(&self) -> CoreResult<MigrationResult> {
        // 1. Load legacy raw JSON (backup should already exist).
        let raw_json = self.credential_store.load_raw_json().await?;

        // 2. Parse legacy credential format.
        let old_creds: HashMap<String, HashMap<String, String>> = serde_json::from_str(&raw_json)
            .map_err(|e| {
            CoreError::MigrationFailed(format!("Failed to parse legacy format: {e}"))
        })?;

        if old_creds.is_empty() {
            log::info!("Legacy credentials are empty; migration not needed");
            return Ok(MigrationResult::NotNeeded);
        }

        // 3. Load account -> provider mapping.
        let accounts = self.account_repository.find_all().await?;
        let account_providers: HashMap<String, ProviderType> =
            accounts.into_iter().map(|a| (a.id, a.provider)).collect();

        // 4. Convert legacy credential maps to typed credentials.
        let mut new_creds = HashMap::new();
        let mut failed_accounts = Vec::new();

        for (account_id, old_cred_map) in old_creds {
            if let Some(provider) = account_providers.get(&account_id) {
                match ProviderCredentials::from_map(provider, &old_cred_map) {
                    Ok(provider_creds) => {
                        new_creds.insert(account_id.clone(), provider_creds);
                    }
                    Err(e) => {
                        log::warn!("Failed to convert credentials for account {account_id}: {e}");
                        failed_accounts.push((account_id, format!("Conversion failed: {e}")));
                    }
                }
            } else {
                log::warn!("Account metadata not found for {account_id}, skipping migration");
                failed_accounts.push((account_id, "Missing account metadata".to_string()));
            }
        }

        // 5. Save converted credentials using the new storage shape.
        if !new_creds.is_empty() {
            self.credential_store.save_all(&new_creds).await?;
            log::info!(
                "Credential migration completed: {} succeeded, {} failed",
                new_creds.len(),
                failed_accounts.len()
            );
        }

        Ok(MigrationResult::Success {
            migrated_count: new_creds.len(),
            failed_accounts,
        })
    }

    // `backup_credentials` was intentionally removed.
    // Backup is handled in the Tauri layer (`src-tauri/src/lib.rs`) because
    // this core service is platform-agnostic and must not access the file system.
}

/// Migration result.
#[derive(Debug)]
pub enum MigrationResult {
    /// No migration required (already migrated or empty data).
    NotNeeded,

    /// Migration completed.
    Success {
        /// Number of accounts migrated successfully.
        migrated_count: usize,
        /// Failed account list as (`account_id`, `error_reason`).
        failed_accounts: Vec<(String, String)>,
    },
}
