//! Cloudflare 错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::CloudflareProvider;

/// Cloudflare 错误码映射
/// 参考: <https://api.cloudflare.com/#getting-started-responses>
impl ProviderErrorMapper for CloudflareProvider {
    fn provider_name(&self) -> &'static str {
        "cloudflare"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            // 6003: Invalid request headers
            // 6103: Invalid format for X-Auth-Key header
            // 6111: Invalid format for Authorization header
            // 9109: Unauthorized to access requested resource / Max auth failures reached
            // 10000: Authentication error
            Some("6003" | "6103" | "6111" | "9109" | "10000") => {
                ProviderError::InvalidCredentials {
                    provider: self.provider_name().to_string(),
                    raw_message: Some(raw.message),
                }
            }

            // 参数无效
            // 1004: DNS Validation Error
            // 9000: Invalid or missing name
            // 9005: Content for A record is invalid. Must be a valid IPv4 address
            // 9006: Content for AAAA record is invalid. Must be a valid IPv6 address
            // 9009: Content for MX record must be a hostname
            // 9021: Invalid TTL. Must be between 120 and 2147483647 seconds or 1 for automatic
            // 9041: This DNS record cannot be proxied
            Some(code @ ("1004" | "9000" | "9005" | "9006" | "9009" | "9021" | "9041")) => {
                let param = match code {
                    "9000" => "name",
                    "9005" | "9006" | "9009" => "value",
                    "9021" => "ttl",
                    "9041" => "proxied",
                    // "1004" is a general validation error.
                    _ => "general",
                };
                ProviderError::InvalidParameter {
                    provider: self.provider_name().to_string(),
                    param: param.to_string(),
                    detail: raw.message,
                }
            }

            // 记录已存在
            // 81053: An A AAAA or CNAME record already exists with that host
            // 81054: A CNAME record with that host already exists
            // 81055: An A record with that host already exists
            // 81056: NS records with that host already exist
            // 81057: The record already exists
            // 81058: A record with those settings already exists
            Some("81053" | "81054" | "81055" | "81056" | "81057" | "81058") => {
                ProviderError::RecordExists {
                    provider: self.provider_name().to_string(),
                    record_name: context.record_name.unwrap_or_default(),
                    raw_message: Some(raw.message),
                }
            }

            // 记录不存在
            // 81044: Record does not exist
            Some("81044") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // 配额超限
            // 81045: The record quota has been exceeded
            Some("81045") => ProviderError::QuotaExceeded {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },

            // Zone/域名不存在
            // 7000: No route for that URI
            // 7003: Could not route to /path. perhaps your object identifier is invalid?
            Some("7000" | "7003") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },

            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}
