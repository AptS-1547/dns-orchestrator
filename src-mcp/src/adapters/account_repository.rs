//! Tauri Store-based account repository
//!
//! Reads account data from Tauri store files created by the desktop app.

use async_trait::async_trait;
use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::AccountRepository;
use dns_orchestrator_core::types::{Account, AccountStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const STORE_FILE_NAME: &str = "accounts.json";
const ACCOUNTS_KEY: &str = "accounts";

/// Account repository that reads from Tauri store files.
///
/// This implementation reads account metadata from the same store files
/// used by the Tauri desktop application, allowing the MCP server to
/// share account information with the desktop app.
pub struct TauriStoreAccountRepository {
    /// Path to the Tauri store directory.
    store_path: PathBuf,
    /// In-memory cache.
    cache: Arc<RwLock<Option<Vec<Account>>>>,
}

impl TauriStoreAccountRepository {
    /// Create a new account repository instance.
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
            .unwrap_or_else(|| PathBuf::from("."))
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

    /// Save accounts to the store file.
    async fn save_to_store(&self, accounts: &[Account]) -> CoreResult<()> {
        // Ensure directory exists
        if !self.store_path.exists() {
            tokio::fs::create_dir_all(&self.store_path)
                .await
                .map_err(|e| CoreError::StorageError(format!("Failed to create store directory: {e}")))?;
        }

        let store_file = self.get_store_file_path();

        // Load existing store to preserve other data
        let mut store_value: serde_json::Value = if store_file.exists() {
            let content = tokio::fs::read_to_string(&store_file)
                .await
                .unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // Update accounts
        store_value[ACCOUNTS_KEY] = serde_json::to_value(accounts)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let content = serde_json::to_string_pretty(&store_value)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        tokio::fs::write(&store_file, content)
            .await
            .map_err(|e| CoreError::StorageError(format!("Failed to write store file: {e}")))?;

        tracing::debug!("Saved {} accounts to store", accounts.len());
        Ok(())
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
        let mut cache = self.cache.write().await;

        // Ensure cache is loaded
        if cache.is_none() {
            let loaded = self.load_from_store().await?;
            *cache = Some(loaded);
        }

        let accounts = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Failed to load accounts cache".to_string())
        })?;

        // Find and update or insert
        if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
            accounts[pos] = account.clone();
        } else {
            accounts.push(account.clone());
        }

        self.save_to_store(accounts).await
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        // Ensure cache is loaded
        if cache.is_none() {
            let loaded = self.load_from_store().await?;
            *cache = Some(loaded);
        }

        let accounts = cache.as_mut().ok_or_else(|| {
            CoreError::StorageError("Failed to load accounts cache".to_string())
        })?;

        let initial_len = accounts.len();
        accounts.retain(|a| a.id != id);

        if accounts.len() == initial_len {
            return Err(CoreError::AccountNotFound(id.to_string()));
        }

        self.save_to_store(accounts).await?;
        tracing::info!("Deleted account {id} from store");
        Ok(())
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        self.save_to_store(accounts).await?;

        let mut cache = self.cache.write().await;
        *cache = Some(accounts.to_vec());
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let mut cache = self.cache.write().await;

        // Ensure cache is loaded
        if cache.is_none() {
            let loaded = self.load_from_store().await?;
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

        self.save_to_store(accounts).await
    }
}
