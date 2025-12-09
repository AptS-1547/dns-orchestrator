use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

use super::{DnsProvider, ErrorContext, ProviderErrorMapper, RawApiError};
use crate::error::{DnsError, ProviderError, Result};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, Domain, DomainStatus, PaginatedResponse,
    PaginationParams, RecordQueryParams, UpdateDnsRecordRequest,
};

const ALIYUN_DNS_HOST: &str = "alidns.cn-hangzhou.aliyuncs.com";
const ALIYUN_DNS_VERSION: &str = "2015-01-09";
/// 空 body 的 SHA256 hash (固定值)
const EMPTY_BODY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

type HmacSha256 = Hmac<Sha256>;

/// RFC3986 URL 编码
fn url_encode(s: &str) -> String {
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
fn flatten_value(prefix: &str, value: &serde_json::Value, result: &mut BTreeMap<String, String>) {
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
fn serialize_to_query_string<T: Serialize>(params: &T) -> Result<String> {
    let value =
        serde_json::to_value(params).map_err(|e| DnsError::SerializationError(e.to_string()))?;

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
struct AliyunResponse<T> {
    #[serde(flatten)]
    #[allow(dead_code)]
    data: Option<T>,
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
}

// ============ 域名相关结构 ============

#[derive(Debug, Deserialize)]
struct DescribeDomainsResponse {
    #[serde(rename = "Domains")]
    domains: Option<DomainsWrapper>,
    #[serde(rename = "TotalCount")]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct DomainsWrapper {
    #[serde(rename = "Domain")]
    domain: Option<Vec<AliyunDomain>>,
}

#[derive(Debug, Deserialize)]
struct AliyunDomain {
    #[serde(rename = "DomainId")]
    domain_id: Option<String>,
    #[serde(rename = "DomainName")]
    domain_name: String,
    #[serde(rename = "DomainStatus")]
    domain_status: Option<String>,
    #[serde(rename = "RecordCount")]
    record_count: Option<u32>,
}

// ============ 记录相关结构 ============

#[derive(Debug, Deserialize)]
struct DescribeDomainRecordsResponse {
    #[serde(rename = "DomainRecords")]
    domain_records: Option<DomainRecordsWrapper>,
    #[serde(rename = "TotalCount")]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct DomainRecordsWrapper {
    #[serde(rename = "Record")]
    record: Option<Vec<AliyunRecord>>,
}

#[derive(Debug, Deserialize)]
struct AliyunRecord {
    #[serde(rename = "RecordId")]
    record_id: String,
    #[serde(rename = "RR")]
    rr: String,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "TTL")]
    ttl: u32,
    #[serde(rename = "Priority")]
    priority: Option<u16>,
    #[serde(rename = "CreateTimestamp")]
    create_timestamp: Option<i64>,
    #[serde(rename = "UpdateTimestamp")]
    update_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AddDomainRecordResponse {
    #[serde(rename = "RecordId")]
    record_id: String,
}

#[derive(Debug, Deserialize)]
struct UpdateDomainRecordResponse {
    #[serde(rename = "RecordId")]
    record_id: String,
}

#[derive(Debug, Deserialize)]
struct DeleteDomainRecordResponse {
    #[serde(rename = "RecordId")]
    #[allow(dead_code)]
    record_id: Option<String>,
}

// ============ 阿里云 DNS Provider 实现 ============

/// 阿里云 DNS Provider
pub struct AliyunProvider {
    client: Client,
    access_key_id: String,
    access_key_secret: String,
    account_id: String,
}

/// 阿里云错误码映射
/// 参考: <https://help.aliyun.com/document_detail/29774.html>
impl ProviderErrorMapper for AliyunProvider {
    fn provider_name(&self) -> &'static str {
        "aliyun"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("InvalidAccessKeyId.NotFound" | "SignatureDoesNotMatch") => {
                ProviderError::InvalidCredentials {
                    provider: self.provider_name().to_string(),
                }
            }
            // 记录已存在
            Some("DomainRecordDuplicate") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("DomainRecordNotBelongToUser" | "InvalidRecordId.NotFound") => {
                ProviderError::RecordNotFound {
                    provider: self.provider_name().to_string(),
                    record_id: context.record_id.unwrap_or_default(),
                    raw_message: Some(raw.message),
                }
            }
            // 域名不存在
            Some("InvalidDomainName.NoExist") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}

impl AliyunProvider {
    pub fn new(credentials: &HashMap<String, String>) -> Self {
        let access_key_id = credentials.get("accessKeyId").cloned().unwrap_or_default();
        let access_key_secret = credentials
            .get("accessKeySecret")
            .cloned()
            .unwrap_or_default();
        let account_id = uuid::Uuid::new_v4().to_string();

        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            account_id,
        }
    }

    pub fn with_account_id(credentials: &HashMap<String, String>, account_id: String) -> Self {
        let access_key_id = credentials.get("accessKeyId").cloned().unwrap_or_default();
        let access_key_secret = credentials
            .get("accessKeySecret")
            .cloned()
            .unwrap_or_default();

        Self {
            client: Client::new(),
            access_key_id,
            access_key_secret,
            account_id,
        }
    }

    /// 生成 ACS3-HMAC-SHA256 签名
    /// 参考: <https://www.alibabacloud.com/help/zh/sdk/product-overview/v3-request-structure-and-signature>
    fn sign(&self, action: &str, query_string: &str, timestamp: &str, nonce: &str) -> String {
        // 1. 构造规范化请求头 (使用空 body 的 hash)
        let canonical_headers = format!(
            "host:{ALIYUN_DNS_HOST}\nx-acs-action:{action}\nx-acs-content-sha256:{EMPTY_BODY_SHA256}\nx-acs-date:{timestamp}\nx-acs-signature-nonce:{nonce}\nx-acs-version:{ALIYUN_DNS_VERSION}\n"
        );

        let signed_headers =
            "host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version";

        // 2. 构造规范化请求 (RPC 风格: 参数在 query string 中)
        let canonical_request = format!(
            "POST\n/\n{query_string}\n{canonical_headers}\n{signed_headers}\n{EMPTY_BODY_SHA256}"
        );

        log::debug!("CanonicalRequest:\n{canonical_request}");

        // 3. 构造待签名字符串
        let hashed_canonical_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_canonical_request}");

        log::debug!("StringToSign:\n{string_to_sign}");

        // 4. 计算签名
        let signature = hex::encode(Self::hmac_sha256(
            self.access_key_secret.as_bytes(),
            string_to_sign.as_bytes(),
        ));

        // 5. 构造 Authorization 头
        format!(
            "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
            self.access_key_id, signed_headers, signature
        )
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// 执行阿里云 API 请求 (RPC 风格: 参数通过 query string 传递)
    async fn request<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        action: &str,
        params: &B,
    ) -> Result<T> {
        // 1. 序列化参数为 query string
        let query_string = serialize_to_query_string(params)?;

        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = uuid::Uuid::new_v4().to_string();

        // 2. 生成签名 (使用 query string)
        let authorization = self.sign(action, &query_string, &timestamp, &nonce);

        // 3. 构造 URL (参数在 query string 中)
        let url = if query_string.is_empty() {
            format!("https://{ALIYUN_DNS_HOST}/")
        } else {
            format!("https://{ALIYUN_DNS_HOST}/?{query_string}")
        };

        log::debug!("POST {url} Action: {action}");

        // 4. 发送请求 (body 为空)
        let response = self
            .client
            .post(&url)
            .header("Host", ALIYUN_DNS_HOST)
            .header("x-acs-action", action)
            .header("x-acs-version", ALIYUN_DNS_VERSION)
            .header("x-acs-date", &timestamp)
            .header("x-acs-signature-nonce", &nonce)
            .header("x-acs-content-sha256", EMPTY_BODY_SHA256)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {status}");

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Body: {response_text}");

        // 先检查是否有错误响应
        if let Ok(error_response) = serde_json::from_str::<AliyunResponse<()>>(&response_text) {
            if let (Some(code), Some(message)) = (error_response.code, error_response.message) {
                log::error!("API 错误: {code} - {message}");
                return Err(self
                    .map_error(
                        RawApiError::with_code(&code, &message),
                        ErrorContext::default(),
                    )
                    .into());
            }
        }

        // 解析成功响应
        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            log::error!("原始响应: {response_text}");
            self.parse_error(e).into()
        })
    }

    /// 将阿里云域名状态转换为内部状态
    /// 注意：阿里云 `DescribeDomains` API 实际上不返回 `DomainStatus` 字段
    fn convert_domain_status(status: Option<&str>) -> DomainStatus {
        match status {
            Some("ENABLE" | "enable") => DomainStatus::Active,
            Some("PAUSE" | "pause") => DomainStatus::Paused,
            Some("SPAM" | "spam") => DomainStatus::Error,
            _ => DomainStatus::Unknown,
        }
    }

    /// 将阿里云记录类型转换为内部类型
    fn convert_record_type(record_type: &str) -> Result<DnsRecordType> {
        match record_type.to_uppercase().as_str() {
            "A" => Ok(DnsRecordType::A),
            "AAAA" => Ok(DnsRecordType::Aaaa),
            "CNAME" => Ok(DnsRecordType::Cname),
            "MX" => Ok(DnsRecordType::Mx),
            "TXT" => Ok(DnsRecordType::Txt),
            "NS" => Ok(DnsRecordType::Ns),
            "SRV" => Ok(DnsRecordType::Srv),
            "CAA" => Ok(DnsRecordType::Caa),
            _ => Err(ProviderError::InvalidParameter {
                provider: "aliyun".to_string(),
                param: "record_type".to_string(),
                detail: format!("不支持的记录类型: {record_type}"),
            }
            .into()),
        }
    }

    /// 将内部记录类型转换为阿里云 API 格式
    fn record_type_to_string(record_type: &DnsRecordType) -> String {
        match record_type {
            DnsRecordType::A => "A",
            DnsRecordType::Aaaa => "AAAA",
            DnsRecordType::Cname => "CNAME",
            DnsRecordType::Mx => "MX",
            DnsRecordType::Txt => "TXT",
            DnsRecordType::Ns => "NS",
            DnsRecordType::Srv => "SRV",
            DnsRecordType::Caa => "CAA",
        }
        .to_string()
    }

    /// 将时间戳转换为 RFC3339 格式
    fn timestamp_to_rfc3339(timestamp: Option<i64>) -> Option<String> {
        timestamp.and_then(|ts| DateTime::from_timestamp(ts / 1000, 0).map(|dt| dt.to_rfc3339()))
    }
}

#[async_trait]
impl DnsProvider for AliyunProvider {
    fn id(&self) -> &'static str {
        "aliyun"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        #[derive(Serialize)]
        struct DescribeDomainsRequest {
            #[serde(rename = "PageNumber")]
            page_number: u32,
            #[serde(rename = "PageSize")]
            page_size: u32,
        }

        let req = DescribeDomainsRequest {
            page_number: 1,
            page_size: 1,
        };

        match self
            .request::<DescribeDomainsResponse, _>("DescribeDomains", &req)
            .await
        {
            Ok(_) => Ok(true),
            Err(DnsError::Provider(ProviderError::InvalidCredentials { .. })) => Ok(false),
            Err(e) => {
                log::warn!("凭证验证失败: {e}");
                Ok(false)
            }
        }
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        #[derive(Serialize)]
        struct DescribeDomainsRequest {
            #[serde(rename = "PageNumber")]
            page_number: u32,
            #[serde(rename = "PageSize")]
            page_size: u32,
        }

        let req = DescribeDomainsRequest {
            page_number: params.page,
            page_size: params.page_size.min(100), // 阿里云最大支持 100
        };

        let response: DescribeDomainsResponse = self.request("DescribeDomains", &req).await?;

        let total_count = response.total_count.unwrap_or(0);
        let domains = response
            .domains
            .and_then(|d| d.domain)
            .unwrap_or_default()
            .into_iter()
            .map(|d| Domain {
                id: d.domain_id.unwrap_or_else(|| d.domain_name.clone()),
                name: d.domain_name,
                account_id: self.account_id.clone(),
                provider: crate::types::DnsProvider::Aliyun,
                status: Self::convert_domain_status(d.domain_status.as_deref()),
                record_count: d.record_count,
            })
            .collect();

        Ok(PaginatedResponse::new(
            domains,
            params.page,
            params.page_size,
            total_count,
        ))
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        // 阿里云 API 需要域名名称，先从域名列表中查找
        // 使用大页面一次性获取用于查找
        let params = PaginationParams {
            page: 1,
            page_size: 100,
        };
        let response = self.list_domains(&params).await?;

        response
            .items
            .into_iter()
            .find(|d| d.id == domain_id || d.name == domain_id)
            .ok_or_else(|| DnsError::DomainNotFound(domain_id.to_string()))
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        #[derive(Serialize)]
        struct DescribeDomainRecordsRequest {
            #[serde(rename = "DomainName")]
            domain_name: String,
            #[serde(rename = "PageNumber")]
            page_number: u32,
            #[serde(rename = "PageSize")]
            page_size: u32,
            /// 主机记录关键字（模糊搜索）
            #[serde(rename = "RRKeyWord", skip_serializing_if = "Option::is_none")]
            rr_keyword: Option<String>,
            /// 记录类型过滤
            #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
            record_type: Option<String>,
        }

        // 获取域名信息 (因为 API 需要域名名称而不是 ID)
        let domain_info = self.get_domain(domain_id).await?;

        let req = DescribeDomainRecordsRequest {
            domain_name: domain_info.name,
            page_number: params.page,
            page_size: params.page_size.min(100), // 阿里云最大支持 100
            rr_keyword: params.keyword.clone().filter(|k| !k.is_empty()),
            record_type: params.record_type.clone().filter(|t| !t.is_empty()),
        };

        let response: DescribeDomainRecordsResponse =
            self.request("DescribeDomainRecords", &req).await?;

        let total_count = response.total_count.unwrap_or(0);
        let records = response
            .domain_records
            .and_then(|r| r.record)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                let record_type = Self::convert_record_type(&r.record_type).ok()?;
                Some(DnsRecord {
                    id: r.record_id,
                    domain_id: domain_id.to_string(),
                    record_type,
                    name: r.rr,
                    value: r.value,
                    ttl: r.ttl,
                    priority: r.priority,
                    proxied: None, // 阿里云不支持代理
                    created_at: Self::timestamp_to_rfc3339(r.create_timestamp),
                    updated_at: Self::timestamp_to_rfc3339(r.update_timestamp),
                })
            })
            .collect();

        Ok(PaginatedResponse::new(
            records,
            params.page,
            params.page_size,
            total_count,
        ))
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        #[derive(Serialize)]
        struct AddDomainRecordRequest {
            #[serde(rename = "DomainName")]
            domain_name: String,
            #[serde(rename = "RR")]
            rr: String,
            #[serde(rename = "Type")]
            record_type: String,
            #[serde(rename = "Value")]
            value: String,
            #[serde(rename = "TTL")]
            ttl: u32,
            #[serde(rename = "Priority", skip_serializing_if = "Option::is_none")]
            priority: Option<u16>,
        }

        // 获取域名信息
        let domain_info = self.get_domain(&req.domain_id).await?;

        let api_req = AddDomainRecordRequest {
            domain_name: domain_info.name,
            rr: req.name.clone(),
            record_type: Self::record_type_to_string(&req.record_type),
            value: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
        };

        let response: AddDomainRecordResponse = self.request("AddDomainRecord", &api_req).await?;

        let now = chrono::Utc::now().to_rfc3339();
        Ok(DnsRecord {
            id: response.record_id,
            domain_id: req.domain_id.clone(),
            record_type: req.record_type.clone(),
            name: req.name.clone(),
            value: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
            proxied: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        #[derive(Serialize)]
        struct UpdateDomainRecordRequest {
            #[serde(rename = "RecordId")]
            record_id: String,
            #[serde(rename = "RR")]
            rr: String,
            #[serde(rename = "Type")]
            record_type: String,
            #[serde(rename = "Value")]
            value: String,
            #[serde(rename = "TTL")]
            ttl: u32,
            #[serde(rename = "Priority", skip_serializing_if = "Option::is_none")]
            priority: Option<u16>,
        }

        let api_req = UpdateDomainRecordRequest {
            record_id: record_id.to_string(),
            rr: req.name.clone(),
            record_type: Self::record_type_to_string(&req.record_type),
            value: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
        };

        let _response: UpdateDomainRecordResponse =
            self.request("UpdateDomainRecord", &api_req).await?;

        let now = chrono::Utc::now().to_rfc3339();
        Ok(DnsRecord {
            id: record_id.to_string(),
            domain_id: req.domain_id.clone(),
            record_type: req.record_type.clone(),
            name: req.name.clone(),
            value: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
            proxied: None,
            created_at: None,
            updated_at: Some(now),
        })
    }

    async fn delete_record(&self, record_id: &str, _domain_id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct DeleteDomainRecordRequest {
            #[serde(rename = "RecordId")]
            record_id: String,
        }

        let api_req = DeleteDomainRecordRequest {
            record_id: record_id.to_string(),
        };

        let _response: DeleteDomainRecordResponse =
            self.request("DeleteDomainRecord", &api_req).await?;

        Ok(())
    }
}
