//! Tauri 凭证存储适配器
//!
//! 将同步的 keyring/stronghold 实现包装为 async trait

use async_trait::async_trait;
use dns_orchestrator_provider::ProviderCredentials;
use serde::Deserialize;
use std::collections::HashMap;

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{CredentialStore, CredentialsMap, LegacyCredentialsMap};

/// 存储格式检测枚举（用于双格式支持）
#[derive(Deserialize)]
#[serde(untagged)]
enum StorageFormat {
    V2(HashMap<String, ProviderCredentials>),     // 新格式
    V1(HashMap<String, HashMap<String, String>>), // 旧格式
}

// ============ 桌面端实现 (Keychain) ============

#[cfg(not(target_os = "android"))]
mod desktop {
    use super::{
        async_trait, CoreError, CoreResult, CredentialStore, CredentialsMap, HashMap,
        ProviderCredentials, StorageFormat,
    };
    use keyring::Entry;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    const SERVICE_NAME: &str = "dns-orchestrator";
    const CREDENTIALS_KEY: &str = "all-credentials";

    /// Tauri 桌面端凭证存储（使用系统 Keychain + 内存缓存）
    pub struct TauriCredentialStore {
        /// 内存缓存，减少 Keychain 访问频率
        cache: Arc<RwLock<Option<CredentialsMap>>>,
    }

    impl TauriCredentialStore {
        pub fn new() -> Self {
            Self {
                cache: Arc::new(RwLock::new(None)),
            }
        }

        fn get_entry() -> CoreResult<Entry> {
            Entry::new(SERVICE_NAME, CREDENTIALS_KEY)
                .map_err(|e| CoreError::CredentialError(e.to_string()))
        }

        /// 读取原始 JSON（同步方法）
        fn read_raw_sync() -> CoreResult<String> {
            let entry = Self::get_entry()?;

            match entry.get_password() {
                Ok(json) => Ok(json),
                Err(keyring::Error::NoEntry) => Ok("{}".to_string()),
                Err(e) => Err(CoreError::CredentialError(e.to_string())),
            }
        }

        /// 写入原始 JSON（同步方法）
        fn write_raw_sync(json: &str) -> CoreResult<()> {
            let entry = Self::get_entry()?;
            entry
                .set_password(json)
                .map_err(|e| CoreError::CredentialError(e.to_string()))?;
            Ok(())
        }

        /// 读取所有凭证（同步方法，支持双格式）
        fn read_all_sync() -> CoreResult<CredentialsMap> {
            let json = Self::read_raw_sync()?;

            // 尝试解析
            match serde_json::from_str::<StorageFormat>(&json) {
                Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
                Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
                Err(_) if json.trim().is_empty() || json.trim() == "{}" => Ok(HashMap::new()),
                Err(e) => Err(CoreError::SerializationError(e.to_string())),
            }
        }

        /// 写入所有凭证（同步方法）
        fn write_all_sync(credentials: &CredentialsMap) -> CoreResult<()> {
            let json = serde_json::to_string(credentials)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;
            Self::write_raw_sync(&json)
        }

        /// 更新缓存（辅助方法）
        async fn update_cache(&self, credentials: CredentialsMap) {
            let mut cache = self.cache.write().await;
            *cache = Some(credentials);
        }
    }

    impl Default for TauriCredentialStore {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl CredentialStore for TauriCredentialStore {
        async fn load_all(&self) -> CoreResult<CredentialsMap> {
            // 先检查缓存
            {
                let cache = self.cache.read().await;
                if let Some(ref creds) = *cache {
                    return Ok(creds.clone());
                }
            }

            // 从 Keychain 加载
            let credentials = tokio::task::spawn_blocking(|| {
                log::debug!("Loading all credentials from Keychain");
                Self::read_all_sync()
            })
            .await
            .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?;

            // 处理迁移错误：向上传递
            let credentials = credentials?;

            // 更新缓存
            self.update_cache(credentials.clone()).await;

            log::info!("Loaded {} accounts from Keychain", credentials.len());
            Ok(credentials)
        }

        async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
            let creds_clone = credentials.clone();
            tokio::task::spawn_blocking(move || Self::write_all_sync(&creds_clone))
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))??;

            // 更新缓存
            self.update_cache(credentials.clone()).await;

            log::info!("Saved {} accounts to Keychain", credentials.len());
            Ok(())
        }

        async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
            let all_creds = self.load_all().await?;
            Ok(all_creds.get(account_id).cloned())
        }

        async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
            let mut all_creds = self.load_all().await?;
            all_creds.insert(account_id.to_string(), credentials.clone());
            self.save_all(&all_creds).await?;

            log::info!("Credentials saved for account: {account_id}");
            Ok(())
        }

        async fn remove(&self, account_id: &str) -> CoreResult<()> {
            let mut all_creds = self.load_all().await?;
            all_creds.remove(account_id);
            self.save_all(&all_creds).await?;

            log::info!("Credentials deleted for account: {account_id}");
            Ok(())
        }

        async fn load_raw_json(&self) -> CoreResult<String> {
            tokio::task::spawn_blocking(Self::read_raw_sync)
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
        }

        async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
            let json_clone = json.to_string();
            tokio::task::spawn_blocking(move || Self::write_raw_sync(&json_clone))
                .await
                .map_err(|e| CoreError::CredentialError(format!("Task join error: {e}")))?
        }
    }
}

// ============ Android 实现 (Stronghold via Store) ============

#[cfg(target_os = "android")]
mod android {
    use super::{
        async_trait, CoreError, CoreResult, CredentialStore, CredentialsMap, HashMap,
        LegacyCredentialsMap, ProviderCredentials, StorageFormat,
    };
    use std::sync::Arc;
    use tauri::AppHandle;
    use tauri_plugin_store::StoreExt;
    use tokio::sync::RwLock;

    const STORE_FILE_NAME: &str = "credentials.json";
    const CREDENTIALS_KEY: &str = "credentials";

    /// Tauri Android 凭证存储
    ///
    /// 使用 tauri-plugin-store 将凭证持久化到应用私有目录。
    /// 注意：这不是加密存储，依赖 Android 沙箱机制保护数据。
    pub struct TauriCredentialStore {
        app_handle: AppHandle,
        cache: Arc<RwLock<Option<CredentialsMap>>>,
    }

    impl TauriCredentialStore {
        pub fn new(app_handle: AppHandle) -> Self {
            Self {
                app_handle,
                cache: Arc::new(RwLock::new(None)),
            }
        }

        /// 加载原始 JSON
        fn load_raw_from_store(&self) -> CoreResult<String> {
            let store = self
                .app_handle
                .store(STORE_FILE_NAME)
                .map_err(|e| CoreError::StorageError(format!("Failed to access store: {e}")))?;

            let Some(value) = store.get(CREDENTIALS_KEY) else {
                return Ok("{}".to_string());
            };

            serde_json::to_string(&value).map_err(|e| CoreError::SerializationError(e.to_string()))
        }

        /// 保存原始 JSON
        fn save_raw_to_store(&self, json: &str) -> CoreResult<()> {
            let store = self
                .app_handle
                .store(STORE_FILE_NAME)
                .map_err(|e| CoreError::StorageError(format!("Failed to access store: {e}")))?;

            let value: serde_json::Value = serde_json::from_str(json)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;

            store.set(CREDENTIALS_KEY.to_string(), value);
            store
                .save()
                .map_err(|e| CoreError::StorageError(format!("Failed to save store: {e}")))?;

            Ok(())
        }

        /// 从 Store 加载（支持双格式）
        fn load_from_store(&self) -> CoreResult<CredentialsMap> {
            let json = self.load_raw_from_store()?;

            // 尝试解析
            match serde_json::from_str::<StorageFormat>(&json) {
                Ok(StorageFormat::V2(new_creds)) => Ok(new_creds),
                Ok(StorageFormat::V1(_)) => Err(CoreError::MigrationRequired),
                Err(_) if json.trim().is_empty() || json.trim() == "{}" => Ok(HashMap::new()),
                Err(e) => Err(CoreError::SerializationError(e.to_string())),
            }
        }

        /// 保存到 Store
        fn save_to_store(&self, credentials: &CredentialsMap) -> CoreResult<()> {
            let json = serde_json::to_string(credentials)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;
            self.save_raw_to_store(&json)
        }

        /// 更新缓存（辅助方法）
        async fn update_cache(&self, credentials: CredentialsMap) {
            let mut cache = self.cache.write().await;
            *cache = Some(credentials);
        }
    }

    #[async_trait]
    impl CredentialStore for TauriCredentialStore {
        async fn load_all(&self) -> CoreResult<CredentialsMap> {
            // 先检查缓存
            {
                let cache = self.cache.read().await;
                if let Some(ref creds) = *cache {
                    return Ok(creds.clone());
                }
            }

            // 从 Store 加载
            let credentials = self.load_from_store()?;

            // 更新缓存
            self.update_cache(credentials.clone()).await;

            log::info!("Loaded {} accounts from Store", credentials.len());
            Ok(credentials)
        }

        async fn save_all(&self, credentials: &CredentialsMap) -> CoreResult<()> {
            self.save_to_store(credentials)?;

            // 更新缓存
            self.update_cache(credentials.clone()).await;

            log::info!("Saved {} accounts to Store", credentials.len());
            Ok(())
        }

        async fn get(&self, account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
            let all_creds = self.load_all().await?;
            Ok(all_creds.get(account_id).cloned())
        }

        async fn set(&self, account_id: &str, credentials: &ProviderCredentials) -> CoreResult<()> {
            let mut all_creds = self.load_all().await?;
            all_creds.insert(account_id.to_string(), credentials.clone());
            self.save_all(&all_creds).await?;

            log::info!("Credentials saved for account: {account_id}");
            Ok(())
        }

        async fn remove(&self, account_id: &str) -> CoreResult<()> {
            let mut all_creds = self.load_all().await?;
            all_creds.remove(account_id);
            self.save_all(&all_creds).await?;

            log::info!("Credentials deleted for account: {account_id}");
            Ok(())
        }

        async fn load_raw_json(&self) -> CoreResult<String> {
            self.load_raw_from_store()
        }

        async fn save_raw_json(&self, json: &str) -> CoreResult<()> {
            self.save_raw_to_store(json)
        }
    }
}

// Re-export 平台特定实现
#[cfg(not(target_os = "android"))]
pub use desktop::TauriCredentialStore;

#[cfg(target_os = "android")]
pub use android::TauriCredentialStore;
