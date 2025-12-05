use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub record_type: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsProvider {
    Cloudflare,
    Aliyun,
    Dnspod,
    Huaweicloud,
}

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
    pub provider: DnsProvider,
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
    pub provider: DnsProvider,
    pub credentials: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainStatus {
    Active,
    Paused,
    Pending,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub provider: DnsProvider,
    pub status: DomainStatus,
    #[serde(rename = "recordCount", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
}

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
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
            }),
        }
    }
}

/// 提供商凭证字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCredentialField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
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

/// 提供商元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_fields: Vec<ProviderCredentialField>,
    pub features: ProviderFeatures,
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

// ============ 导入导出相关类型 ============

/// 单个账号的导出数据（包含凭证）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedAccount {
    pub id: String,
    pub name: String,
    pub provider: DnsProvider,
    pub created_at: String,
    pub updated_at: String,
    /// 凭证数据（导出时包含）
    pub credentials: HashMap<String, String>,
}

/// 导出文件头部（明文部分）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFileHeader {
    /// 文件格式版本
    pub version: u32,
    /// 是否加密
    pub encrypted: bool,
    /// 加密时使用的盐值（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// 加密时使用的 IV/Nonce（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// 导出时间
    pub exported_at: String,
    /// 应用版本
    pub app_version: String,
}

/// 完整的导出文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub header: ExportFileHeader,
    /// 账号数据（加密时为 Base64 编码的密文，未加密时为 JSON 数组）
    pub data: serde_json::Value,
}

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
    pub provider: DnsProvider,
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
