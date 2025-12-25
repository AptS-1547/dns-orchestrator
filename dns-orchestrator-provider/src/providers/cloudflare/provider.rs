//! Cloudflare DnsProvider trait 实现

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::Result;
use crate::providers::common::{full_name_to_relative, relative_to_full_name};
use crate::traits::{DnsProvider, ErrorContext, ProviderErrorMapper};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DomainStatus, FieldType, PaginatedResponse,
    PaginationParams, ProviderCredentialField, ProviderDomain, ProviderFeatures, ProviderLimits,
    ProviderMetadata, ProviderType, RecordData, RecordQueryParams, UpdateDnsRecordRequest,
};

use super::{
    CloudflareCaaData, CloudflareDnsRecord, CloudflareProvider, CloudflareSrvData, CloudflareZone,
    MAX_PAGE_SIZE_RECORDS,
};

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
        let data = self.parse_record_data(&cf_record)?;

        Ok(DnsRecord {
            id: cf_record.id,
            domain_id: zone_id.to_string(),
            name: full_name_to_relative(&cf_record.name, zone_name),
            ttl: cf_record.ttl,
            data,
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

    /// 解析 Cloudflare 记录为 RecordData
    fn parse_record_data(&self, cf_record: &CloudflareDnsRecord) -> Result<RecordData> {
        match cf_record.record_type.as_str() {
            "A" => Ok(RecordData::A {
                address: cf_record.content.clone(),
            }),
            "AAAA" => Ok(RecordData::AAAA {
                address: cf_record.content.clone(),
            }),
            "CNAME" => Ok(RecordData::CNAME {
                target: cf_record.content.clone(),
            }),
            "MX" => Ok(RecordData::MX {
                priority: cf_record.priority.ok_or_else(|| {
                    crate::error::ProviderError::ParseError {
                        provider: self.provider_name().to_string(),
                        detail: "MX record missing priority field".to_string(),
                    }
                })?,
                exchange: cf_record.content.clone(),
            }),
            "TXT" => Ok(RecordData::TXT {
                text: cf_record.content.clone(),
            }),
            "NS" => Ok(RecordData::NS {
                nameserver: cf_record.content.clone(),
            }),
            "SRV" => {
                // SRV 记录使用 data 字段
                if let Some(ref data) = cf_record.data {
                    let srv: CloudflareSrvData =
                        serde_json::from_value(data.clone()).map_err(|e| {
                            crate::error::ProviderError::ParseError {
                                provider: self.provider_name().to_string(),
                                detail: format!("Failed to parse SRV data: {e}"),
                            }
                        })?;
                    Ok(RecordData::SRV {
                        priority: srv.priority,
                        weight: srv.weight,
                        port: srv.port,
                        target: srv.target,
                    })
                } else {
                    // Fallback: 尝试从 content 解析
                    let parts: Vec<&str> = cf_record.content.split_whitespace().collect();
                    if parts.len() >= 3 {
                        Ok(RecordData::SRV {
                            priority: cf_record.priority.ok_or_else(|| {
                                crate::error::ProviderError::ParseError {
                                    provider: self.provider_name().to_string(),
                                    detail: "SRV record missing priority field".to_string(),
                                }
                            })?,
                            weight: parts[0].parse().map_err(|_| {
                                crate::error::ProviderError::ParseError {
                                    provider: self.provider_name().to_string(),
                                    detail: format!("Invalid SRV weight: '{}'", parts[0]),
                                }
                            })?,
                            port: parts[1].parse().map_err(|_| {
                                crate::error::ProviderError::ParseError {
                                    provider: self.provider_name().to_string(),
                                    detail: format!("Invalid SRV port: '{}'", parts[1]),
                                }
                            })?,
                            target: parts[2].to_string(),
                        })
                    } else {
                        Err(crate::error::ProviderError::ParseError {
                            provider: self.provider_name().to_string(),
                            detail: format!(
                                "Invalid SRV record format: expected 'weight port target', got '{}'",
                                cf_record.content
                            ),
                        })
                    }
                }
            }
            "CAA" => {
                // CAA 记录使用 data 字段
                if let Some(ref data) = cf_record.data {
                    let caa: CloudflareCaaData =
                        serde_json::from_value(data.clone()).map_err(|e| {
                            crate::error::ProviderError::ParseError {
                                provider: self.provider_name().to_string(),
                                detail: format!("Failed to parse CAA data: {e}"),
                            }
                        })?;
                    Ok(RecordData::CAA {
                        flags: caa.flags,
                        tag: caa.tag,
                        value: caa.value,
                    })
                } else {
                    // Fallback: 尝试从 content 解析 "flags tag value"
                    let parts: Vec<&str> = cf_record.content.splitn(3, ' ').collect();
                    if parts.len() >= 3 {
                        Ok(RecordData::CAA {
                            flags: parts[0].parse().map_err(|_| {
                                crate::error::ProviderError::ParseError {
                                    provider: self.provider_name().to_string(),
                                    detail: format!("Invalid CAA flags: '{}'", parts[0]),
                                }
                            })?,
                            tag: parts[1].to_string(),
                            value: parts[2].trim_matches('"').to_string(),
                        })
                    } else {
                        Err(crate::error::ProviderError::ParseError {
                            provider: self.provider_name().to_string(),
                            detail: format!(
                                "Invalid CAA record format: expected 'flags tag value', got '{}'",
                                cf_record.content
                            ),
                        })
                    }
                }
            }
            _ => Err(crate::error::ProviderError::UnsupportedRecordType {
                provider: self.provider_name().to_string(),
                record_type: cf_record.record_type.clone(),
            }),
        }
    }

    /// 将 RecordData 转换为 Cloudflare API 请求体
    fn build_create_body(
        &self,
        full_name: &str,
        ttl: u32,
        data: &RecordData,
        proxied: Option<bool>,
    ) -> serde_json::Value {
        match data {
            RecordData::A { address } => serde_json::json!({
                "type": "A",
                "name": full_name,
                "content": address,
                "ttl": ttl,
                "proxied": proxied,
            }),
            RecordData::AAAA { address } => serde_json::json!({
                "type": "AAAA",
                "name": full_name,
                "content": address,
                "ttl": ttl,
                "proxied": proxied,
            }),
            RecordData::CNAME { target } => serde_json::json!({
                "type": "CNAME",
                "name": full_name,
                "content": target,
                "ttl": ttl,
                "proxied": proxied,
            }),
            RecordData::MX { priority, exchange } => serde_json::json!({
                "type": "MX",
                "name": full_name,
                "content": exchange,
                "ttl": ttl,
                "priority": priority,
            }),
            RecordData::TXT { text } => serde_json::json!({
                "type": "TXT",
                "name": full_name,
                "content": text,
                "ttl": ttl,
            }),
            RecordData::NS { nameserver } => serde_json::json!({
                "type": "NS",
                "name": full_name,
                "content": nameserver,
                "ttl": ttl,
            }),
            RecordData::SRV {
                priority,
                weight,
                port,
                target,
            } => serde_json::json!({
                "type": "SRV",
                "name": full_name,
                "ttl": ttl,
                "data": {
                    "priority": priority,
                    "weight": weight,
                    "port": port,
                    "target": target,
                }
            }),
            RecordData::CAA { flags, tag, value } => serde_json::json!({
                "type": "CAA",
                "name": full_name,
                "ttl": ttl,
                "data": {
                    "flags": flags,
                    "tag": tag,
                    "value": value,
                }
            }),
        }
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
            let type_str = crate::providers::common::record_type_to_string(record_type);
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
        let body = self.build_create_body(&full_name, req.ttl, &req.data, req.proxied);

        let cf_record: CloudflareDnsRecord = self
            .post_json(&format!("/zones/{}/dns_records", req.domain_id), body, ctx)
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
        let body = self.build_create_body(&full_name, req.ttl, &req.data, req.proxied);

        let cf_record: CloudflareDnsRecord = self
            .patch_json(
                &format!("/zones/{}/dns_records/{}", req.domain_id, record_id),
                body,
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
