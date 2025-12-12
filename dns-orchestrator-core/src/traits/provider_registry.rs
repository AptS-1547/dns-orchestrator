//! Provider 注册表抽象 Trait

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use dns_orchestrator_provider::DnsProvider;

/// Provider 注册表 Trait
///
/// 管理所有已注册的 Provider 实例，按 `account_id` 索引。
/// 提供默认的内存实现 `InMemoryProviderRegistry`。
#[async_trait]
pub trait ProviderRegistry: Send + Sync {
    /// 注册 Provider 实例
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    /// * `provider` - Provider 实例
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);

    /// 注销 Provider
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn unregister(&self, account_id: &str);

    /// 获取 Provider 实例
    ///
    /// # Arguments
    /// * `account_id` - 账户 ID
    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;

    /// 列出所有已注册的 `account_id`
    async fn list_account_ids(&self) -> Vec<String>;
}

/// 内存实现的 Provider 注册表
///
/// 默认实现，适用于所有平台。
#[derive(Clone)]
pub struct InMemoryProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl InMemoryProviderRegistry {
    /// 创建新的内存注册表
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderRegistry for InMemoryProviderRegistry {
    async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.providers.write().await.insert(account_id, provider);
    }

    async fn unregister(&self, account_id: &str) {
        self.providers.write().await.remove(account_id);
    }

    async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>> {
        self.providers.read().await.get(account_id).cloned()
    }

    async fn list_account_ids(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }
}
