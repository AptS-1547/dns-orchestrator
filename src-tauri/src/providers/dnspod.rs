use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::{DnsProvider, ErrorContext, ProviderErrorMapper, RawApiError};
use crate::error::{DnsError, ProviderError, Result};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, Domain, DomainStatus, PaginatedResponse,
    PaginationParams, RecordQueryParams, UpdateDnsRecordRequest,
};

const DNSPOD_API_HOST: &str = "dnspod.tencentcloudapi.com";
const DNSPOD_SERVICE: &str = "dnspod";
const DNSPOD_VERSION: &str = "2021-03-23";

type HmacSha256 = Hmac<Sha256>;

// ============ 腾讯云 API 响应结构 ============

#[derive(Debug, Deserialize)]
struct TencentResponse<T> {
    #[serde(rename = "Response")]
    response: TencentResponseInner<T>,
}

#[derive(Debug, Deserialize)]
struct TencentResponseInner<T> {
    #[serde(flatten)]
    data: Option<T>,
    #[serde(rename = "Error")]
    error: Option<TencentError>,
    #[serde(rename = "RequestId")]
    #[allow(dead_code)]
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct TencentError {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
}

// ============ DNSPod 域名相关结构 ============

#[derive(Debug, Deserialize)]
struct DomainListResponse {
    #[serde(rename = "DomainList")]
    domain_list: Option<Vec<DnspodDomain>>,
    #[serde(rename = "DomainCountInfo")]
    domain_count_info: Option<DomainCountInfo>,
}

#[derive(Debug, Deserialize)]
struct DomainCountInfo {
    #[serde(rename = "AllTotal")]
    all_total: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct DnspodDomain {
    #[serde(rename = "DomainId")]
    domain_id: u64,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "RecordCount")]
    record_count: Option<u32>,
}

// ============ DNSPod 记录相关结构 ============

#[derive(Debug, Deserialize)]
struct RecordListResponse {
    #[serde(rename = "RecordList")]
    record_list: Option<Vec<DnspodRecord>>,
    #[serde(rename = "RecordCountInfo")]
    record_count_info: Option<RecordCountInfo>,
}

#[derive(Debug, Deserialize)]
struct RecordCountInfo {
    #[serde(rename = "TotalCount")]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct DnspodRecord {
    #[serde(rename = "RecordId")]
    record_id: u64,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "TTL")]
    ttl: u32,
    #[serde(rename = "MX")]
    mx: Option<u16>,
    #[serde(rename = "UpdatedOn")]
    updated_on: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRecordResponse {
    #[serde(rename = "RecordId")]
    record_id: u64,
}

#[derive(Debug, Deserialize)]
struct ModifyRecordResponse {
    #[serde(rename = "RecordId")]
    record_id: u64,
}

// ============ DNSPod Provider 实现 ============

/// 腾讯云 `DNSPod` Provider
pub struct DnspodProvider {
    client: Client,
    secret_id: String,
    secret_key: String,
    account_id: String,
}

/// `DNSPod` 错误码映射
impl ProviderErrorMapper for DnspodProvider {
    fn provider_name(&self) -> &'static str {
        "dnspod"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("AuthFailure" | "AuthFailure.SecretIdNotFound") => {
                ProviderError::InvalidCredentials {
                    provider: self.provider_name().to_string(),
                }
            }
            // 记录已存在
            Some("ResourceInUse.RecordExist") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("ResourceNotFound.RecordNotExist") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 域名不存在
            Some("ResourceNotFound.NoDataOfDomain") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}

impl DnspodProvider {
    pub fn new(credentials: &HashMap<String, String>) -> Self {
        let secret_id = credentials.get("secretId").cloned().unwrap_or_default();
        let secret_key = credentials.get("secretKey").cloned().unwrap_or_default();
        let account_id = uuid::Uuid::new_v4().to_string();

        Self {
            client: Client::new(),
            secret_id,
            secret_key,
            account_id,
        }
    }

    pub fn with_account_id(credentials: &HashMap<String, String>, account_id: String) -> Self {
        let secret_id = credentials.get("secretId").cloned().unwrap_or_default();
        let secret_key = credentials.get("secretKey").cloned().unwrap_or_default();

        Self {
            client: Client::new(),
            secret_id,
            secret_key,
            account_id,
        }
    }

    /// 生成 TC3-HMAC-SHA256 签名
    fn sign(&self, action: &str, payload: &str, timestamp: i64) -> String {
        let date = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(Utc::now)
            .format("%Y-%m-%d")
            .to_string();

        // 1. 拼接规范请求串
        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_query_string = "";
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n",
            DNSPOD_API_HOST,
            action.to_lowercase()
        );
        let signed_headers = "content-type;host;x-tc-action";
        let hashed_payload = hex::encode(Sha256::digest(payload.as_bytes()));
        let canonical_request = format!(
            "{http_request_method}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers}\n{signed_headers}\n{hashed_payload}"
        );

        // 2. 拼接待签名字符串
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{date}/{DNSPOD_SERVICE}/tc3_request");
        let hashed_canonical_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign =
            format!("{algorithm}\n{timestamp}\n{credential_scope}\n{hashed_canonical_request}");

        // 3. 计算签名
        let secret_date = Self::hmac_sha256(
            format!("TC3{}", self.secret_key).as_bytes(),
            date.as_bytes(),
        );
        let secret_service = Self::hmac_sha256(&secret_date, DNSPOD_SERVICE.as_bytes());
        let secret_signing = Self::hmac_sha256(&secret_service, b"tc3_request");
        let signature = hex::encode(Self::hmac_sha256(
            &secret_signing,
            string_to_sign.as_bytes(),
        ));

        // 4. 拼接 Authorization
        format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        )
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// 执行腾讯云 API 请求
    async fn request<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        action: &str,
        body: &B,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| DnsError::SerializationError(e.to_string()))?;

        let timestamp = Utc::now().timestamp();
        let authorization = self.sign(action, &payload, timestamp);

        let url = format!("https://{DNSPOD_API_HOST}");
        log::debug!("POST {url} Action: {action}");
        log::debug!("Request Body: {payload}");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", DNSPOD_API_HOST)
            .header("X-TC-Action", action)
            .header("X-TC-Version", DNSPOD_VERSION)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("Authorization", authorization)
            .body(payload)
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

        let tc_response: TencentResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {e}");
                log::error!("原始响应: {response_text}");
                self.parse_error(e)
            })?;

        if let Some(error) = tc_response.response.error {
            log::error!("API 错误: {} - {}", error.code, error.message);
            return Err(self
                .map_error(
                    RawApiError::with_code(&error.code, &error.message),
                    ErrorContext::default(),
                )
                .into());
        }

        tc_response
            .response
            .data
            .ok_or_else(|| self.parse_error("响应中缺少数据").into())
    }

    /// 将 `DNSPod` 域名状态转换为内部状态
    fn convert_domain_status(status: &str) -> DomainStatus {
        match status {
            "ENABLE" | "enable" => DomainStatus::Active,
            "PAUSE" | "pause" => DomainStatus::Paused,
            "SPAM" | "spam" => DomainStatus::Error,
            _ => DomainStatus::Unknown,
        }
    }

    /// 将 `DNSPod` 记录类型转换为内部类型
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
                provider: "dnspod".to_string(),
                param: "record_type".to_string(),
                detail: format!("不支持的记录类型: {record_type}"),
            }
            .into()),
        }
    }

    /// 将内部记录类型转换为 `DNSPod` API 格式
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
}

#[async_trait]
impl DnsProvider for DnspodProvider {
    fn id(&self) -> &'static str {
        "dnspod"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // 通过获取域名列表来验证凭证
        #[derive(Serialize)]
        struct DescribeDomainListRequest {
            #[serde(rename = "Offset")]
            offset: u32,
            #[serde(rename = "Limit")]
            limit: u32,
        }

        let req = DescribeDomainListRequest {
            offset: 0,
            limit: 1,
        };

        match self
            .request::<DomainListResponse, _>("DescribeDomainList", &req)
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
        struct DescribeDomainListRequest {
            #[serde(rename = "Offset")]
            offset: u32,
            #[serde(rename = "Limit")]
            limit: u32,
        }

        // 将 page/page_size 转换为 offset/limit
        let offset = (params.page - 1) * params.page_size;
        let req = DescribeDomainListRequest {
            offset,
            limit: params.page_size.min(100),
        };

        let response: DomainListResponse = self.request("DescribeDomainList", &req).await?;

        let total_count = response
            .domain_count_info
            .and_then(|c| c.all_total)
            .unwrap_or(0);

        let domains = response
            .domain_list
            .unwrap_or_default()
            .into_iter()
            .map(|d| Domain {
                id: d.domain_id.to_string(),
                name: d.name,
                account_id: self.account_id.clone(),
                provider: crate::types::DnsProvider::Dnspod,
                status: Self::convert_domain_status(&d.status),
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
        // DNSPod API 需要域名字符串，先从域名列表中查找
        let params = PaginationParams {
            page: 1,
            page_size: 100,
        };
        let response = self.list_domains(&params).await?;

        response
            .items
            .into_iter()
            .find(|d| d.id == domain_id)
            .ok_or_else(|| DnsError::DomainNotFound(domain_id.to_string()))
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        #[derive(Serialize)]
        struct DescribeRecordListRequest {
            #[serde(rename = "Domain")]
            domain: String,
            #[serde(rename = "Offset")]
            offset: u32,
            #[serde(rename = "Limit")]
            limit: u32,
            /// 主机头关键字（模糊搜索）
            #[serde(rename = "Keyword", skip_serializing_if = "Option::is_none")]
            keyword: Option<String>,
            /// 记录类型过滤
            #[serde(rename = "RecordType", skip_serializing_if = "Option::is_none")]
            record_type: Option<String>,
        }

        // 先获取域名信息以获取域名名称
        let domain_info = self.get_domain(domain_id).await?;

        // 将 page/page_size 转换为 offset/limit
        let offset = (params.page - 1) * params.page_size;
        let req = DescribeRecordListRequest {
            domain: domain_info.name,
            offset,
            limit: params.page_size.min(100),
            keyword: params.keyword.clone().filter(|k| !k.is_empty()),
            record_type: params.record_type.clone().filter(|t| !t.is_empty()),
        };

        // DNSPod API 在记录为空时返回错误而不是空列表，需要特殊处理
        let response: Result<RecordListResponse> = self.request("DescribeRecordList", &req).await;

        match response {
            Ok(data) => {
                let total_count = data
                    .record_count_info
                    .and_then(|c| c.total_count)
                    .unwrap_or(0);

                let records = data
                    .record_list
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|r| {
                        let record_type = Self::convert_record_type(&r.record_type).ok()?;
                        Some(DnsRecord {
                            id: r.record_id.to_string(),
                            domain_id: domain_id.to_string(),
                            record_type,
                            name: r.name,
                            value: r.value,
                            ttl: r.ttl,
                            priority: r.mx,
                            proxied: None,
                            created_at: None,
                            updated_at: r.updated_on,
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
            // "NoDataOfRecord" 表示记录列表为空，返回空结果而不是错误
            Err(DnsError::Provider(ProviderError::Unknown { raw_code, .. }))
                if raw_code.as_deref() == Some("ResourceNotFound.NoDataOfRecord") =>
            {
                Ok(PaginatedResponse::new(
                    vec![],
                    params.page,
                    params.page_size,
                    0,
                ))
            }
            Err(e) => Err(e),
        }
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        #[derive(Serialize)]
        struct CreateRecordRequest {
            #[serde(rename = "Domain")]
            domain: String,
            #[serde(rename = "SubDomain")]
            sub_domain: String,
            #[serde(rename = "RecordType")]
            record_type: String,
            #[serde(rename = "RecordLine")]
            record_line: String,
            #[serde(rename = "Value")]
            value: String,
            #[serde(rename = "TTL")]
            ttl: u32,
            #[serde(rename = "MX", skip_serializing_if = "Option::is_none")]
            mx: Option<u16>,
        }

        // 获取域名信息
        let domain_info = self.get_domain(&req.domain_id).await?;

        let api_req = CreateRecordRequest {
            domain: domain_info.name,
            sub_domain: req.name.clone(),
            record_type: Self::record_type_to_string(&req.record_type),
            record_line: "默认".to_string(),
            value: req.value.clone(),
            ttl: req.ttl,
            mx: req.priority,
        };

        let response: CreateRecordResponse = self.request("CreateRecord", &api_req).await?;

        let now = chrono::Utc::now().to_rfc3339();
        Ok(DnsRecord {
            id: response.record_id.to_string(),
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
        struct ModifyRecordRequest {
            #[serde(rename = "Domain")]
            domain: String,
            #[serde(rename = "RecordId")]
            record_id: u64,
            #[serde(rename = "SubDomain")]
            sub_domain: String,
            #[serde(rename = "RecordType")]
            record_type: String,
            #[serde(rename = "RecordLine")]
            record_line: String,
            #[serde(rename = "Value")]
            value: String,
            #[serde(rename = "TTL")]
            ttl: u32,
            #[serde(rename = "MX", skip_serializing_if = "Option::is_none")]
            mx: Option<u16>,
        }

        let record_id_num: u64 = record_id
            .parse()
            .map_err(|_| DnsError::RecordNotFound(record_id.to_string()))?;

        // 获取域名信息
        let domain_info = self.get_domain(&req.domain_id).await?;

        let api_req = ModifyRecordRequest {
            domain: domain_info.name,
            record_id: record_id_num,
            sub_domain: req.name.clone(),
            record_type: Self::record_type_to_string(&req.record_type),
            record_line: "默认".to_string(),
            value: req.value.clone(),
            ttl: req.ttl,
            mx: req.priority,
        };

        let _response: ModifyRecordResponse = self.request("ModifyRecord", &api_req).await?;

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

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        #[derive(Serialize)]
        struct DeleteRecordRequest {
            #[serde(rename = "Domain")]
            domain: String,
            #[serde(rename = "RecordId")]
            record_id: u64,
        }

        #[derive(Debug, Deserialize)]
        struct DeleteRecordResponse {}

        let record_id_num: u64 = record_id
            .parse()
            .map_err(|_| DnsError::RecordNotFound(record_id.to_string()))?;

        // 获取域名信息
        let domain_info = self.get_domain(domain_id).await?;

        let api_req = DeleteRecordRequest {
            domain: domain_info.name,
            record_id: record_id_num,
        };

        let _response: DeleteRecordResponse = self.request("DeleteRecord", &api_req).await?;

        Ok(())
    }
}
