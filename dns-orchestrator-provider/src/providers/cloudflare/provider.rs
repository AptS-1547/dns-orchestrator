//! Cloudflare DnsProvider trait 实现

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::providers::common::{
    full_name_to_relative, parse_record_type, record_type_to_string, relative_to_full_name,
};
use crate::traits::{DnsProvider, ErrorContext, ProviderErrorMapper};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, DomainStatus, FieldType, PaginatedResponse,
    PaginationParams, ProviderCredentialField, ProviderDomain, ProviderFeatures, ProviderLimits,
    ProviderMetadata, ProviderType, RecordQueryParams, UpdateDnsRecordRequest,
};

use super::{CloudflareDnsRecord, CloudflareProvider, CloudflareZone, MAX_PAGE_SIZE_RECORDS};

impl CloudflareProvider {
    /// 将 Cloudflare zone 转换为 ProviderDomain
    /// Cloudflare 状态：active, pending, initializing, moved
    pub(crate) fn zone_to_domain(zone: CloudflareZone) -> ProviderDomain {
        let status = match zone.status.as_str() {
            "active" => DomainStatus::Active,
            "pending" | "initializing" => DomainStatus::Pending,
            "moved" => DomainStatus::Paused,
            _ => DomainStatus::Unknown,
        };

        ProviderDomain {
            id: zone.id,
            name: zone.name,
            provider: ProviderType::Cloudflare,
            status,
            record_count: None,
        }
    }

    /// 将 Cloudflare 记录转换为 `DnsRecord`
    pub(crate) fn cf_record_to_dns_record(
        &self,
        cf_record: CloudflareDnsRecord,
        zone_id: &str,
        zone_name: &str,
    ) -> Result<DnsRecord> {
        let record_type = parse_record_type(&cf_record.record_type, self.provider_name())?;
        // 只有 MX 和 SRV 记录才有 priority，其他类型忽略
        let priority = match record_type {
            DnsRecordType::Mx | DnsRecordType::Srv => cf_record.priority,
            _ => None,
        };

        Ok(DnsRecord {
            id: cf_record.id,
            domain_id: zone_id.to_string(),
            record_type,
            name: full_name_to_relative(&cf_record.name, zone_name),
            value: cf_record.content,
            ttl: cf_record.ttl,
            priority,
            proxied: cf_record.proxied,
            created_at: cf_record.created_on.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            updated_at: cf_record.modified_on.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        })
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    fn id(&self) -> &'static str {
        "cloudflare"
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderType::Cloudflare,
            name: "Cloudflare".to_string(),
            description: "全球领先的 CDN 和 DNS 服务商".to_string(),
            required_fields: vec![ProviderCredentialField {
                key: "apiToken".to_string(),
                label: "API Token".to_string(),
                field_type: FieldType::Password,
                placeholder: Some("输入 Cloudflare API Token".to_string()),
                help_text: Some(
                    "在 Cloudflare Dashboard -> My Profile -> API Tokens 创建".to_string(),
                ),
            }],
            features: ProviderFeatures { proxy: true },
            limits: ProviderLimits {
                max_page_size_domains: 50,
                max_page_size_records: 5000,
            },
        }
    }

    async fn validate_credentials(&self) -> Result<bool> {
        #[derive(Deserialize)]
        struct VerifyResponse {
            status: String,
        }

        match self
            .get::<VerifyResponse>("/user/tokens/verify", ErrorContext::default())
            .await
        {
            Ok(resp) => Ok(resp.status == "active"),
            Err(_) => Ok(false),
        }
    }

    async fn list_domains(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<ProviderDomain>> {
        let (zones, total_count): (Vec<CloudflareZone>, u32) = self
            .get_paginated("/zones", params, ErrorContext::default())
            .await?;
        let domains = zones.into_iter().map(Self::zone_to_domain).collect();
        Ok(PaginatedResponse::new(
            domains,
            params.page,
            params.page_size,
            total_count,
        ))
    }

    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain> {
        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };
        let zone: CloudflareZone = self.get(&format!("/zones/{domain_id}"), ctx).await?;
        Ok(Self::zone_to_domain(zone))
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };

        // 先获取 zone 信息以获取域名
        let zone: CloudflareZone = self
            .get(&format!("/zones/{domain_id}"), ctx.clone())
            .await?;
        let zone_name = zone.name;

        // 构建查询 URL，包含搜索参数
        let mut url = format!(
            "/zones/{}/dns_records?page={}&per_page={}",
            domain_id,
            params.page,
            params.page_size.min(MAX_PAGE_SIZE_RECORDS)
        );

        // 添加搜索关键词（只搜索记录名称）
        if let Some(ref keyword) = params.keyword
            && !keyword.is_empty()
        {
            url.push_str(&format!("&name.contains={}", urlencoding::encode(keyword)));
        }

        // 添加记录类型过滤
        if let Some(ref record_type) = params.record_type {
            let type_str = record_type_to_string(record_type);
            url.push_str(&format!("&type={}", urlencoding::encode(type_str)));
        }

        let (cf_records, total_count) = self.get_records(&url, ctx).await?;

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
        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            domain: Some(req.domain_id.clone()),
            ..Default::default()
        };

        // 先获取 zone 信息
        let zone: CloudflareZone = self
            .get(&format!("/zones/{}", req.domain_id), ctx.clone())
            .await?;
        let zone_name = zone.name;

        let full_name = relative_to_full_name(&req.name, &zone_name);

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
            record_type: record_type_to_string(&req.record_type).to_string(),
            name: full_name,
            content: req.value.clone(),
            ttl: req.ttl,
            priority: req.priority,
            proxied: req.proxied,
        };

        let cf_record: CloudflareDnsRecord = self
            .post(&format!("/zones/{}/dns_records", req.domain_id), &body, ctx)
            .await?;

        self.cf_record_to_dns_record(cf_record, &req.domain_id, &zone_name)
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            record_id: Some(record_id.to_string()),
            domain: Some(req.domain_id.clone()),
        };

        // 先获取 zone 信息
        let zone: CloudflareZone = self
            .get(&format!("/zones/{}", req.domain_id), ctx.clone())
            .await?;
        let zone_name = zone.name;

        let full_name = relative_to_full_name(&req.name, &zone_name);

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
            record_type: record_type_to_string(&req.record_type).to_string(),
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
                ctx,
            )
            .await?;

        self.cf_record_to_dns_record(cf_record, &req.domain_id, &zone_name)
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        let ctx = ErrorContext {
            record_id: Some(record_id.to_string()),
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };
        self.delete(&format!("/zones/{domain_id}/dns_records/{record_id}"), ctx)
            .await
    }
}
