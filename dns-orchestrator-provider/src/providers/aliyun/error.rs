//! 阿里云错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::AliyunProvider;

/// 阿里云错误码映射
/// 参考: <https://api.aliyun.com/document/Alidns/2015-01-09/errorCode>
impl ProviderErrorMapper for AliyunProvider {
    fn provider_name(&self) -> &'static str {
        "aliyun"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // ============ 认证错误 ============
            Some("InvalidAccessKeyId.NotFound" | "SignatureDoesNotMatch") => {
                ProviderError::InvalidCredentials {
                    provider: self.provider_name().to_string(),
                    raw_message: Some(raw.message),
                }
            }

            // ============ 记录已存在 ============
            Some("DomainRecordDuplicate" | "DomainRecordConflict") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // ============ 记录不存在 ============
            Some(
                "DomainRecordNotBelongToUser"
                | "InvalidRecordId.NotFound"
                | "InvalidRR.NoExist"
                | "PdnsRecord.NotExists",
            ) => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // ============ 域名不存在 ============
            Some("InvalidDomainName.NoExist" | "DomainNotFound" | "PdnsZone.NotExists") => {
                ProviderError::DomainNotFound {
                    provider: self.provider_name().to_string(),
                    domain: context.domain.unwrap_or_default(),
                    raw_message: Some(raw.message),
                }
            }

            // ============ 配额/频率限制 ============
            Some(
                "QuotaExceeded.ARecord"
                | "QuotaExceeded.Record"
                | "QuotaExceeded.FreeDnsRecord"
                | "QuotaExceeded.SubDomain"
                | "QuotaExceeded.TTL"
                | "QuotaExceeded.AliasRecord"
                | "QuotaExceeded.ALIASRecord"
                | "QuotaExceeded.HTTPSRecord"
                | "QuotaExceeded.SVCBRecord"
                | "LineDnsSlb.QuotaExceeded"
                | "Throttling"
                | "Throttling.User",
            ) => ProviderError::QuotaExceeded {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },

            // ============ 域名被锁定/禁用 ============
            Some(
                "DomainRecordLocked"
                | "DomainExpiredDNSForbidden"
                | "Forbidden.DomainExpired"
                | "RecordForbidden.BlackHole"
                | "RecordFobidden.BlackHole",
            ) => ProviderError::DomainLocked {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // ============ 权限/操作被拒绝 ============
            Some(
                "Forbidden"
                | "Forbidden.RiskControl"
                | "OperationDomain.NoPermission"
                | "IllegalUser"
                | "IncorrectDomainUser",
            ) => ProviderError::PermissionDenied {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },

            // ============ 参数无效 - 记录类型 ============
            Some("InvalidRR.TypeEmpty" | "SubDomainInvalid.Type" | "PdnsRecord.InvalidType") => {
                ProviderError::InvalidParameter {
                    provider: self.provider_name().to_string(),
                    param: "type".to_string(),
                    detail: raw.message,
                }
            }

            // ============ 参数无效 - 记录值 ============
            Some(
                "InvalidRR.AValue"
                | "InvalidRR.AAAAValue"
                | "InvalidRR.MXValue"
                | "InvalidRR.NSValue"
                | "PdnsRecord.InvalidRecordValue",
            ) => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "value".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 主机记录 ============
            Some(
                "InvalidRR.RrEmpty" | "InvalidRR.Format" | "Record.Invalid.Rr" | "InvalidRR.Length",
            ) => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "rr".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - TTL ============
            Some("SubDomainInvalid.TTL" | "PdnsRecord.InvalidTtl") => {
                ProviderError::InvalidParameter {
                    provider: self.provider_name().to_string(),
                    param: "ttl".to_string(),
                    detail: raw.message,
                }
            }

            // ============ 参数无效 - 线路 ============
            Some("SubDomainInvalid.Line" | "UnsupportedLine") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "line".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - MX优先级 ============
            Some("SubDomainInvalid.Priority") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "priority".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 域名格式 ============
            Some(
                "InvalidDomainName.Format"
                | "InvalidDomainName.Suffix"
                | "InvalidDomainName.Length"
                | "DomainEmpty"
                | "PdnsZone.InvalidZoneName",
            ) => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "domain".to_string(),
                detail: raw.message,
            },

            // ============ 其他错误 fallback ============
            _ => self.unknown_error(raw),
        }
    }
}
