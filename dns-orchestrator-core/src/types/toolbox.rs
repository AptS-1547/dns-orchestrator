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
