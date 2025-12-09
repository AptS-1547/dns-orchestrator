use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DnsProvider, ErrorContext, ProviderErrorMapper, RawApiError};
use crate::error::{ProviderError, Result};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, Domain, DomainStatus, PaginatedResponse,
    PaginationParams, RecordQueryParams, UpdateDnsRecordRequest,
};

const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API 通用响应
#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    result: Option<T>,
    errors: Option<Vec<CloudflareError>>,
    result_info: Option<CloudflareResultInfo>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    #[allow(dead_code)]
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareResultInfo {
    #[allow(dead_code)]
    page: u32,
    #[allow(dead_code)]
    per_page: u32,
    total_count: u32,
}

/// Cloudflare Zone 结构
#[derive(Debug, Deserialize)]
struct CloudflareZone {
    id: String,
    name: String,
    status: String,
}

/// Cloudflare DNS Record 结构
#[derive(Debug, Deserialize, Serialize)]
struct CloudflareDnsRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_on: Option<String>,
}

/// Cloudflare DNS Provider
pub struct CloudflareProvider {
    client: Client,
    api_token: String,
    account_id: String,
}

/// Cloudflare 错误码映射
/// 参考: https://api.cloudflare.com/#getting-started-responses
impl ProviderErrorMapper for CloudflareProvider {
    fn provider_name(&self) -> &'static str {
        "cloudflare"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        // Cloudflare 错误码映射
        match raw.code.as_deref() {
            // 认证错误
            Some("9109") | Some("10000") => ProviderError::InvalidCredentials {
                provider: self.provider_name().to_string(),
            },
            // 记录已存在
            Some("81057") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("81044") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // Zone 不存在
            Some("7003") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}

impl CloudflareProvider {
    pub fn new(credentials: HashMap<String, String>) -> Self {
        let api_token = credentials.get("apiToken").cloned().unwrap_or_default();

        let account_id = uuid::Uuid::new_v4().to_string();

        Self {
            client: Client::new(),
            api_token,
            account_id,
        }
    }

    pub fn with_account_id(credentials: HashMap<String, String>, account_id: String) -> Self {
        let api_token = credentials.get("apiToken").cloned().unwrap_or_default();

        Self {
            client: Client::new(),
            api_token,
            account_id,
        }
    }

    /// 执行 GET 请求
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", CF_API_BASE, path);
        log::debug!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {}", status);

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        log::debug!("Response Body: {}", response_text);

        let cf_response: CloudflareResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {}", e);
                log::error!("原始响应: {}", response_text);
                self.parse_error(e)
            })?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {}", message);
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段").into())
    }

    /// 执行 GET 请求 (带分页)
    async fn get_paginated<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        params: &PaginationParams,
    ) -> Result<(Vec<T>, u32)> {
        // Cloudflare zones API 最大 per_page 是 50
        let url = format!(
            "{}{}?page={}&per_page={}",
            CF_API_BASE,
            path,
            params.page,
            params.page_size.min(50)
        );
        log::debug!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {}", status);

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        log::debug!("Response Body: {}", response_text);

        let cf_response: CloudflareResponse<Vec<T>> = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("JSON 解析失败: {}", e);
                log::error!("原始响应: {}", response_text);
                self.parse_error(e)
            })?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {}", message);
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        let total_count = cf_response.result_info.map(|i| i.total_count).unwrap_or(0);
        let items = cf_response.result.unwrap_or_default();

        Ok((items, total_count))
    }

    /// 执行 POST 请求
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", CF_API_BASE, path);
        let body_json =
            serde_json::to_string_pretty(body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("POST {}", url);
        log::debug!("Request Body: {}", body_json);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(body)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {}", status);

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        log::debug!("Response Body: {}", response_text);

        let cf_response: CloudflareResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {}", e);
                log::error!("原始响应: {}", response_text);
                self.parse_error(e)
            })?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {}", message);
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段").into())
    }

    /// 执行 PATCH 请求
    async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", CF_API_BASE, path);
        let body_json =
            serde_json::to_string_pretty(body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("PATCH {}", url);
        log::debug!("Request Body: {}", body_json);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(body)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {}", status);

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        log::debug!("Response Body: {}", response_text);

        let cf_response: CloudflareResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {}", e);
                log::error!("原始响应: {}", response_text);
                self.parse_error(e)
            })?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {}", message);
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段").into())
    }

    /// 执行 DELETE 请求
    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", CF_API_BASE, path);
        log::debug!("DELETE {}", url);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {}", status);

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        log::debug!("Response Body: {}", response_text);

        let cf_response: CloudflareResponse<serde_json::Value> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {}", e);
                log::error!("原始响应: {}", response_text);
                self.parse_error(e)
            })?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {}", message);
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        Ok(())
    }

    /// 将 Cloudflare zone 转换为 Domain
    /// Cloudflare 状态：active, pending, initializing, moved
    fn zone_to_domain(&self, zone: CloudflareZone) -> Domain {
        let status = match zone.status.as_str() {
            "active" => DomainStatus::Active,
            "pending" | "initializing" => DomainStatus::Pending,
            "moved" => DomainStatus::Paused,
            _ => DomainStatus::Unknown,
        };

        Domain {
            id: zone.id,
            name: zone.name,
            account_id: self.account_id.clone(),
            provider: crate::types::DnsProvider::Cloudflare,
            status,
            record_count: None,
        }
    }

    /// 将完整域名转换为相对名称 (用于显示)
    /// 如: "www.example.com" + "example.com" -> "www"
    /// 如: "example.com" + "example.com" -> "@"
    fn full_name_to_relative(&self, full_name: &str, zone_name: &str) -> String {
        if full_name == zone_name {
            "@".to_string()
        } else if let Some(subdomain) = full_name.strip_suffix(&format!(".{}", zone_name)) {
            subdomain.to_string()
        } else {
            full_name.to_string()
        }
    }

    /// 将相对名称转换为完整域名 (用于 API 调用)
    /// 如: "www" + "example.com" -> "www.example.com"
    /// 如: "@" + "example.com" -> "example.com"
    fn relative_to_full_name(&self, relative_name: &str, zone_name: &str) -> String {
        if relative_name == "@" || relative_name.is_empty() {
            zone_name.to_string()
        } else {
            format!("{}.{}", relative_name, zone_name)
        }
    }

    /// 将 Cloudflare 记录转换为 DnsRecord
    fn cf_record_to_dns_record(
        &self,
        cf_record: CloudflareDnsRecord,
        zone_id: &str,
        zone_name: &str,
    ) -> Result<DnsRecord> {
        let record_type = match cf_record.record_type.as_str() {
            "A" => DnsRecordType::A,
            "AAAA" => DnsRecordType::Aaaa,
            "CNAME" => DnsRecordType::Cname,
            "MX" => DnsRecordType::Mx,
            "TXT" => DnsRecordType::Txt,
            "NS" => DnsRecordType::Ns,
            "SRV" => DnsRecordType::Srv,
            "CAA" => DnsRecordType::Caa,
            _ => {
                return Err(ProviderError::InvalidParameter {
                    provider: self.provider_name().to_string(),
                    param: "record_type".to_string(),
                    detail: format!("不支持的记录类型: {}", cf_record.record_type),
                }.into())
            }
        };

        Ok(DnsRecord {
            id: cf_record.id,
            domain_id: zone_id.to_string(),
            record_type,
            name: self.full_name_to_relative(&cf_record.name, zone_name),
            value: cf_record.content,
            ttl: cf_record.ttl,
            priority: cf_record.priority,
            proxied: cf_record.proxied,
            created_at: cf_record.created_on,
            updated_at: cf_record.modified_on,
        })
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    fn id(&self) -> &'static str {
        "cloudflare"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        #[derive(Deserialize)]
        struct VerifyResponse {
            status: String,
        }

        match self.get::<VerifyResponse>("/user/tokens/verify").await {
            Ok(resp) => Ok(resp.status == "active"),
            Err(_) => Ok(false),
        }
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        let (zones, total_count): (Vec<CloudflareZone>, u32) =
            self.get_paginated("/zones", params).await?;
        let domains = zones.into_iter().map(|z| self.zone_to_domain(z)).collect();
        Ok(PaginatedResponse::new(
            domains,
            params.page,
            params.page_size,
            total_count,
        ))
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        let zone: CloudflareZone = self.get(&format!("/zones/{}", domain_id)).await?;
        Ok(self.zone_to_domain(zone))
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        // 先获取 zone 信息以获取域名
        let zone: CloudflareZone = self.get(&format!("/zones/{}", domain_id)).await?;
        let zone_name = zone.name;

        // 构建查询 URL，包含搜索参数
        let mut url = format!(
            "/zones/{}/dns_records?page={}&per_page={}",
            domain_id,
            params.page,
            params.page_size.min(100) // Cloudflare 最大 per_page 是 5000000, 但为了性能考虑这里限制为 100
        );

        // 添加搜索关键词（只搜索记录名称）
        if let Some(ref keyword) = params.keyword {
            if !keyword.is_empty() {
                url.push_str(&format!("&name.contains={}", urlencoding::encode(keyword)));
            }
        }

        // 添加记录类型过滤
        if let Some(ref record_type) = params.record_type {
            if !record_type.is_empty() {
                url.push_str(&format!("&type={}", urlencoding::encode(record_type)));
            }
        }

        log::debug!("GET {}{}", CF_API_BASE, url);

        let response = self
            .client
            .get(format!("{}{}", CF_API_BASE, url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {}", e)))?;

        let cf_response: CloudflareResponse<Vec<CloudflareDnsRecord>> =
            serde_json::from_str(&response_text)
                .map_err(|e| self.parse_error(e))?;

        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| errors.first().map(|e| (e.code.to_string(), e.message.clone())))
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            return Err(self.map_error(
                RawApiError::with_code(code, message),
                ErrorContext::default(),
            ).into());
        }

        let total_count = cf_response.result_info.map(|i| i.total_count).unwrap_or(0);
        let cf_records = cf_response.result.unwrap_or_default();

        let records: Result<Vec<DnsRecord>> = cf_records
            .into_iter()
            .map(|r| self.cf_record_to_dns_record(r, domain_id, &zone_name))
            .collect();

        Ok(PaginatedResponse::new(
            records?,
            params.page,
            params.page_size,
            total_count,
        ))
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        // 先获取 zone 信息
        let zone: CloudflareZone = self.get(&format!("/zones/{}", req.domain_id)).await?;
        let zone_name = zone.name;

        let full_name = self.relative_to_full_name(&req.name, &zone_name);

        #[derive(Serialize)]
        struct CreateRecordBody {
            #[serde(rename = "type")]
            record_type: String,
            name: String,
            content: String,
            ttl: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            priority: Option<u16>,
            #[serde(skip_serializing_if = "Option::is_none")]
            proxied: Option<bool>,
        }

        let body = CreateRecordBody {
            record_type: format!("{:?}", req.record_type).to_uppercase(),
            name: full_name,
            content: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
            proxied: req.proxied,
        };

        let cf_record: CloudflareDnsRecord = self
            .post(&format!("/zones/{}/dns_records", req.domain_id), &body)
            .await?;

        self.cf_record_to_dns_record(cf_record, &req.domain_id, &zone_name)
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        // 先获取 zone 信息
        let zone: CloudflareZone = self.get(&format!("/zones/{}", req.domain_id)).await?;
        let zone_name = zone.name;

        let full_name = self.relative_to_full_name(&req.name, &zone_name);

        #[derive(Serialize)]
        struct UpdateRecordBody {
            #[serde(rename = "type")]
            record_type: String,
            name: String,
            content: String,
            ttl: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            priority: Option<u16>,
            #[serde(skip_serializing_if = "Option::is_none")]
            proxied: Option<bool>,
        }

        let body = UpdateRecordBody {
            record_type: format!("{:?}", req.record_type).to_uppercase(),
            name: full_name,
            content: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
            proxied: req.proxied,
        };

        let cf_record: CloudflareDnsRecord = self
            .patch(
                &format!("/zones/{}/dns_records/{}", req.domain_id, record_id),
                &body,
            )
            .await?;

        self.cf_record_to_dns_record(cf_record, &req.domain_id, &zone_name)
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        self.delete(&format!("/zones/{}/dns_records/{}", domain_id, record_id))
            .await
    }
}
