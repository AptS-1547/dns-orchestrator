//! DNSPod 错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::DnspodProvider;

/// DNSPod 错误码映射
/// 参考: <https://cloud.tencent.com/document/api/1427/56192>
impl ProviderErrorMapper for DnspodProvider {
    fn provider_name(&self) -> &'static str {
        "dnspod"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // ============ 认证错误 ============
            Some(
                "AuthFailure"
                | "AuthFailure.InvalidAuthorization"
                | "AuthFailure.InvalidSecretId"
                | "AuthFailure.MFAFailure"
                | "AuthFailure.SecretIdNotFound"
                | "AuthFailure.SignatureExpire"
                | "AuthFailure.SignatureFailure"
                | "AuthFailure.TokenFailure"
                | "AuthFailure.UnauthorizedOperation"
                | "InvalidParameter.InvalidSecretId"
                | "InvalidParameter.InvalidSignature"
                | "InvalidParameter.PermissionDenied"
                | "InvalidParameter.LoginTokenIdError"
                | "InvalidParameter.LoginTokenNotExists"
                | "InvalidParameter.LoginTokenValidateFailed",
            ) => ProviderError::InvalidCredentials {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message.clone()),
            },

            // ============ 配额/频率限制 ============
            Some(
                "LimitExceeded"
                | "LimitExceeded.AAAACountLimit"
                | "LimitExceeded.AtNsRecordLimit"
                | "LimitExceeded.CustomLineLimited"
                | "LimitExceeded.DomainAliasCountExceeded"
                | "LimitExceeded.DomainAliasNumberLimit"
                | "LimitExceeded.FailedLoginLimitExceeded"
                | "LimitExceeded.GroupNumberLimit"
                | "LimitExceeded.HiddenUrlExceeded"
                | "LimitExceeded.NsCountLimit"
                | "LimitExceeded.OffsetExceeded"
                | "LimitExceeded.SrvCountLimit"
                | "LimitExceeded.SubdomainLevelLimit"
                | "LimitExceeded.SubdomainRollLimit"
                | "LimitExceeded.SubdomainWcardLimit"
                | "LimitExceeded.UrlCountLimit"
                | "RequestLimitExceeded"
                | "RequestLimitExceeded.GlobalRegionUinLimitExceeded"
                | "RequestLimitExceeded.IPLimitExceeded"
                | "RequestLimitExceeded.UinLimitExceeded"
                | "RequestLimitExceeded.BatchTaskLimit"
                | "RequestLimitExceeded.CreateDomainLimit"
                | "RequestLimitExceeded.RequestLimitExceeded"
                | "FailedOperation.FrequencyLimit"
                | "InvalidParameter.OperationIsTooFrequent",
            ) => ProviderError::QuotaExceeded {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },

            // ============ 记录已存在 ============
            Some("InvalidParameter.DomainRecordExist") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // ============ 域名不存在 ============
            Some("ResourceNotFound.NoDataOfDomain" | "InvalidParameterValue.DomainNotExists") => {
                ProviderError::DomainNotFound {
                    provider: self.provider_name().to_string(),
                    domain: context.domain.unwrap_or_default(),
                    raw_message: Some(raw.message),
                }
            }

            // ============ 域名被锁定/禁用 ============
            Some(
                "FailedOperation.DomainIsLocked"
                | "FailedOperation.DomainIsSpam"
                | "FailedOperation.AccountIsLocked"
                | "InvalidParameter.UserAlreadyLocked"
                | "InvalidParameter.DomainIsNotlocked"
                | "InvalidParameter.DomainNotAllowedLock",
            ) => ProviderError::DomainLocked {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // ============ 权限/操作被拒绝 ============
            Some(
                "OperationDenied"
                | "OperationDenied.AccessDenied"
                | "OperationDenied.DomainOwnerAllowedOnly"
                | "OperationDenied.NoPermissionToOperateDomain"
                | "OperationDenied.NotAdmin"
                | "OperationDenied.NotAgent"
                | "OperationDenied.NotGrantedByOwner"
                | "OperationDenied.NotManagedUser"
                | "OperationDenied.NotOrderOwner"
                | "OperationDenied.NotResourceOwner"
                | "OperationDenied.AgentDenied"
                | "OperationDenied.AgentSubordinateDenied"
                | "UnauthorizedOperation"
                | "FailedOperation.NotDomainOwner"
                | "FailedOperation.NotResourceOwner"
                | "FailedOperation.NotBatchTaskOwner"
                | "InvalidParameter.NoAuthorityToSrcDomain"
                | "InvalidParameter.NoAuthorityToTheGroup",
            ) => ProviderError::PermissionDenied {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },

            // ============ 参数无效 - 线路 ============
            Some("InvalidParameter.RecordLineInvalid" | "InvalidParameter.LineNotExist") => {
                ProviderError::InvalidParameter {
                    provider: self.provider_name().to_string(),
                    param: "line".to_string(),
                    detail: raw.message,
                }
            }

            // ============ 参数无效 - 记录类型 ============
            Some("InvalidParameter.RecordTypeInvalid") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "type".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 记录值 ============
            Some(
                "InvalidParameter.RecordValueInvalid" | "InvalidParameter.RecordValueLengthInvalid",
            ) => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "value".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 子域名 ============
            Some("InvalidParameter.SubdomainInvalid") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "subdomain".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - TTL ============
            Some("LimitExceeded.RecordTtlLimit") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "ttl".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - MX优先级 ============
            Some("InvalidParameter.MxInvalid") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "mx".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 域名 ============
            Some(
                "InvalidParameter.DomainIdInvalid"
                | "InvalidParameter.DomainInvalid"
                | "InvalidParameter.DomainTooLong"
                | "InvalidParameter.DomainTypeInvalid",
            ) => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "domain".to_string(),
                detail: raw.message,
            },

            // ============ 参数无效 - 记录ID ============
            Some("InvalidParameter.RecordIdInvalid") => ProviderError::InvalidParameter {
                provider: self.provider_name().to_string(),
                param: "record_id".to_string(),
                detail: raw.message,
            },

            // ============ 其他错误 fallback ============
            _ => self.unknown_error(raw),
        }
    }
}
