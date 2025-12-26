//! 华为云 DNS API 类型定义

use serde::Deserialize;

// ============ 华为云 API 响应结构 ============

#[derive(Debug, Deserialize)]
pub struct ListZonesResponse {
    pub zones: Option<Vec<HuaweicloudZone>>,
    #[serde(rename = "metadata")]
    pub metadata: Option<ListMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct ListMetadata {
    pub total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct HuaweicloudZone {
    pub id: String,
    pub name: String,
    pub status: Option<String>,
    pub record_num: Option<u32>,
}

/// ShowPublicZone API 响应结构，需验证是否直接返回 zone 对象
pub type ShowPublicZoneResponse = HuaweicloudZone;

#[derive(Debug, Deserialize)]
pub struct ListRecordSetsResponse {
    pub recordsets: Option<Vec<HuaweicloudRecordSet>>,
    #[serde(rename = "metadata")]
    pub metadata: Option<ListMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct HuaweicloudRecordSet {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub records: Option<Vec<String>>,
    pub ttl: Option<u32>,
    #[allow(dead_code)]
    pub status: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: Option<String>,
    #[serde(rename = "updated_at")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecordSetResponse {
    pub id: String,
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub record_type: String,
    #[allow(dead_code)]
    pub records: Option<Vec<String>>,
    #[allow(dead_code)]
    pub ttl: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub code: Option<String>,
    pub message: Option<String>,
}
