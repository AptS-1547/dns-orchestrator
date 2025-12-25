//! 华为云 DnsProvider trait 实现

use async_trait::async_trait;
use serde::Serialize;

use crate::error::{ProviderError, Result};
use crate::providers::common::{
    full_name_to_relative, normalize_domain_name, record_type_to_string, relative_to_full_name,
};
use crate::traits::{DnsProvider, ErrorContext};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DomainStatus, FieldType, PaginatedResponse,
    PaginationParams, ProviderCredentialField, ProviderDomain, ProviderFeatures, ProviderLimits,
    ProviderMetadata, ProviderType, RecordData, RecordQueryParams, UpdateDnsRecordRequest,
};

use super::types::{
    CreateRecordSetResponse, ListRecordSetsResponse, ListZonesResponse, ShowPublicZoneResponse,
};
use super::{HuaweicloudProvider, MAX_PAGE_SIZE};

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

    /// 解析华为云记录为 RecordData
    /// 华为云格式：MX/SRV/CAA 的所有字段都编码在 records 字符串中
    fn parse_record_data(record_type: &str, record: &str) -> Result<RecordData> {
        match record_type {
            "A" => Ok(RecordData::A {
                address: record.to_string(),
            }),
            "AAAA" => Ok(RecordData::AAAA {
                address: record.to_string(),
            }),
            "CNAME" => Ok(RecordData::CNAME {
                target: record.to_string(),
            }),
            "MX" => {
                // 华为云 MX 格式: "priority exchange"
                let parts: Vec<&str> = record.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    Ok(RecordData::MX {
                        priority: parts[0].parse().map_err(|_| ProviderError::ParseError {
                            provider: "huaweicloud".to_string(),
                            detail: format!("Invalid MX priority: '{}'", parts[0]),
                        })?,
                        exchange: parts[1].to_string(),
                    })
                } else {
                    Err(ProviderError::ParseError {
                        provider: "huaweicloud".to_string(),
                        detail: format!(
                            "Invalid MX record format: expected 'priority exchange', got '{record}'"
                        ),
                    })
                }
            }
            "TXT" => Ok(RecordData::TXT {
                text: record.to_string(),
            }),
            "NS" => Ok(RecordData::NS {
                nameserver: record.to_string(),
            }),
            "SRV" => {
                // 华为云 SRV 格式: "priority weight port target"
                let parts: Vec<&str> = record.splitn(4, ' ').collect();
                if parts.len() == 4 {
                    Ok(RecordData::SRV {
                        priority: parts[0].parse().map_err(|_| ProviderError::ParseError {
                            provider: "huaweicloud".to_string(),
                            detail: format!("Invalid SRV priority: '{}'", parts[0]),
                        })?,
                        weight: parts[1].parse().map_err(|_| ProviderError::ParseError {
                            provider: "huaweicloud".to_string(),
                            detail: format!("Invalid SRV weight: '{}'", parts[1]),
                        })?,
                        port: parts[2].parse().map_err(|_| ProviderError::ParseError {
                            provider: "huaweicloud".to_string(),
                            detail: format!("Invalid SRV port: '{}'", parts[2]),
                        })?,
                        target: parts[3].to_string(),
                    })
                } else {
                    Err(ProviderError::ParseError {
                        provider: "huaweicloud".to_string(),
                        detail: format!(
                            "Invalid SRV record format: expected 'priority weight port target', got '{record}'"
                        ),
                    })
                }
            }
            "CAA" => {
                // 华为云 CAA 格式: "flags tag value"
                let parts: Vec<&str> = record.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    Ok(RecordData::CAA {
                        flags: parts[0].parse().map_err(|_| ProviderError::ParseError {
                            provider: "huaweicloud".to_string(),
                            detail: format!("Invalid CAA flags: '{}'", parts[0]),
                        })?,
                        tag: parts[1].to_string(),
                        value: parts[2].trim_matches('"').to_string(),
                    })
                } else {
                    Err(ProviderError::ParseError {
                        provider: "huaweicloud".to_string(),
                        detail: format!(
                            "Invalid CAA record format: expected 'flags tag value', got '{record}'"
                        ),
                    })
                }
            }
            _ => Err(ProviderError::UnsupportedRecordType {
                provider: "huaweicloud".to_string(),
                record_type: record_type.to_string(),
            }),
        }
    }

    /// 将 RecordData 转换为华为云 API 格式（records 字符串）
    fn record_data_to_record_string(data: &RecordData) -> String {
        match data {
            RecordData::A { address } => address.clone(),
            RecordData::AAAA { address } => address.clone(),
            RecordData::CNAME { target } => target.clone(),
            RecordData::MX { priority, exchange } => format!("{priority} {exchange}"),
            RecordData::TXT { text } => text.clone(),
            RecordData::NS { nameserver } => nameserver.clone(),
            RecordData::SRV {
                priority,
                weight,
                port,
                target,
            } => format!("{priority} {weight} {port} {target}"),
            RecordData::CAA { flags, tag, value } => format!("{flags} {tag} \"{value}\""),
        }
    }
}

#[async_trait]
impl DnsProvider for HuaweicloudProvider {
    fn id(&self) -> &'static str {
        "huaweicloud"
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderType::Huaweicloud,
            name: "华为云 DNS".to_string(),
            description: "华为云云解析服务".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "accessKeyId".to_string(),
                    label: "Access Key ID".to_string(),
                    field_type: FieldType::Text,
                    placeholder: Some("输入 Access Key ID".to_string()),
                    help_text: None,
                },
                ProviderCredentialField {
                    key: "secretAccessKey".to_string(),
                    label: "Secret Access Key".to_string(),
                    field_type: FieldType::Password,
                    placeholder: Some("输入 Secret Access Key".to_string()),
                    help_text: None,
                },
            ],
            features: ProviderFeatures::default(),
            limits: ProviderLimits {
                max_page_size_domains: 500,
                max_page_size_records: 500,
            },
        }
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

                let value = r.records.as_ref()?.first()?.clone();
                let data = Self::parse_record_data(&r.record_type, &value).ok()?;

                Some(DnsRecord {
                    id: r.id,
                    domain_id: domain_id.to_string(),
                    name: full_name_to_relative(&r.name, &domain_info.name),
                    ttl: r.ttl.unwrap_or(300),
                    data,
                    proxied: None,
                    created_at: r.created_at.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    updated_at: r.updated_at.and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
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

        // 构造记录值
        let record_value = Self::record_data_to_record_string(&req.data);
        let record_type = record_type_to_string(&req.data.record_type());

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
            record_type: record_type.to_string(),
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

        let now = chrono::Utc::now();
        Ok(DnsRecord {
            id: response.id,
            domain_id: req.domain_id.clone(),
            name: req.name.clone(),
            ttl: req.ttl,
            data: req.data.clone(),
            proxied: None,
            created_at: Some(now),
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

        // 构造记录值
        let record_value = Self::record_data_to_record_string(&req.data);
        let record_type = record_type_to_string(&req.data.record_type());

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
            record_type: record_type.to_string(),
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

        let now = chrono::Utc::now();
        Ok(DnsRecord {
            id: record_id.to_string(),
            domain_id: req.domain_id.clone(),
            name: req.name.clone(),
            ttl: req.ttl,
            data: req.data.clone(),
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
