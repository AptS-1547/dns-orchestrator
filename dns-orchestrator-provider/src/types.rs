use serde::{Deserialize, Serialize};

// ============ 分页相关类型 ============

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// DNS 记录查询参数（包含搜索和过滤）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordQueryParams {
    pub page: u32,
    pub page_size: u32,
    /// 搜索关键词（匹配记录名称或值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// 记录类型过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_type: Option<DnsRecordType>,
}

impl Default for RecordQueryParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            keyword: None,
            record_type: None,
        }
    }
}

impl RecordQueryParams {
    /// 转换为基础分页参数
    pub fn to_pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            page_size: self.page_size,
        }
    }
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_count: u32,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total_count: u32) -> Self {
        let has_more = (page * page_size) < total_count;
        Self {
            items,
            page,
            page_size,
            total_count,
            has_more,
        }
    }
}

// ============ Provider 相关类型 ============

/// Provider 类型枚举（原名 DnsProvider，重命名避免与 trait 冲突）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    #[cfg(feature = "cloudflare")]
    Cloudflare,
    #[cfg(feature = "aliyun")]
    Aliyun,
    #[cfg(feature = "dnspod")]
    Dnspod,
    #[cfg(feature = "huaweicloud")]
    Huaweicloud,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "cloudflare")]
            Self::Cloudflare => write!(f, "cloudflare"),
            #[cfg(feature = "aliyun")]
            Self::Aliyun => write!(f, "aliyun"),
            #[cfg(feature = "dnspod")]
            Self::Dnspod => write!(f, "dnspod"),
            #[cfg(feature = "huaweicloud")]
            Self::Huaweicloud => write!(f, "huaweicloud"),
        }
    }
}

// ============ 域名相关类型 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainStatus {
    Active,
    Paused,
    Pending,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDomain {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    pub status: DomainStatus,
    #[serde(rename = "recordCount", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
}

// ============ DNS 记录相关类型 ============

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    A,
    Aaaa,
    Cname,
    Mx,
    Txt,
    Ns,
    Srv,
    Caa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "domainId")]
    pub domain_id: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(rename = "createdAt")]
    #[serde(with = "crate::utils::datetime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    #[serde(with = "crate::utils::datetime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsRecordRequest {
    #[serde(rename = "domainId")]
    pub domain_id: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDnsRecordRequest {
    #[serde(rename = "domainId")]
    pub domain_id: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
}

// ============ 批量操作类型 ============

/// 批量创建结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub created_records: Vec<DnsRecord>,
    pub failures: Vec<BatchCreateFailure>,
}

/// 批量创建失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateFailure {
    pub request_index: usize,
    pub record_name: String,
    pub reason: String,
}

/// 批量更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub updated_records: Vec<DnsRecord>,
    pub failures: Vec<BatchUpdateFailure>,
}

/// 批量更新失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateFailure {
    pub record_id: String,
    pub reason: String,
}

/// 批量更新请求项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateItem {
    pub record_id: String,
    pub request: UpdateDnsRecordRequest,
}

/// 批量删除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<BatchDeleteFailure>,
}

/// 批量删除失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteFailure {
    pub record_id: String,
    pub reason: String,
}

// ============ Provider 元数据类型 ============

/// 凭证字段类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Password,
}

/// 提供商凭证字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentialField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_text: Option<String>,
}

/// 提供商支持的功能
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderFeatures {
    /// 是否支持代理功能 (如 Cloudflare 的 CDN 代理)
    pub proxy: bool,
}

/// 提供商分页限制
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLimits {
    /// 域名列表的最大分页大小
    pub max_page_size_domains: u32,
    /// DNS 记录列表的最大分页大小
    pub max_page_size_records: u32,
}

/// 提供商元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMetadata {
    pub id: ProviderType,
    pub name: String,
    pub description: String,
    pub required_fields: Vec<ProviderCredentialField>,
    pub features: ProviderFeatures,
    pub limits: ProviderLimits,
}

// ============ 凭证类型 ============

/// 凭证验证错误
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CredentialValidationError {
    /// 缺失必需字段
    MissingField {
        provider: ProviderType,
        field: String,
        label: String,
    },
    /// 字段值为空
    EmptyField {
        provider: ProviderType,
        field: String,
        label: String,
    },
    /// 字段格式无效
    InvalidFormat {
        provider: ProviderType,
        field: String,
        label: String,
        reason: String,
    },
}

impl std::fmt::Display for CredentialValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField { label, .. } => write!(f, "缺少必填字段: {label}"),
            Self::EmptyField { label, .. } => write!(f, "字段不能为空: {label}"),
            Self::InvalidFormat { label, reason, .. } => write!(f, "{label}: {reason}"),
        }
    }
}

impl std::error::Error for CredentialValidationError {}

/// 凭证枚举 - 类型安全的凭证定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "credentials")]
pub enum ProviderCredentials {
    #[cfg(feature = "cloudflare")]
    #[serde(rename = "cloudflare")]
    Cloudflare { api_token: String },

    #[cfg(feature = "aliyun")]
    #[serde(rename = "aliyun")]
    Aliyun {
        access_key_id: String,
        access_key_secret: String,
    },

    #[cfg(feature = "dnspod")]
    #[serde(rename = "dnspod")]
    Dnspod {
        secret_id: String,
        secret_key: String,
    },

    #[cfg(feature = "huaweicloud")]
    #[serde(rename = "huaweicloud")]
    Huaweicloud {
        access_key_id: String,
        secret_access_key: String,
    },
}

impl ProviderCredentials {
    /// 从 HashMap 转换（兼容旧格式存储）
    pub fn from_map(
        provider: &ProviderType,
        map: &std::collections::HashMap<String, String>,
    ) -> Result<Self, CredentialValidationError> {
        match provider {
            #[cfg(feature = "cloudflare")]
            ProviderType::Cloudflare => Ok(Self::Cloudflare {
                api_token: Self::get_required_field(provider, map, "apiToken", "API Token")?,
            }),
            #[cfg(feature = "aliyun")]
            ProviderType::Aliyun => Ok(Self::Aliyun {
                access_key_id: Self::get_required_field(
                    provider,
                    map,
                    "accessKeyId",
                    "Access Key ID",
                )?,
                access_key_secret: Self::get_required_field(
                    provider,
                    map,
                    "accessKeySecret",
                    "Access Key Secret",
                )?,
            }),
            #[cfg(feature = "dnspod")]
            ProviderType::Dnspod => Ok(Self::Dnspod {
                secret_id: Self::get_required_field(provider, map, "secretId", "Secret ID")?,
                secret_key: Self::get_required_field(provider, map, "secretKey", "Secret Key")?,
            }),
            #[cfg(feature = "huaweicloud")]
            ProviderType::Huaweicloud => Ok(Self::Huaweicloud {
                access_key_id: Self::get_required_field(
                    provider,
                    map,
                    "accessKeyId",
                    "Access Key ID",
                )?,
                secret_access_key: Self::get_required_field(
                    provider,
                    map,
                    "secretAccessKey",
                    "Secret Access Key",
                )?,
            }),
            #[allow(unreachable_patterns)]
            _ => Err(CredentialValidationError::InvalidFormat {
                provider: provider.clone(),
                field: "provider".to_string(),
                label: "Provider".to_string(),
                reason: format!(
                    "Provider '{}' is not supported or its feature is not enabled.",
                    provider
                ),
            }),
        }
    }

    /// 从 HashMap 中获取必需字段，校验非空
    fn get_required_field(
        provider: &ProviderType,
        map: &std::collections::HashMap<String, String>,
        key: &str,
        label: &str,
    ) -> Result<String, CredentialValidationError> {
        match map.get(key) {
            None => Err(CredentialValidationError::MissingField {
                provider: provider.clone(),
                field: key.to_string(),
                label: label.to_string(),
            }),
            Some(v) if v.trim().is_empty() => Err(CredentialValidationError::EmptyField {
                provider: provider.clone(),
                field: key.to_string(),
                label: label.to_string(),
            }),
            Some(v) => Ok(v.clone()),
        }
    }

    /// 转换为 HashMap（保存时用，保持存储格式兼容）
    pub fn to_map(&self) -> std::collections::HashMap<String, String> {
        match self {
            Self::Cloudflare { api_token } => [("apiToken".to_string(), api_token.clone())].into(),
            Self::Aliyun {
                access_key_id,
                access_key_secret,
            } => [
                ("accessKeyId".to_string(), access_key_id.clone()),
                ("accessKeySecret".to_string(), access_key_secret.clone()),
            ]
            .into(),
            Self::Dnspod {
                secret_id,
                secret_key,
            } => [
                ("secretId".to_string(), secret_id.clone()),
                ("secretKey".to_string(), secret_key.clone()),
            ]
            .into(),
            Self::Huaweicloud {
                access_key_id,
                secret_access_key,
            } => [
                ("accessKeyId".to_string(), access_key_id.clone()),
                ("secretAccessKey".to_string(), secret_access_key.clone()),
            ]
            .into(),
        }
    }

    /// 获取凭证对应的 provider 类型
    pub fn provider_type(&self) -> ProviderType {
        match self {
            Self::Cloudflare { .. } => ProviderType::Cloudflare,
            Self::Aliyun { .. } => ProviderType::Aliyun,
            Self::Dnspod { .. } => ProviderType::Dnspod,
            Self::Huaweicloud { .. } => ProviderType::Huaweicloud,
        }
    }
}
