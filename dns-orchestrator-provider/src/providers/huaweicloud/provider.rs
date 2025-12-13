//! 华为云 DnsProvider trait 实现

use async_trait::async_trait;
use serde::Serialize;

use crate::error::{ProviderError, Result};
use crate::providers::common::{
    full_name_to_relative, normalize_domain_name, parse_record_type, record_type_to_string,
    relative_to_full_name,
};
use crate::traits::{DnsProvider, ErrorContext};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DnsRecordType, DomainStatus, PaginatedResponse,
    PaginationParams, ProviderDomain, ProviderType, RecordQueryParams, UpdateDnsRecordRequest,
};

use super::HuaweicloudProvider;
use super::MAX_PAGE_SIZE;
use super::types::{
    CreateRecordSetResponse, ListRecordSetsResponse, ListZonesResponse, ShowPublicZoneResponse,
};

impl HuaweicloudProvider {
    /// 将华为云域名状态转换为内部状态
    /// 华为云状态：ACTIVE, `PENDING_CREATE`, `PENDING_UPDATE`, `PENDING_DELETE`,
    /// `PENDING_FREEZE`, FREEZE, ILLEGAL, POLICE, `PENDING_DISABLE`, DISABLE, ERROR
    pub(crate) fn convert_domain_status(status: Option<&str>) -> DomainStatus {
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
}

#[async_trait]
impl DnsProvider for HuaweicloudProvider {
    fn id(&self) -> &'static str {
        "huaweicloud"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        match self
            .get::<ListZonesResponse>("/v2/zones", "type=public&limit=1", ErrorContext::default())
            .await
        {
            Ok(_) => Ok(true),
            Err(ProviderError::InvalidCredentials { .. }) => Ok(false),
            Err(e) => {
                log::warn!("凭证验证失败: {e}");
                Ok(false)
            }
        }
    }

    async fn list_domains(
        &self,
        params: &PaginationParams,
    ) -> Result<PaginatedResponse<ProviderDomain>> {
        // 华为云使用 offset/limit 分页
        let offset = (params.page - 1) * params.page_size;
        let limit = params.page_size.min(MAX_PAGE_SIZE);
        let query = format!("type=public&offset={offset}&limit={limit}");

        let response: ListZonesResponse = self
            .get("/v2/zones", &query, ErrorContext::default())
            .await?;

        let total_count = response.metadata.and_then(|m| m.total_count).unwrap_or(0);

        let domains = response
            .zones
            .unwrap_or_default()
            .into_iter()
            .map(|z| ProviderDomain {
                id: z.id,
                name: normalize_domain_name(&z.name),
                provider: ProviderType::Huaweicloud,
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

    /// 使用 ShowPublicZone API 直接获取域名信息
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain> {
        let path = format!("/v2/zones/{domain_id}");
        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };
        let response: ShowPublicZoneResponse = self.get(&path, "", ctx).await?;

        Ok(ProviderDomain {
            id: response.id,
            name: normalize_domain_name(&response.name),
            provider: ProviderType::Huaweicloud,
            status: Self::convert_domain_status(response.status.as_deref()),
            record_count: response.record_num,
        })
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
        let limit = params.page_size.min(MAX_PAGE_SIZE);
        let mut query = format!("offset={offset}&limit={limit}");

        // 添加搜索关键词（华为云支持 name 参数模糊匹配）
        if let Some(ref keyword) = params.keyword
            && !keyword.is_empty()
        {
            query.push_str(&format!("&name={}", urlencoding::encode(keyword)));
        }

        // 添加记录类型过滤
        if let Some(ref record_type) = params.record_type {
            let type_str = record_type_to_string(record_type);
            query.push_str(&format!("&type={}", urlencoding::encode(type_str)));
        }

        let path = format!("/v2/zones/{domain_id}/recordsets");
        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };
        let response: ListRecordSetsResponse = self.get(&path, &query, ctx).await?;

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

                let record_type = parse_record_type(&r.record_type, "huaweicloud").ok()?;
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
                    name: full_name_to_relative(&r.name, &domain_info.name),
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

        // 构造完整的记录名称（华为云需要末尾带点）
        let full_name = format!("{}.", relative_to_full_name(&req.name, &domain_info.name));

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
            record_type: record_type_to_string(&req.record_type).to_string(),
            records: vec![record_value],
            ttl: req.ttl,
        };

        let path = format!("/v2/zones/{}/recordsets", req.domain_id);
        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            domain: Some(req.domain_id.clone()),
            ..Default::default()
        };
        let response: CreateRecordSetResponse = self.post(&path, &api_req, ctx).await?;

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

        // 构造完整的记录名称（华为云需要末尾带点）
        let full_name = format!("{}.", relative_to_full_name(&req.name, &domain_info.name));

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
            record_type: record_type_to_string(&req.record_type).to_string(),
            records: vec![record_value],
            ttl: req.ttl,
        };

        let path = format!("/v2/zones/{}/recordsets/{}", req.domain_id, record_id);
        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            record_id: Some(record_id.to_string()),
            domain: Some(req.domain_id.clone()),
        };
        let _response: CreateRecordSetResponse = self.put(&path, &api_req, ctx).await?;

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
        let ctx = ErrorContext {
            record_id: Some(record_id.to_string()),
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };
        self.delete(&path, ctx).await
    }
}
