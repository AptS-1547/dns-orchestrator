//! Tauri Store-based account repository (Read-Only)
//!
//! Reads account data from Tauri store files created by the desktop app.
//! This is a read-only implementation - all write operations only update the
//! in-memory cache without persisting to disk.

use async_trait::async_trait;
use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::AccountRepository;
use dns_orchestrator_core::types::{Account, AccountStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const STORE_FILE_NAME: &str = "accounts.json";
const ACCOUNTS_KEY: &str = "accounts";
const MAX_STORE_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

/// Account repository that reads from Tauri store files.
///
/// This implementation reads account metadata from the same store files
/// used by the Tauri desktop application, allowing the MCP server to
/// share account information with the desktop app.
///
/// **Read-Only Mode**: All write operations (`save`, `delete`, `save_all`,
/// `update_status`) only update the in-memory cache and do not persist
/// changes to disk. This prevents the MCP server from modifying the
/// desktop app's data.
pub struct TauriStoreAccountRepository {
    /// Path to the Tauri store directory.
    store_path: PathBuf,
    /// In-memory cache.
    cache: Arc<RwLock<Option<Vec<Account>>>>,
}

impl TauriStoreAccountRepository {
    /// Create a new read-only account repository instance.
    ///
    /// Automatically detects the platform-specific Tauri store location:
    /// - macOS: `~/Library/Application Support/com.apts-1547.dns-orchestrator/`
    /// - Windows: `%APPDATA%/com.apts-1547.dns-orchestrator/`
    /// - Linux: `~/.local/share/com.apts-1547.dns-orchestrator/`
    #[must_use]
    pub fn new() -> Self {
        let store_path = Self::get_store_path();
        tracing::debug!("Tauri store path: {:?}", store_path);

        Self {
            store_path,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the platform-specific Tauri store directory.
    fn get_store_path() -> PathBuf {
        dirs::data_local_dir()
            .expect("Failed to determine data directory - unsupported platform or environment")
            .join("com.apts-1547.dns-orchestrator")
    }

    /// Get the full path to the accounts store file.
    fn get_store_file_path(&self) -> PathBuf {
        self.store_path.join(STORE_FILE_NAME)
    }

    /// Load accounts from the store file.
    async fn load_from_store(&self) -> CoreResult<Vec<Account>> {
        let store_file = self.get_store_file_path();

        if !store_file.exists() {
            tracing::debug!("Store file does not exist: {:?}", store_file);
            return Ok(Vec::new());
        }

        // Check file size before reading
        let metadata = tokio::fs::metadata(&store_file)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to read store file metadata: {e}")))?;

        if metadata.len() > MAX_STORE_FILE_SIZE {
            return Err(CoreError::StorageError(format!(
                "Store file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                MAX_STORE_FILE_SIZE
            )));
        }

        let content = tokio::fs::read_to_string(&store_file)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to read store file: {e}")))?;

        let store_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| CoreError::SerializationError(format!("Invalid store format: {e}")))?;

        let accounts_value = store_value
            .get(ACCOUNTS_KEY)
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        if accounts_value.is_null() {
            return Ok(Vec::new());
        }

        serde_json::from_value(accounts_value)
            .map_err(|e| CoreError::SerializationError(format!("Invalid accounts format: {e}")))
    }
}

impl Default for TauriStoreAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for TauriStoreAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().await;
            if let Some(ref accounts) = *cache {
                return Ok(accounts.clone());
            }
        }

        // Cache empty, acquire write lock and load (double-check)
        let mut cache = self.cache.write().await;
        if let Some(ref accounts) = *cache {
            return Ok(accounts.clone());
        }

        let accounts = self.load_from_store().await?;
        *cache = Some(accounts.clone());
        Ok(accounts)
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        let accounts = self.find_all().await?;
        Ok(accounts.iter().find(|a| a.id == id).cloned())
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        // Read-only mode: only update in-memory cache
        let mut cache = self.cache.write().await;

        // Ensure cache is loaded
        if cache.is_none() {
            drop(cache); // Release write lock before loading
            let loaded = self.load_from_store().await?;
            cache = self.cache.write().await;
            *cache = Some(loaded);
        }

        let accounts = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Failed to load accounts cache".to_string())
        })?;

        // Find and update or insert (in-memory only)
        if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
            accounts[pos] = account.clone();
        } else {
            accounts.push(account.clone());
        }

        tracing::debug!("Account {} updated in cache (read-only mode)", account.id);
        Ok(())
    }

    async fn delete(&self, _id: &str) -> CoreResult<()> {
        // Read-only mode: silently succeed without deleting
        tracing::debug!("Delete operation ignored (read-only mode)");
        Ok(())
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        // Read-only mode: only update in-memory cache
        let mut cache = self.cache.write().await;
        *cache = Some(accounts.to_vec());
        tracing::debug!("Saved {} accounts to cache (read-only mode)", accounts.len());
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        // Read-only mode: only update in-memory cache
        let mut cache = self.cache.write().await;

        // Ensure cache is loaded
        if cache.is_none() {
            drop(cache); // Release write lock before loading
            let loaded = self.load_from_store().await?;
            cache = self.cache.write().await;
            *cache = Some(loaded);
        }

        let accounts = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Failed to load accounts cache".to_string())
        })?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| CoreError::AccountNotFound(id.to_string()))?;

        account.status = Some(status);
        account.error = error;
        account.updated_at = chrono::Utc::now();

        tracing::debug!("Account {} status updated in cache (read-only mode)", id);
        Ok(())
    }
}
