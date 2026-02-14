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
    /// Maximum allowed store file size in bytes.
    max_store_file_size: u64,
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
        Self::new_with_limits(Self::get_store_path(), MAX_STORE_FILE_SIZE)
    }

    fn new_with_limits(store_path: PathBuf, max_store_file_size: u64) -> Self {
        tracing::debug!("Tauri store path: {:?}", store_path);
        Self {
            store_path,
            max_store_file_size,
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
        let metadata = tokio::fs::metadata(&store_file).await.map_err(|e| {
            CoreError::StorageError(format!("Failed to read store file metadata: {e}"))
        })?;

        if metadata.len() > self.max_store_file_size {
            return Err(CoreError::StorageError(format!(
                "Store file too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.max_store_file_size
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

        let accounts = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Failed to load accounts cache".to_string()))?;

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
        tracing::debug!(
            "Saved {} accounts to cache (read-only mode)",
            accounts.len()
        );
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

        let accounts = cache
            .as_mut()
            .ok_or_else(|| CoreError::StorageError("Failed to load accounts cache".to_string()))?;

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use dns_orchestrator_core::types::ProviderType;

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_account(account_id: &str) -> Account {
        Account {
            id: account_id.to_string(),
            name: format!("Account {account_id}"),
            provider: ProviderType::Cloudflare,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: Some(AccountStatus::Active),
            error: None,
        }
    }

    fn unique_test_dir() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("dns-orchestrator-mcp-account-repo-{timestamp}-{counter}"))
    }

    async fn write_store_json(store_dir: &Path, value: serde_json::Value) {
        tokio::fs::create_dir_all(store_dir).await.unwrap();
        let store_file = store_dir.join(STORE_FILE_NAME);
        tokio::fs::write(store_file, serde_json::to_string(&value).unwrap())
            .await
            .unwrap();
    }

    async fn read_store_raw(store_dir: &Path) -> String {
        let store_file = store_dir.join(STORE_FILE_NAME);
        tokio::fs::read_to_string(store_file).await.unwrap()
    }

    #[tokio::test]
    async fn find_all_returns_empty_when_store_file_missing() {
        let store_dir = unique_test_dir();
        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);

        let accounts = repo.find_all().await.unwrap();
        assert!(accounts.is_empty());

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn find_all_returns_empty_when_accounts_key_missing() {
        let store_dir = unique_test_dir();
        write_store_json(&store_dir, serde_json::json!({ "other": [] })).await;

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        let accounts = repo.find_all().await.unwrap();
        assert!(accounts.is_empty());

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn find_all_returns_serialization_error_for_invalid_json() {
        let store_dir = unique_test_dir();
        tokio::fs::create_dir_all(&store_dir).await.unwrap();
        tokio::fs::write(store_dir.join(STORE_FILE_NAME), "{invalid json")
            .await
            .unwrap();

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        let error = repo.find_all().await.unwrap_err();
        assert!(matches!(error, CoreError::SerializationError(_)));

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn find_all_rejects_oversized_store_file() {
        let store_dir = unique_test_dir();
        tokio::fs::create_dir_all(&store_dir).await.unwrap();
        tokio::fs::write(store_dir.join(STORE_FILE_NAME), "0123456789ABCDEF")
            .await
            .unwrap();

        let repo = TauriStoreAccountRepository::new_with_limits(store_dir.clone(), 8);
        let error = repo.find_all().await.unwrap_err();
        assert!(matches!(error, CoreError::StorageError(_)));

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn find_all_uses_cache_after_first_read() {
        let store_dir = unique_test_dir();

        let first_account = test_account("acc-1");
        write_store_json(
            &store_dir,
            serde_json::json!({ "accounts": [serde_json::to_value(&first_account).unwrap()] }),
        )
        .await;

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        let first = repo.find_all().await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].id, "acc-1");

        let second_account = test_account("acc-2");
        write_store_json(
            &store_dir,
            serde_json::json!({ "accounts": [serde_json::to_value(&second_account).unwrap()] }),
        )
        .await;

        let second = repo.find_all().await.unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id, "acc-1");

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn save_updates_cache_without_persisting_to_disk() {
        let store_dir = unique_test_dir();
        write_store_json(&store_dir, serde_json::json!({ "accounts": [] })).await;

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        repo.save(&test_account("acc-1")).await.unwrap();

        let accounts = repo.find_all().await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].id, "acc-1");

        let raw = read_store_raw(&store_dir).await;
        let store_value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(store_value[ACCOUNTS_KEY], serde_json::json!([]));

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn save_all_and_update_status_only_change_cache() {
        let store_dir = unique_test_dir();
        write_store_json(&store_dir, serde_json::json!({ "accounts": [] })).await;

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        repo.save_all(&[test_account("acc-1")]).await.unwrap();

        let updated_error = "provider auth failed".to_string();
        repo.update_status("acc-1", AccountStatus::Error, Some(updated_error.clone()))
            .await
            .unwrap();

        let accounts = repo.find_all().await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].status, Some(AccountStatus::Error));
        assert_eq!(accounts[0].error, Some(updated_error));

        let raw = read_store_raw(&store_dir).await;
        let store_value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(store_value[ACCOUNTS_KEY], serde_json::json!([]));

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }

    #[tokio::test]
    async fn update_status_returns_not_found_for_missing_account() {
        let store_dir = unique_test_dir();
        write_store_json(&store_dir, serde_json::json!({ "accounts": [] })).await;

        let repo =
            TauriStoreAccountRepository::new_with_limits(store_dir.clone(), MAX_STORE_FILE_SIZE);
        let error = repo
            .update_status("missing", AccountStatus::Error, Some("oops".to_string()))
            .await
            .unwrap_err();

        assert!(matches!(error, CoreError::AccountNotFound(_)));

        let _ = tokio::fs::remove_dir_all(store_dir).await;
    }
}
