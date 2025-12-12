use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============ Re-export 库类型 ============

pub use dns_orchestrator_provider::{
    // DNS 记录类型
    CreateDnsRecordRequest,
    DnsRecord,
    DnsRecordType,
    // Domain 相关
    DomainStatus,
    // 分页类型
    PaginatedResponse,
    // Provider 元数据类型
    ProviderMetadata,
    ProviderType,
    UpdateDnsRecordRequest,
};

// ============ 应用层 Provider 相关类型 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider: ProviderType,
    pub credentials: HashMap<String, String>,
}

// ============ 应用层 Domain（包含 account_id）============

/// 应用层 Domain 类型（包含 `account_id`）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub provider: ProviderType,
    pub status: DomainStatus,
    #[serde(rename = "recordCount", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
}

// ============ API 响应类型 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
        }
    }
}

// ============ 工具箱相关类型 ============

/// WHOIS 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoisResult {
    pub domain: String,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiration_date: Option<String>,
    pub updated_date: Option<String>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    pub raw: String,
}

/// DNS 查询记录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

/// DNS 查询结果（包含 nameserver 信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupResult {
    /// 使用的 DNS 服务器
    pub nameserver: String,
    /// 查询记录列表
    pub records: Vec<DnsLookupRecord>,
}

/// IP 地理位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpGeoInfo {
    pub ip: String,
    /// IP 版本: "IPv4" 或 "IPv6"
    pub ip_version: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
    pub org: Option<String>,
    pub asn: Option<String>,
    pub as_name: Option<String>,
}

/// IP 查询结果（支持域名解析多个 IP）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpLookupResult {
    /// 查询的原始输入（IP 或域名）
    pub query: String,
    /// 是否为域名查询
    pub is_domain: bool,
    /// IP 地理位置结果列表
    pub results: Vec<IpGeoInfo>,
}

/// SSL 证书信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCertInfo {
    pub domain: String,
    pub issuer: String,
    pub subject: String,
    pub valid_from: String,
    pub valid_to: String,
    pub days_remaining: i64,
    pub is_expired: bool,
    pub is_valid: bool,
    pub san: Vec<String>,
    pub serial_number: String,
    pub signature_algorithm: String,
    pub certificate_chain: Vec<CertChainItem>,
}

/// SSL 检查结果（包含连接状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCheckResult {
    /// 查询的域名
    pub domain: String,
    /// 检查的端口
    pub port: u16,
    /// 连接状态: "https" | "http" | "failed"
    pub connection_status: String,
    /// 证书信息（仅当 HTTPS 连接成功时存在）
    pub cert_info: Option<SslCertInfo>,
    /// 错误信息（连接失败时）
    pub error: Option<String>,
}

/// 证书链项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertChainItem {
    pub subject: String,
    pub issuer: String,
    pub is_ca: bool,
}

// ============ 批量操作相关类型 ============

/// 批量删除 DNS 记录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    pub domain_id: String,
    pub record_ids: Vec<String>,
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

// ============ 导入导出相关类型 ============

/// 导出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsRequest {
    /// 要导出的账号 ID 列表
    pub account_ids: Vec<String>,
    /// 是否加密
    pub encrypt: bool,
    /// 加密密码（仅当 encrypt=true 时需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// 导出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsResponse {
    /// 导出的 JSON 内容
    pub content: String,
    /// 建议的文件名
    pub suggested_filename: String,
}

/// 导入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAccountsRequest {
    /// 导入文件的内容
    pub content: String,
    /// 解密密码（如果文件加密）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// 导入预览（用于显示将要导入的账号）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    /// 文件是否加密
    pub encrypted: bool,
    /// 账号数量
    pub account_count: usize,
    /// 账号预览列表（仅在未加密或已解密后可用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<ImportPreviewAccount>>,
}

/// 导入预览中的账号信息（不含敏感凭证）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewAccount {
    pub name: String,
    pub provider: ProviderType,
    /// 是否与现有账号名称冲突
    pub has_conflict: bool,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// 成功导入的账号数量
    pub success_count: usize,
    /// 失败的账号及原因
    pub failures: Vec<ImportFailure>,
}

/// 导入失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    pub name: String,
    pub reason: String,
}
