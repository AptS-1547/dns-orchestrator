//! 应用状态模块

use dns_orchestrator_provider::DnsProvider;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::crypto::CryptoManager;

/// Provider 注册表
#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册 Provider
    pub async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.providers.write().await.insert(account_id, provider);
    }

    /// 注销 Provider
    pub async fn unregister(&self, account_id: &str) {
        self.providers.write().await.remove(account_id);
    }

    /// 获取 Provider
    pub async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>> {
        self.providers.read().await.get(account_id).cloned()
    }

    /// 获取所有已注册的账户 ID
    pub async fn list_account_ids(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接
    pub db: DatabaseConnection,
    /// Provider 注册表
    pub registry: ProviderRegistry,
    /// 加密管理器
    pub crypto: CryptoManager,
}

impl AppState {
    pub fn new(db: DatabaseConnection, crypto: CryptoManager) -> Self {
        Self {
            db,
            registry: ProviderRegistry::new(),
            crypto,
        }
    }
}
