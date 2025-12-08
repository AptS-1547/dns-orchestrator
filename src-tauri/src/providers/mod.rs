mod aliyun;
mod cloudflare;
mod dnspod;
mod huaweicloud;

pub use aliyun::AliyunProvider;
pub use cloudflare::CloudflareProvider;
pub use dnspod::DnspodProvider;
pub use huaweicloud::HuaweicloudProvider;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{DnsError, Result};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, Domain, PaginatedResponse, PaginationParams,
    RecordQueryParams, UpdateDnsRecordRequest,
};

/// DNS 提供商 Trait
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// 提供商标识符
    fn id(&self) -> &'static str;

    /// 验证凭证是否有效
    async fn validate_credentials(&self) -> Result<bool>;

    /// 获取域名列表 (分页)
    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>>;

    /// 获取域名详情
    async fn get_domain(&self, domain_id: &str) -> Result<Domain>;

    /// 获取 DNS 记录列表 (分页 + 搜索)
    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>>;

    /// 创建 DNS 记录
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;

    /// 更新 DNS 记录
    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord>;

    /// 删除 DNS 记录
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;
}

/// Provider 注册表 - 管理所有已注册的 Provider 实例
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册提供商实例 (按 `account_id`)
    pub async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.providers.write().await.insert(account_id, provider);
    }

    /// 注销提供商
    pub async fn unregister(&self, account_id: &str) {
        self.providers.write().await.remove(account_id);
    }

    /// 获取提供商实例
    pub async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>> {
        self.providers.read().await.get(account_id).cloned()
    }

    /// 获取所有已注册的 `account_id`
    pub async fn list_account_ids(&self) -> Vec<String> {
        self.providers.read().await.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工厂函数 - 根据提供商类型创建 Provider 实例
pub fn create_provider(
    provider_type: &str,
    credentials: HashMap<String, String>,
) -> Result<Arc<dyn DnsProvider>> {
    match provider_type {
        "cloudflare" => Ok(Arc::new(CloudflareProvider::new(credentials))),
        "aliyun" => Ok(Arc::new(AliyunProvider::new(credentials))),
        "dnspod" => Ok(Arc::new(DnspodProvider::new(credentials))),
        "huaweicloud" => Ok(Arc::new(HuaweicloudProvider::new(credentials))),
        _ => Err(DnsError::ProviderNotFound(provider_type.to_string())),
    }
}

/// 获取所有支持的提供商元数据
pub fn get_all_provider_metadata() -> Vec<crate::types::ProviderMetadata> {
    use crate::types::{ProviderCredentialField, ProviderFeatures, ProviderMetadata};

    vec![
        ProviderMetadata {
            id: "cloudflare".to_string(),
            name: "Cloudflare".to_string(),
            description: "全球领先的 CDN 和 DNS 服务商".to_string(),
            required_fields: vec![ProviderCredentialField {
                key: "apiToken".to_string(),
                label: "API Token".to_string(),
                field_type: "password".to_string(),
                placeholder: Some("输入 Cloudflare API Token".to_string()),
                help_text: Some(
                    "在 Cloudflare Dashboard -> My Profile -> API Tokens 创建".to_string(),
                ),
            }],
            features: ProviderFeatures { proxy: true },
        },
        ProviderMetadata {
            id: "aliyun".to_string(),
            name: "阿里云 DNS".to_string(),
            description: "阿里云域名解析服务".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "accessKeyId".to_string(),
                    label: "AccessKey ID".to_string(),
                    field_type: "text".to_string(),
                    placeholder: Some("输入 AccessKey ID".to_string()),
                    help_text: None,
                },
                ProviderCredentialField {
                    key: "accessKeySecret".to_string(),
                    label: "AccessKey Secret".to_string(),
                    field_type: "password".to_string(),
                    placeholder: Some("输入 AccessKey Secret".to_string()),
                    help_text: None,
                },
            ],
            features: ProviderFeatures::default(),
        },
        ProviderMetadata {
            id: "dnspod".to_string(),
            name: "腾讯云 DNSPod".to_string(),
            description: "腾讯云 DNS 解析服务".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "secretId".to_string(),
                    label: "SecretId".to_string(),
                    field_type: "text".to_string(),
                    placeholder: Some("输入 SecretId".to_string()),
                    help_text: None,
                },
                ProviderCredentialField {
                    key: "secretKey".to_string(),
                    label: "SecretKey".to_string(),
                    field_type: "password".to_string(),
                    placeholder: Some("输入 SecretKey".to_string()),
                    help_text: None,
                },
            ],
            features: ProviderFeatures::default(),
        },
        ProviderMetadata {
            id: "huaweicloud".to_string(),
            name: "华为云 DNS".to_string(),
            description: "华为云云解析服务".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "accessKeyId".to_string(),
                    label: "Access Key ID".to_string(),
                    field_type: "text".to_string(),
                    placeholder: Some("输入 Access Key ID".to_string()),
                    help_text: None,
                },
                ProviderCredentialField {
                    key: "secretAccessKey".to_string(),
                    label: "Secret Access Key".to_string(),
                    field_type: "password".to_string(),
                    placeholder: Some("输入 Secret Access Key".to_string()),
                    help_text: None,
                },
            ],
            features: ProviderFeatures::default(),
        },
    ]
}
