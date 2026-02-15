//! Account persistence abstraction.

use async_trait::async_trait;

use crate::error::CoreResult;
use crate::types::{Account, AccountStatus};

/// Repository for account metadata.
///
/// Platform implementations:
/// - Tauri: `TauriAccountRepository` (tauri-plugin-store)
/// - Actix-Web: `DatabaseAccountRepository` (`SeaORM`)
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Returns all accounts.
    async fn find_all(&self) -> CoreResult<Vec<Account>>;

    /// Returns an account by ID.
    ///
    /// # Arguments
    /// * `id` - Account ID.
    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>>;

    /// Saves an account (insert or update).
    ///
    /// # Arguments
    /// * `account` - Account data.
    async fn save(&self, account: &Account) -> CoreResult<()>;

    /// Deletes an account.
    ///
    /// # Arguments
    /// * `id` - Account ID.
    async fn delete(&self, id: &str) -> CoreResult<()>;

    /// Saves accounts in batch (used by import).
    ///
    /// # Arguments
    /// * `accounts` - Account list.
    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()>;

    /// Updates account status.
    ///
    /// # Arguments
    /// * `id` - Account ID.
    /// * `status` - New status.
    /// * `error` - Optional error message when status is `Error`.
    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()>;
}
