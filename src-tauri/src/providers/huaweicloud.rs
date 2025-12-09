use async_trait::async_trait;
use chrono::Utc;
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

const HUAWEICLOUD_DNS_HOST: &str = "dns.myhuaweicloud.com";

type HmacSha256 = Hmac<Sha256>;

// ============ 华为云 API 响应结构 ============

#[derive(Debug, Deserialize)]
struct ListZonesResponse {
    zones: Option<Vec<HuaweicloudZone>>,
    #[serde(rename = "metadata")]
    metadata: Option<ListMetadata>,
}

#[derive(Debug, Deserialize)]
struct ListMetadata {
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct HuaweicloudZone {
    id: String,
    name: String,
    status: Option<String>,
    record_num: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ListRecordSetsResponse {
    recordsets: Option<Vec<HuaweicloudRecordSet>>,
    #[serde(rename = "metadata")]
    metadata: Option<ListMetadata>,
}

#[derive(Debug, Deserialize)]
struct HuaweicloudRecordSet {
    id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    records: Option<Vec<String>>,
    ttl: Option<u32>,
    status: Option<String>,
    #[serde(rename = "created_at")]
    created_at: Option<String>,
    #[serde(rename = "updated_at")]
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRecordSetResponse {
    id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    records: Option<Vec<String>>,
    ttl: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error_code: Option<String>,
    error_msg: Option<String>,
}

// ============ 华为云 DNS Provider 实现 ============

pub struct HuaweicloudProvider {
    client: Client,
    access_key_id: String,
    secret_access_key: String,
    account_id: String,
}

/// 华为云错误码映射
impl ProviderErrorMapper for HuaweicloudProvider {
    fn provider_name(&self) -> &'static str {
        "huaweicloud"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("APIGW.0301" | "APIGW.0101") => ProviderError::InvalidCredentials {
                provider: self.provider_name().to_string(),
            },
            // 记录已存在
            Some("DNS.0312") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("DNS.0305") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // Zone 不存在
            Some("DNS.0101") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}

impl HuaweicloudProvider {
    pub fn new(credentials: &HashMap<String, String>) -> Self {
        let access_key_id = credentials.get("accessKeyId").cloned().unwrap_or_default();
        let secret_access_key = credentials
            .get("secretAccessKey")
            .cloned()
            .unwrap_or_default();
        let account_id = uuid::Uuid::new_v4().to_string();

        Self {
            client: Client::new(),
            access_key_id,
            secret_access_key,
            account_id,
        }
    }

    /// 生成华为云 SDK 签名
    /// 参考: <https://support.huaweicloud.com/devg-apisign/api-sign-algorithm-005.html>
    fn sign(
        &self,
        method: &str,
        uri: &str,
        query: &str,
        headers: &[(String, String)],
        payload: &str,
        timestamp: &str,
    ) -> String {
        // 1. URI 规范化：确保以 "/" 结尾
        let canonical_uri = if uri.ends_with('/') {
            uri.to_string()
        } else {
            format!("{uri}/")
        };

        // 2. Query String 排序（按参数名升序）
        let canonical_query = if query.is_empty() {
            String::new()
        } else {
            let mut params: Vec<&str> = query.split('&').collect();
            params.sort_unstable();
            params.join("&")
        };

        // 3. 构造规范请求头
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let canonical_headers: String = sorted_headers
            .iter()
            .map(|(k, v)| format!("{}:{}\n", k.to_lowercase(), v.trim()))
            .collect();

        let signed_headers: String = sorted_headers
            .iter()
            .map(|(k, _)| k.to_lowercase())
            .collect::<Vec<_>>()
            .join(";");

        // 4. 计算 payload hash
        let hashed_payload = hex::encode(Sha256::digest(payload.as_bytes()));

        // 5. 构造规范请求
        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{hashed_payload}"
        );

        log::debug!("CanonicalRequest:\n{canonical_request}");

        // 6. 构造待签名字符串（3 行格式）
        let hashed_canonical_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!("SDK-HMAC-SHA256\n{timestamp}\n{hashed_canonical_request}");

        log::debug!("StringToSign:\n{string_to_sign}");

        // 7. 计算签名（直接用 SK）
        let signature = hex::encode(Self::hmac_sha256(
            self.secret_access_key.as_bytes(),
            string_to_sign.as_bytes(),
        ));

        // 8. 构造 Authorization 头（正确格式：Access=xxx）
        format!(
            "SDK-HMAC-SHA256 Access={}, SignedHeaders={}, Signature={}",
            self.access_key_id, signed_headers, signature
        )
    }

    fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// 执行 GET 请求
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str, query: &str) -> Result<T> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("GET", path, query, &headers, "", &timestamp);

        let url = if query.is_empty() {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}")
        } else {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}?{query}")
        };

        log::debug!("GET {url}");

        let response = self
            .client
            .get(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self
                    .map_error(
                        RawApiError::with_code(
                            error.error_code.unwrap_or_default(),
                            error.error_msg.unwrap_or_default(),
                        ),
                        ErrorContext::default(),
                    )
                    .into());
            }
            return Err(self
                .unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
                .into());
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e).into()
        })
    }

    /// 执行 POST 请求
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| DnsError::SerializationError(e.to_string()))?;

        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let authorization = self.sign("POST", path, "", &headers, &payload, &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("POST {url} Body: {payload}");

        let response = self
            .client
            .post(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self
                    .map_error(
                        RawApiError::with_code(
                            error.error_code.unwrap_or_default(),
                            error.error_msg.unwrap_or_default(),
                        ),
                        ErrorContext::default(),
                    )
                    .into());
            }
            return Err(self
                .unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
                .into());
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e).into()
        })
    }

    /// 执行 PUT 请求
    async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| DnsError::SerializationError(e.to_string()))?;

        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let authorization = self.sign("PUT", path, "", &headers, &payload, &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("PUT {url} Body: {payload}");

        let response = self
            .client
            .put(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self
                    .map_error(
                        RawApiError::with_code(
                            error.error_code.unwrap_or_default(),
                            error.error_msg.unwrap_or_default(),
                        ),
                        ErrorContext::default(),
                    )
                    .into());
            }
            return Err(self
                .unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
                .into());
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e).into()
        })
    }

    /// 执行 DELETE 请求
    async fn delete(&self, path: &str) -> Result<()> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("DELETE", path, "", &headers, "", &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("DELETE {url}");

        let response = self
            .client
            .delete(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();

        if !status.is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self
                    .map_error(
                        RawApiError::with_code(
                            error.error_code.unwrap_or_default(),
                            error.error_msg.unwrap_or_default(),
                        ),
                        ErrorContext::default(),
                    )
                    .into());
            }
            return Err(self
                .unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
                .into());
        }

        Ok(())
    }

    /// 将华为云域名状态转换为内部状态
    /// 华为云状态：ACTIVE, `PENDING_CREATE`, `PENDING_UPDATE`, `PENDING_DELETE`,
    /// `PENDING_FREEZE`, FREEZE, ILLEGAL, POLICE, `PENDING_DISABLE`, DISABLE, ERROR
    fn convert_domain_status(status: Option<&str>) -> DomainStatus {
        match status {
            Some("ACTIVE") => DomainStatus::Active,
            // 各种 PENDING 状态
            Some(
                "PENDING_CREATE" | "PENDING_UPDATE" | "PENDING_DELETE" | "PENDING_FREEZE"
                | "PENDING_DISABLE",
            ) => DomainStatus::Pending,
            // 冻结/暂停状态
            Some("FREEZE" | "ILLEGAL" | "POLICE" | "DISABLE") => DomainStatus::Paused,
            Some("ERROR") => DomainStatus::Error,
            _ => DomainStatus::Unknown,
        }
    }

    /// 将华为云记录类型转换为内部类型
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
                provider: "huaweicloud".to_string(),
                param: "record_type".to_string(),
                detail: format!("不支持的记录类型: {record_type}"),
            }
            .into()),
        }
    }

    /// 将内部记录类型转换为华为云 API 格式
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

    /// 从域名名称中提取根域名（去掉末尾的点）
    fn normalize_domain_name(name: &str) -> String {
        name.trim_end_matches('.').to_string()
    }

    /// 从记录名称中提取子域名部分
    fn extract_subdomain(record_name: &str, domain_name: &str) -> String {
        let record = Self::normalize_domain_name(record_name);
        let domain = Self::normalize_domain_name(domain_name);

        if record == domain {
            "@".to_string()
        } else if record.ends_with(&format!(".{domain}")) {
            record[..record.len() - domain.len() - 1].to_string()
        } else {
            record
        }
    }
}

#[async_trait]
impl DnsProvider for HuaweicloudProvider {
    fn id(&self) -> &'static str {
        "huaweicloud"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        match self
            .get::<ListZonesResponse>("/v2/zones", "type=public&limit=1")
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
        // 华为云使用 offset/limit 分页
        let offset = (params.page - 1) * params.page_size;
        let limit = params.page_size.min(500); // 华为云最大支持 500
        let query = format!("type=public&offset={offset}&limit={limit}");

        let response: ListZonesResponse = self.get("/v2/zones", &query).await?;

        let total_count = response.metadata.and_then(|m| m.total_count).unwrap_or(0);

        let domains = response
            .zones
            .unwrap_or_default()
            .into_iter()
            .map(|z| Domain {
                id: z.id,
                name: Self::normalize_domain_name(&z.name),
                account_id: self.account_id.clone(),
                provider: crate::types::DnsProvider::Huaweicloud,
                status: Self::convert_domain_status(z.status.as_deref()),
                record_count: z.record_num,
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
        // 获取域名信息以获取域名名称
        let domain_info = self.get_domain(domain_id).await?;

        // 华为云使用 offset/limit 分页
        let offset = (params.page - 1) * params.page_size;
        let limit = params.page_size.min(500); // 华为云最大支持 500
        let mut query = format!("offset={offset}&limit={limit}");

        // 添加搜索关键词（华为云支持 name 参数模糊匹配）
        if let Some(ref keyword) = params.keyword {
            if !keyword.is_empty() {
                query.push_str(&format!("&name={}", urlencoding::encode(keyword)));
            }
        }

        // 添加记录类型过滤
        if let Some(ref record_type) = params.record_type {
            if !record_type.is_empty() {
                query.push_str(&format!("&type={}", urlencoding::encode(record_type)));
            }
        }

        let path = format!("/v2/zones/{domain_id}/recordsets");
        let response: ListRecordSetsResponse = self.get(&path, &query).await?;

        let total_count = response.metadata.and_then(|m| m.total_count).unwrap_or(0);

        let records = response
            .recordsets
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                // 跳过 SOA 和 NS 根记录
                if r.record_type == "SOA" {
                    return None;
                }

                let record_type = Self::convert_record_type(&r.record_type).ok()?;
                let value = r.records.as_ref()?.first()?.clone();

                // 提取优先级（对于 MX 记录）
                let (priority, actual_value) = if r.record_type == "MX" {
                    let parts: Vec<&str> = value.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        (parts[0].parse().ok(), parts[1].to_string())
                    } else {
                        (None, value)
                    }
                } else {
                    (None, value)
                };

                Some(DnsRecord {
                    id: r.id,
                    domain_id: domain_id.to_string(),
                    record_type,
                    name: Self::extract_subdomain(&r.name, &domain_info.name),
                    value: actual_value,
                    ttl: r.ttl.unwrap_or(300),
                    priority,
                    proxied: None,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
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
        // 获取域名信息
        let domain_info = self.get_domain(&req.domain_id).await?;

        // 构造完整的记录名称
        let full_name = if req.name == "@" || req.name.is_empty() {
            format!("{}.", domain_info.name)
        } else {
            format!("{}.{}.", req.name, domain_info.name)
        };

        // 构造记录值（MX 需要包含优先级）
        let record_value = if req.record_type == DnsRecordType::Mx {
            format!("{} {}", req.priority.unwrap_or(10), req.value)
        } else {
            req.value.clone()
        };

        #[derive(Serialize)]
        struct CreateRecordSetRequest {
            name: String,
            #[serde(rename = "type")]
            record_type: String,
            records: Vec<String>,
            ttl: u32,
        }

        let api_req = CreateRecordSetRequest {
            name: full_name,
            record_type: Self::record_type_to_string(&req.record_type),
            records: vec![record_value],
            ttl: req.ttl,
        };

        let path = format!("/v2/zones/{}/recordsets", req.domain_id);
        let response: CreateRecordSetResponse = self.post(&path, &api_req).await?;

        let now = chrono::Utc::now().to_rfc3339();
        Ok(DnsRecord {
            id: response.id,
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
        // 获取域名信息
        let domain_info = self.get_domain(&req.domain_id).await?;

        // 构造完整的记录名称
        let full_name = if req.name == "@" || req.name.is_empty() {
            format!("{}.", domain_info.name)
        } else {
            format!("{}.{}.", req.name, domain_info.name)
        };

        // 构造记录值（MX 需要包含优先级）
        let record_value = if req.record_type == DnsRecordType::Mx {
            format!("{} {}", req.priority.unwrap_or(10), req.value)
        } else {
            req.value.clone()
        };

        #[derive(Serialize)]
        struct UpdateRecordSetRequest {
            name: String,
            #[serde(rename = "type")]
            record_type: String,
            records: Vec<String>,
            ttl: u32,
        }

        let api_req = UpdateRecordSetRequest {
            name: full_name,
            record_type: Self::record_type_to_string(&req.record_type),
            records: vec![record_value],
            ttl: req.ttl,
        };

        let path = format!("/v2/zones/{}/recordsets/{}", req.domain_id, record_id);
        let _response: CreateRecordSetResponse = self.put(&path, &api_req).await?;

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
        let path = format!("/v2/zones/{domain_id}/recordsets/{record_id}");
        self.delete(&path).await
    }
}
