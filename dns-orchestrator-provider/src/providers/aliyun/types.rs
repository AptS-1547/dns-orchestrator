//! 阿里云 API 类型定义和辅助函数

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::error::{ProviderError, Result};

// ============ RFC3986 URL 编码 ============

/// RFC3986 URL 编码
pub fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    result
}

/// 将 `serde_json::Value` 展平为 key-value 对 (处理嵌套对象)
pub fn flatten_value(
    prefix: &str,
    value: &serde_json::Value,
    result: &mut BTreeMap<String, String>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_value(&new_key, v, result);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_key = format!("{}.{}", prefix, i + 1);
                flatten_value(&new_key, v, result);
            }
        }
        serde_json::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        serde_json::Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        serde_json::Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        serde_json::Value::Null => {}
    }
}

/// 将结构体序列化为排序后的 query string
pub fn serialize_to_query_string<T: Serialize>(params: &T) -> Result<String> {
    let value = serde_json::to_value(params).map_err(|e| ProviderError::SerializationError {
        provider: "aliyun".to_string(),
        detail: e.to_string(),
    })?;

    let mut flat_map = BTreeMap::new();
    flatten_value("", &value, &mut flat_map);

    let query_string = flat_map
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    Ok(query_string)
}

// ============ 阿里云 API 响应结构 ============

#[derive(Debug, Deserialize)]
pub struct AliyunResponse<T> {
    #[serde(flatten)]
    #[allow(dead_code)]
    pub data: Option<T>,
    #[serde(rename = "Code")]
    pub code: Option<String>,
    #[serde(rename = "Message")]
    pub message: Option<String>,
}

// ============ 域名相关结构 ============

#[derive(Debug, Deserialize)]
pub struct DescribeDomainsResponse {
    #[serde(rename = "Domains")]
    pub domains: Option<DomainsWrapper>,
    #[serde(rename = "TotalCount")]
    pub total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DomainsWrapper {
    #[serde(rename = "Domain")]
    pub domain: Option<Vec<AliyunDomain>>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunDomain {
    #[serde(rename = "DomainId")]
    pub domain_id: Option<String>,
    #[serde(rename = "DomainName")]
    pub domain_name: String,
    #[serde(rename = "DomainStatus")]
    pub domain_status: Option<String>,
    #[serde(rename = "RecordCount")]
    pub record_count: Option<u32>,
}

/// ErrorRequireCheck: DescribeDomainInfo API 响应结构，需验证字段映射是否正确
#[derive(Debug, Deserialize)]
pub struct DescribeDomainInfoResponse {
    #[serde(rename = "DomainId")]
    pub domain_id: Option<String>,
    #[serde(rename = "DomainName")]
    pub domain_name: String,
    #[serde(rename = "DomainStatus")]
    pub domain_status: Option<String>,
    #[serde(rename = "RecordCount")]
    pub record_count: Option<u32>,
}

// ============ 记录相关结构 ============

#[derive(Debug, Deserialize)]
pub struct DescribeDomainRecordsResponse {
    #[serde(rename = "DomainRecords")]
    pub domain_records: Option<DomainRecordsWrapper>,
    #[serde(rename = "TotalCount")]
    pub total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DomainRecordsWrapper {
    #[serde(rename = "Record")]
    pub record: Option<Vec<AliyunRecord>>,
}

#[derive(Debug, Deserialize)]
pub struct AliyunRecord {
    #[serde(rename = "RecordId")]
    pub record_id: String,
    #[serde(rename = "RR")]
    pub rr: String,
    #[serde(rename = "Type")]
    pub record_type: String,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "TTL")]
    pub ttl: u32,
    #[serde(rename = "Priority")]
    pub priority: Option<u16>,
    #[serde(rename = "CreateTimestamp")]
    pub create_timestamp: Option<i64>,
    #[serde(rename = "UpdateTimestamp")]
    pub update_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddDomainRecordResponse {
    #[serde(rename = "RecordId")]
    pub record_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDomainRecordResponse {
    #[serde(rename = "RecordId")]
    #[allow(dead_code)]
    pub record_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDomainRecordResponse {
    #[serde(rename = "RecordId")]
    #[allow(dead_code)]
    pub record_id: Option<String>,
}
