//! Cloudflare API 类型定义

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Cloudflare API 通用响应
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Option<Vec<CloudflareError>>,
    pub result_info: Option<CloudflareResultInfo>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareError {
    #[allow(dead_code)]
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResultInfo {
    #[allow(dead_code)]
    pub page: u32,
    #[allow(dead_code)]
    pub per_page: u32,
    pub total_count: u32,
}

/// Cloudflare Zone 结构
#[derive(Debug, Deserialize)]
pub struct CloudflareZone {
    pub id: String,
    pub name: String,
    pub status: String,
}

/// Cloudflare DNS Record 结构（响应）
#[derive(Debug, Deserialize)]
pub struct CloudflareDnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_on: Option<String>,
    /// SRV/CAA 等复杂记录类型的结构化数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// SRV 记录的 data 字段
#[derive(Debug, Serialize, Deserialize)]
pub struct CloudflareSrvData {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
    /// SRV 记录的服务名和协议（可选，API 可能返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// CAA 记录的 data 字段
#[derive(Debug, Serialize, Deserialize)]
pub struct CloudflareCaaData {
    pub flags: u8,
    pub tag: String,
    pub value: String,
}
