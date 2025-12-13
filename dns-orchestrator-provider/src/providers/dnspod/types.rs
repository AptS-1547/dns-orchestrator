//! 腾讯云 DNSPod API 类型定义

use serde::Deserialize;

// ============ 腾讯云 API 响应结构 ============

#[derive(Debug, Deserialize)]
pub struct TencentResponse<T> {
    #[serde(rename = "Response")]
    pub response: TencentResponseInner<T>,
}

#[derive(Debug, Deserialize)]
pub struct TencentResponseInner<T> {
    #[serde(flatten)]
    pub data: Option<T>,
    #[serde(rename = "Error")]
    pub error: Option<TencentError>,
    #[serde(rename = "RequestId")]
    #[allow(dead_code)]
    pub request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TencentError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

// ============ DNSPod 域名相关结构 ============

#[derive(Debug, Deserialize)]
pub struct DomainListResponse {
    #[serde(rename = "DomainList")]
    pub domain_list: Option<Vec<DnspodDomain>>,
    #[serde(rename = "DomainCountInfo")]
    pub domain_count_info: Option<DomainCountInfo>,
}

#[derive(Debug, Deserialize)]
pub struct DomainCountInfo {
    #[serde(rename = "AllTotal")]
    pub all_total: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DnspodDomain {
    #[serde(rename = "DomainId")]
    pub domain_id: u64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "DNSStatus")]
    pub dns_status: String,
    #[serde(rename = "RecordCount")]
    pub record_count: Option<u32>,
}

/// DescribeDomain API 响应结构
#[derive(Debug, Deserialize)]
pub struct DescribeDomainResponse {
    #[serde(rename = "DomainInfo")]
    pub domain_info: DescribeDomainInfo,
}

/// DomainInfo 嵌套结构
#[derive(Debug, Deserialize)]
pub struct DescribeDomainInfo {
    #[serde(rename = "DomainId")]
    pub domain_id: u64,
    #[serde(rename = "Domain")]
    pub domain: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "DNSStatus")]
    pub dns_status: String,
    #[serde(rename = "RecordCount")]
    pub record_count: Option<u32>,
}

// ============ DNSPod 记录相关结构 ============

#[derive(Debug, Deserialize)]
pub struct RecordListResponse {
    #[serde(rename = "RecordList")]
    pub record_list: Option<Vec<DnspodRecord>>,
    #[serde(rename = "RecordCountInfo")]
    pub record_count_info: Option<RecordCountInfo>,
}

#[derive(Debug, Deserialize)]
pub struct RecordCountInfo {
    #[serde(rename = "TotalCount")]
    pub total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DnspodRecord {
    #[serde(rename = "RecordId")]
    pub record_id: u64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub record_type: String,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "TTL")]
    pub ttl: u32,
    #[serde(rename = "MX")]
    pub mx: Option<u16>,
    #[serde(rename = "UpdatedOn")]
    pub updated_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordResponse {
    #[serde(rename = "RecordId")]
    pub record_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ModifyRecordResponse {
    #[serde(rename = "RecordId")]
    #[allow(dead_code)]
    pub record_id: u64,
}
