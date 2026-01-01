//! 工具箱相关类型定义

use serde::{Deserialize, Serialize};

/// WHOIS 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoisResult {
    /// 域名
    pub domain: String,
    /// 注册商
    pub registrar: Option<String>,
    /// 创建日期
    pub creation_date: Option<String>,
    /// 过期日期
    pub expiration_date: Option<String>,
    /// 更新日期
    pub updated_date: Option<String>,
    /// 名称服务器
    pub name_servers: Vec<String>,
    /// 状态
    pub status: Vec<String>,
    /// 原始响应
    pub raw: String,
}

/// DNS 查询记录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupRecord {
    /// 记录类型
    pub record_type: String,
    /// 记录名称
    pub name: String,
    /// 记录值
    pub value: String,
    /// TTL
    pub ttl: u32,
    /// 优先级（MX/SRV 记录）
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
    /// IP 地址
    pub ip: String,
    /// IP 版本: "IPv4" 或 "IPv6"
    pub ip_version: String,
    /// 国家
    pub country: Option<String>,
    /// 国家代码
    pub country_code: Option<String>,
    /// 地区/省份
    pub region: Option<String>,
    /// 城市
    pub city: Option<String>,
    /// 纬度
    pub latitude: Option<f64>,
    /// 经度
    pub longitude: Option<f64>,
    /// 时区
    pub timezone: Option<String>,
    /// ISP
    pub isp: Option<String>,
    /// 组织
    pub org: Option<String>,
    /// ASN
    pub asn: Option<String>,
    /// AS 名称
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
    /// 域名
    pub domain: String,
    /// 颁发者
    pub issuer: String,
    /// 主题
    pub subject: String,
    /// 有效期起始
    pub valid_from: String,
    /// 有效期截止
    pub valid_to: String,
    /// 剩余天数
    pub days_remaining: i64,
    /// 是否已过期
    pub is_expired: bool,
    /// 是否有效
    pub is_valid: bool,
    /// 主题备用名称
    pub san: Vec<String>,
    /// 序列号
    pub serial_number: String,
    /// 签名算法
    pub signature_algorithm: String,
    /// 证书链
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
    /// 主题
    pub subject: String,
    /// 颁发者
    pub issuer: String,
    /// 是否为 CA 证书
    pub is_ca: bool,
}

/// HTTP 请求方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
}

/// HTTP 请求头
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    /// 请求头名称
    pub name: String,
    /// 请求头值
    pub value: String,
}

/// HTTP 头检查请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeaderCheckRequest {
    /// 目标 URL
    pub url: String,
    /// HTTP 方法
    pub method: HttpMethod,
    /// 自定义请求头列表
    pub custom_headers: Vec<HttpHeader>,
    /// 请求体（仅 POST/PUT/PATCH）
    pub body: Option<String>,
    /// 请求体内容类型
    pub content_type: Option<String>,
}

/// 安全头分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityHeaderAnalysis {
    /// 安全头名称
    pub name: String,
    /// 是否存在
    pub present: bool,
    /// 头值（如果存在）
    pub value: Option<String>,
    /// 状态: "good" | "warning" | "missing"
    pub status: String,
    /// 建议
    pub recommendation: Option<String>,
}

/// HTTP 头检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeaderCheckResult {
    /// 请求的 URL
    pub url: String,
    /// HTTP 状态码
    pub status_code: u16,
    /// 状态文本
    pub status_text: String,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 所有响应头
    pub headers: Vec<HttpHeader>,
    /// 安全头分析
    pub security_analysis: Vec<SecurityHeaderAnalysis>,
    /// Content-Length
    pub content_length: Option<u64>,
    /// 原始请求报文
    pub raw_request: String,
    /// 原始响应报文
    pub raw_response: String,
}

/// DNS 传播检查服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationServer {
    /// 服务器名称（如 "Google DNS"）
    pub name: String,
    /// 服务器 IP 地址
    pub ip: String,
    /// 地区（如 "美国（北美）"）
    pub region: String,
    /// 国家代码（如 "US"）
    pub country_code: String,
}

/// 单个 DNS 服务器的查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationServerResult {
    /// 服务器信息
    pub server: DnsPropagationServer,
    /// 查询状态: "success" | "timeout" | "error"
    pub status: String,
    /// 查询记录列表（成功时）
    pub records: Vec<DnsLookupRecord>,
    /// 错误信息（失败时）
    pub error: Option<String>,
    /// 查询耗时（毫秒）
    pub response_time_ms: u64,
}

/// DNS 传播检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationResult {
    /// 查询的域名
    pub domain: String,
    /// 查询的记录类型
    pub record_type: String,
    /// 各服务器查询结果
    pub results: Vec<DnsPropagationServerResult>,
    /// 总查询时间（毫秒）
    pub total_time_ms: u64,
    /// 传播一致性（0-100%）
    pub consistency_percentage: f32,
    /// 唯一值列表（用于检测一致性）
    pub unique_values: Vec<String>,
}

/// DNSSEC DNSKEY 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnskeyRecord {
    /// 标志位（256=ZSK, 257=KSK）
    pub flags: u16,
    /// 协议（始终为 3）
    pub protocol: u8,
    /// 算法编号
    pub algorithm: u8,
    /// 算法名称
    pub algorithm_name: String,
    /// 公钥（Base64 编码）
    pub public_key: String,
    /// 密钥标签
    pub key_tag: u16,
    /// 密钥类型: "ZSK" | "KSK"
    pub key_type: String,
}

/// DNSSEC DS 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DsRecord {
    /// 密钥标签
    pub key_tag: u16,
    /// 算法编号
    pub algorithm: u8,
    /// 算法名称
    pub algorithm_name: String,
    /// 摘要类型（1=SHA-1, 2=SHA-256, 4=SHA-384）
    pub digest_type: u8,
    /// 摘要类型名称
    pub digest_type_name: String,
    /// 摘要（十六进制）
    pub digest: String,
}

/// DNSSEC RRSIG 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RrsigRecord {
    /// 覆盖的记录类型
    pub type_covered: String,
    /// 算法编号
    pub algorithm: u8,
    /// 算法名称
    pub algorithm_name: String,
    /// 标签数
    pub labels: u8,
    /// 原始 TTL
    pub original_ttl: u32,
    /// 签名过期时间
    pub signature_expiration: String,
    /// 签名生成时间
    pub signature_inception: String,
    /// 密钥标签
    pub key_tag: u16,
    /// 签名者名称
    pub signer_name: String,
    /// 签名数据（Base64）
    pub signature: String,
}

/// DNSSEC 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnssecResult {
    /// 查询的域名
    pub domain: String,
    /// DNSSEC 是否启用
    pub dnssec_enabled: bool,
    /// DNSKEY 记录列表
    pub dnskey_records: Vec<DnskeyRecord>,
    /// DS 记录列表
    pub ds_records: Vec<DsRecord>,
    /// RRSIG 记录列表
    pub rrsig_records: Vec<RrsigRecord>,
    /// 验证状态: "secure" | "insecure" | "bogus" | "indeterminate"
    pub validation_status: String,
    /// 使用的 DNS 服务器
    pub nameserver: String,
    /// 查询耗时（毫秒）
    pub response_time_ms: u64,
    /// 错误信息（查询失败时）
    pub error: Option<String>,
}
