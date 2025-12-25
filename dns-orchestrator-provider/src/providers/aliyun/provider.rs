//! 阿里云 DnsProvider trait 实现

use async_trait::async_trait;
use chrono::DateTime;
use serde::Serialize;

use crate::error::{ProviderError, Result};
use crate::providers::common::record_type_to_string;
use crate::traits::{DnsProvider, ErrorContext};
use crate::types::{
    CreateDnsRecordRequest, DnsRecord, DomainStatus, FieldType, PaginatedResponse,
    PaginationParams, ProviderCredentialField, ProviderDomain, ProviderFeatures, ProviderLimits,
    ProviderMetadata, ProviderType, RecordData, RecordQueryParams, UpdateDnsRecordRequest,
};

use super::{
    AddDomainRecordResponse, AliyunProvider, DeleteDomainRecordResponse,
    DescribeDomainInfoResponse, DescribeDomainRecordsResponse, DescribeDomainsResponse,
    MAX_PAGE_SIZE, UpdateDomainRecordResponse,
};

impl AliyunProvider {
    /// 将阿里云域名状态转换为内部状态
    /// 注意：阿里云 `DescribeDomains` API 实际上不返回 `DomainStatus` 字段
    pub(crate) fn convert_domain_status(status: Option<&str>) -> DomainStatus {
        match status {
            Some("ENABLE" | "enable") => DomainStatus::Active,
            Some("PAUSE" | "pause") => DomainStatus::Paused,
            Some("SPAM" | "spam") => DomainStatus::Error,
            _ => DomainStatus::Unknown,
        }
    }

    /// 将阿里云的 Unix 毫秒时间戳转换为 DateTime<Utc>
    pub(crate) fn timestamp_to_datetime(timestamp: Option<i64>) -> Option<DateTime<chrono::Utc>> {
        timestamp.and_then(DateTime::from_timestamp_millis)
    }

    /// 解析阿里云记录为 RecordData
    fn parse_record_data(
        record_type: &str,
        value: &str,
        priority: Option<u16>,
    ) -> Result<RecordData> {
        match record_type {
            "A" => Ok(RecordData::A {
                address: value.to_string(),
            }),
            "AAAA" => Ok(RecordData::AAAA {
                address: value.to_string(),
            }),
            "CNAME" => Ok(RecordData::CNAME {
                target: value.to_string(),
            }),
            "MX" => Ok(RecordData::MX {
                priority: priority.ok_or_else(|| ProviderError::ParseError {
                    provider: "aliyun".to_string(),
                    detail: "MX record missing priority field".to_string(),
                })?,
                exchange: value.to_string(),
            }),
            "TXT" => Ok(RecordData::TXT {
                text: value.to_string(),
            }),
            "NS" => Ok(RecordData::NS {
                nameserver: value.to_string(),
            }),
            "SRV" => {
                // 阿里云 SRV value 格式: "priority weight port target"（所有字段都在 value 中）
                let parts: Vec<&str> = value.splitn(4, ' ').collect();
                if parts.len() == 4 {
                    Ok(RecordData::SRV {
                        priority: parts[0].parse().map_err(|_| ProviderError::ParseError {
                            provider: "aliyun".to_string(),
                            detail: format!("Invalid SRV priority: '{}'", parts[0]),
                        })?,
                        weight: parts[1].parse().map_err(|_| ProviderError::ParseError {
                            provider: "aliyun".to_string(),
                            detail: format!("Invalid SRV weight: '{}'", parts[1]),
                        })?,
                        port: parts[2].parse().map_err(|_| ProviderError::ParseError {
                            provider: "aliyun".to_string(),
                            detail: format!("Invalid SRV port: '{}'", parts[2]),
                        })?,
                        target: parts[3].to_string(),
                    })
                } else {
                    Err(ProviderError::ParseError {
                        provider: "aliyun".to_string(),
                        detail: format!(
                            "Invalid SRV record format: expected 'priority weight port target', got '{value}'"
                        ),
                    })
                }
            }
            "CAA" => {
                // 阿里云 CAA value 格式: "flags tag value"
                let parts: Vec<&str> = value.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    Ok(RecordData::CAA {
                        flags: parts[0].parse().map_err(|_| ProviderError::ParseError {
                            provider: "aliyun".to_string(),
                            detail: format!("Invalid CAA flags: '{}'", parts[0]),
                        })?,
                        tag: parts[1].to_string(),
                        value: parts[2].trim_matches('"').to_string(),
                    })
                } else {
                    Err(ProviderError::ParseError {
                        provider: "aliyun".to_string(),
                        detail: format!(
                            "Invalid CAA record format: expected 'flags tag value', got '{value}'"
                        ),
                    })
                }
            }
            _ => Err(ProviderError::UnsupportedRecordType {
                provider: "aliyun".to_string(),
                record_type: record_type.to_string(),
            }),
        }
    }

    /// 将 RecordData 转换为阿里云 API 格式 (value, priority)
    fn record_data_to_api(data: &RecordData) -> (String, Option<u16>) {
        match data {
            RecordData::A { address } => (address.clone(), None),
            RecordData::AAAA { address } => (address.clone(), None),
            RecordData::CNAME { target } => (target.clone(), None),
            RecordData::MX { priority, exchange } => (exchange.clone(), Some(*priority)),
            RecordData::TXT { text } => (text.clone(), None),
            RecordData::NS { nameserver } => (nameserver.clone(), None),
            // 阿里云 SRV：所有字段都在 Value 中（priority weight port target）
            RecordData::SRV {
                priority,
                weight,
                port,
                target,
            } => (format!("{priority} {weight} {port} {target}"), None),
            RecordData::CAA { flags, tag, value } => (format!("{flags} {tag} \"{value}\""), None),
        }
    }
}

#[async_trait]
impl DnsProvider for AliyunProvider {
    fn id(&self) -> &'static str {
        "aliyun"
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderType::Aliyun,
            name: "阿里云 DNS".to_string(),
            description: "阿里云域名解析服务".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "accessKeyId".to_string(),
                    label: "AccessKey ID".to_string(),
                    field_type: FieldType::Text,
                    placeholder: Some("输入 AccessKey ID".to_string()),
                    help_text: None,
                },
                ProviderCredentialField {
                    key: "accessKeySecret".to_string(),
                    label: "AccessKey Secret".to_string(),
                    field_type: FieldType::Password,
                    placeholder: Some("输入 AccessKey Secret".to_string()),
                    help_text: None,
                },
            ],
            features: ProviderFeatures::default(),
            limits: ProviderLimits {
                max_page_size_domains: 100,
                max_page_size_records: 100,
            },
        }
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
            .request::<DescribeDomainsResponse, _>("DescribeDomains", &req, ErrorContext::default())
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
        #[derive(Serialize)]
        struct DescribeDomainsRequest {
            #[serde(rename = "PageNumber")]
            page_number: u32,
            #[serde(rename = "PageSize")]
            page_size: u32,
        }

        let req = DescribeDomainsRequest {
            page_number: params.page,
            page_size: params.page_size.min(MAX_PAGE_SIZE),
        };

        let response: DescribeDomainsResponse = self
            .request("DescribeDomains", &req, ErrorContext::default())
            .await?;

        let total_count = response.total_count.unwrap_or(0);
        let domains = response
            .domains
            .and_then(|d| d.domain)
            .unwrap_or_default()
            .into_iter()
            .map(|d| ProviderDomain {
                // 阿里云 API 使用域名名称作为标识符，而非 domain_id
                id: d.domain_name.clone(),
                name: d.domain_name,
                provider: ProviderType::Aliyun,
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

    /// ErrorRequireCheck: 使用 DescribeDomainInfo API 直接获取域名信息
    /// 注意：阿里云 API 需要域名名称作为参数
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain> {
        #[derive(Serialize)]
        struct DescribeDomainInfoRequest {
            #[serde(rename = "DomainName")]
            domain_name: String,
        }

        let req = DescribeDomainInfoRequest {
            domain_name: domain_id.to_string(),
        };

        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };

        let response: DescribeDomainInfoResponse =
            self.request("DescribeDomainInfo", &req, ctx).await?;

        Ok(ProviderDomain {
            // 统一使用域名名称作为 ID，与 list_domains 保持一致
            id: response.domain_name.clone(),
            name: response.domain_name,
            provider: ProviderType::Aliyun,
            status: Self::convert_domain_status(response.domain_status.as_deref()),
            record_count: response.record_count,
        })
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

        // 阿里云的 domain_id 就是域名名称，可以直接使用
        let req = DescribeDomainRecordsRequest {
            domain_name: domain_id.to_string(),
            page_number: params.page,
            page_size: params.page_size.min(MAX_PAGE_SIZE),
            rr_keyword: params.keyword.clone().filter(|k| !k.is_empty()),
            record_type: params
                .record_type
                .as_ref()
                .map(|t| record_type_to_string(t).to_string()),
        };

        let ctx = ErrorContext {
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };

        let response: DescribeDomainRecordsResponse =
            self.request("DescribeDomainRecords", &req, ctx).await?;

        let total_count = response.total_count.unwrap_or(0);
        let records = response
            .domain_records
            .and_then(|r| r.record)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|r| {
                let data = Self::parse_record_data(&r.record_type, &r.value, r.priority).ok()?;
                Some(DnsRecord {
                    id: r.record_id,
                    domain_id: domain_id.to_string(),
                    name: r.rr,
                    ttl: r.ttl,
                    data,
                    proxied: None, // 阿里云不支持代理
                    created_at: Self::timestamp_to_datetime(r.create_timestamp),
                    updated_at: Self::timestamp_to_datetime(r.update_timestamp),
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

        // 从 RecordData 提取 value 和 priority
        let (value, priority) = Self::record_data_to_api(&req.data);
        let record_type = record_type_to_string(&req.data.record_type());

        // 阿里云的 domain_id 就是域名名称，可以直接使用
        let api_req = AddDomainRecordRequest {
            domain_name: req.domain_id.clone(),
            rr: req.name.clone(),
            record_type: record_type.to_string(),
            value,
            ttl: req.ttl,
            priority,
        };

        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            domain: Some(req.domain_id.clone()),
            ..Default::default()
        };

        let response: AddDomainRecordResponse =
            self.request("AddDomainRecord", &api_req, ctx).await?;

        let now = chrono::Utc::now();
        Ok(DnsRecord {
            id: response.record_id,
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

        // 从 RecordData 提取 value 和 priority
        let (value, priority) = Self::record_data_to_api(&req.data);
        let record_type = record_type_to_string(&req.data.record_type());

        let api_req = UpdateDomainRecordRequest {
            record_id: record_id.to_string(),
            rr: req.name.clone(),
            record_type: record_type.to_string(),
            value,
            ttl: req.ttl,
            priority,
        };

        let ctx = ErrorContext {
            record_name: Some(req.name.clone()),
            record_id: Some(record_id.to_string()),
            domain: Some(req.domain_id.clone()),
        };

        let _response: UpdateDomainRecordResponse =
            self.request("UpdateDomainRecord", &api_req, ctx).await?;

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
        #[derive(Serialize)]
        struct DeleteDomainRecordRequest {
            #[serde(rename = "RecordId")]
            record_id: String,
        }

        let api_req = DeleteDomainRecordRequest {
            record_id: record_id.to_string(),
        };

        let ctx = ErrorContext {
            record_id: Some(record_id.to_string()),
            domain: Some(domain_id.to_string()),
            ..Default::default()
        };

        let _response: DeleteDomainRecordResponse =
            self.request("DeleteDomainRecord", &api_req, ctx).await?;

        Ok(())
    }
}
