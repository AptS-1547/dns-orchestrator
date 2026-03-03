use serde::{Deserialize, Serialize};

/// Unified error type for all DNS provider operations.
///
/// Each variant includes a `provider` field identifying which provider produced the error,
/// plus variant-specific context. All variants are serializable for structured error reporting.
///
/// # Retryable Errors
///
/// The following variants represent transient failures that may succeed on retry:
/// - [`NetworkError`](Self::NetworkError) — network connectivity issues
/// - [`Timeout`](Self::Timeout) — request timed out
/// - [`RateLimited`](Self::RateLimited) — API rate limit exceeded
///
/// The built-in HTTP client automatically retries these with exponential backoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    /// A network-level error occurred (DNS resolution failure, connection refused, etc.).
    ///
    /// This is a transient error and is automatically retried.
    NetworkError {
        /// Provider that produced the error.
        provider: String,
        /// Error details.
        detail: String,
    },

    /// The provided credentials are invalid or expired.
    InvalidCredentials {
        /// Provider that produced the error.
        provider: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// A DNS record with the same name/type already exists.
    RecordExists {
        /// Provider that produced the error.
        provider: String,
        /// Name of the conflicting record.
        record_name: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// The specified DNS record was not found.
    RecordNotFound {
        /// Provider that produced the error.
        provider: String,
        /// ID of the record that was not found.
        record_id: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// A request parameter is invalid (e.g., bad TTL value, malformed IP address).
    InvalidParameter {
        /// Provider that produced the error.
        provider: String,
        /// Name of the invalid parameter.
        param: String,
        /// Description of what's wrong.
        detail: String,
    },

    /// The requested DNS record type is not supported by this provider.
    UnsupportedRecordType {
        /// Provider that produced the error.
        provider: String,
        /// The unsupported record type string.
        record_type: String,
    },

    /// The account's resource quota has been exceeded.
    ///
    /// Unlike [`RateLimited`](Self::RateLimited), this is not a transient condition.
    QuotaExceeded {
        /// Provider that produced the error.
        provider: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// The API rate limit has been exceeded (HTTP 429 or equivalent).
    ///
    /// This is a transient error. Unlike [`QuotaExceeded`](Self::QuotaExceeded),
    /// the request should succeed after waiting.
    RateLimited {
        /// Provider that produced the error.
        provider: String,
        /// Suggested wait time in seconds before retrying, if provided by the API.
        retry_after: Option<u64>,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// The HTTP request timed out.
    ///
    /// This is a transient error and is automatically retried.
    Timeout {
        /// Provider that produced the error.
        provider: String,
        /// Error details.
        detail: String,
    },

    /// The specified domain/zone was not found.
    DomainNotFound {
        /// Provider that produced the error.
        provider: String,
        /// Domain name that was not found.
        domain: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// The domain is locked or disabled and cannot be modified.
    DomainLocked {
        /// Provider that produced the error.
        provider: String,
        /// Domain name that is locked.
        domain: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// The authenticated user lacks permission for the requested operation.
    PermissionDenied {
        /// Provider that produced the error.
        provider: String,
        /// Original error message from the provider API, if available.
        raw_message: Option<String>,
    },

    /// Failed to parse the provider's API response.
    ParseError {
        /// Provider that produced the error.
        provider: String,
        /// Details about the parse failure.
        detail: String,
    },

    /// Failed to serialize a request body.
    SerializationError {
        /// Provider that produced the error.
        provider: String,
        /// Details about the serialization failure.
        detail: String,
    },

    /// An unrecognized error from the provider API.
    ///
    /// This is a catch-all for error codes not yet mapped to a specific variant.
    Unknown {
        /// Provider that produced the error.
        provider: String,
        /// Raw error code from the API, if available.
        raw_code: Option<String>,
        /// Raw error message from the API.
        raw_message: String,
    },
}

impl ProviderError {
    /// Whether it is expected behavior (user input, resource does not exist, etc.) is used for log classification.
    ///
    /// Level `warn` should be used when returning `true` and level `error` when returning `false`.
    /// **Please update this method simultaneously when new variants are added. **
    #[must_use]
    pub fn is_expected(&self) -> bool {
        matches!(
            self,
            Self::InvalidCredentials { .. }
                | Self::RecordExists { .. }
                | Self::RecordNotFound { .. }
                | Self::InvalidParameter { .. }
                | Self::UnsupportedRecordType { .. }
                | Self::QuotaExceeded { .. }
                | Self::DomainNotFound { .. }
                | Self::DomainLocked { .. }
                | Self::PermissionDenied { .. }
        )
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError { provider, detail } => {
                write!(f, "[{provider}] Network error: {detail}")
            }
            Self::InvalidCredentials { provider, .. } => {
                write!(f, "[{provider}] Invalid credentials")
            }
            Self::RecordExists {
                provider,
                record_name,
                ..
            } => {
                write!(f, "[{provider}] Record '{record_name}' already exists")
            }
            Self::RecordNotFound {
                provider,
                record_id,
                ..
            } => {
                write!(f, "[{provider}] Record '{record_id}' not found")
            }
            Self::InvalidParameter {
                provider,
                param,
                detail,
            } => {
                write!(f, "[{provider}] Invalid parameter '{param}': {detail}")
            }
            Self::UnsupportedRecordType {
                provider,
                record_type,
            } => {
                write!(f, "[{provider}] Unsupported record type: {record_type}")
            }
            Self::QuotaExceeded { provider, .. } => {
                write!(f, "[{provider}] Quota exceeded")
            }
            Self::RateLimited {
                provider,
                retry_after,
                ..
            } => {
                if let Some(secs) = retry_after {
                    write!(f, "[{provider}] Rate limited (retry after {secs}s)")
                } else {
                    write!(f, "[{provider}] Rate limited")
                }
            }
            Self::Timeout { provider, detail } => {
                write!(f, "[{provider}] Request timeout: {detail}")
            }
            Self::DomainNotFound {
                provider,
                domain,
                raw_message,
            } => {
                if let Some(msg) = raw_message {
                    write!(f, "[{provider}] Domain '{domain}' not found: {msg}")
                } else {
                    write!(f, "[{provider}] Domain '{domain}' not found")
                }
            }
            Self::DomainLocked {
                provider,
                domain,
                raw_message,
            } => {
                if let Some(msg) = raw_message {
                    write!(f, "[{provider}] Domain '{domain}' is locked: {msg}")
                } else {
                    write!(f, "[{provider}] Domain '{domain}' is locked")
                }
            }
            Self::PermissionDenied {
                provider,
                raw_message,
            } => {
                if let Some(msg) = raw_message {
                    write!(f, "[{provider}] Permission denied: {msg}")
                } else {
                    write!(f, "[{provider}] Permission denied")
                }
            }
            Self::ParseError { provider, detail } => {
                write!(f, "[{provider}] Parse error: {detail}")
            }
            Self::SerializationError { provider, detail } => {
                write!(f, "[{provider}] Serialization error: {detail}")
            }
            Self::Unknown {
                provider,
                raw_message,
                ..
            } => {
                write!(f, "[{provider}] {raw_message}")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

/// Convenience type alias for `Result<T, ProviderError>`.
pub type Result<T> = std::result::Result<T, ProviderError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_network_error() {
        let e = ProviderError::NetworkError {
            provider: "test".to_string(),
            detail: "connection refused".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Network error: connection refused");
    }

    #[test]
    fn display_invalid_credentials_with_message() {
        let e = ProviderError::InvalidCredentials {
            provider: "aliyun".to_string(),
            raw_message: Some("bad key".to_string()),
        };
        // raw_message is intentionally omitted from Display to prevent leakage
        assert_eq!(e.to_string(), "[aliyun] Invalid credentials");
    }

    #[test]
    fn display_invalid_credentials_without_message() {
        let e = ProviderError::InvalidCredentials {
            provider: "aliyun".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[aliyun] Invalid credentials");
    }

    #[test]
    fn display_record_exists() {
        let e = ProviderError::RecordExists {
            provider: "dnspod".to_string(),
            record_name: "www".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[dnspod] Record 'www' already exists");
    }

    #[test]
    fn display_record_not_found() {
        let e = ProviderError::RecordNotFound {
            provider: "cf".to_string(),
            record_id: "123".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[cf] Record '123' not found");
    }

    #[test]
    fn display_invalid_parameter() {
        let e = ProviderError::InvalidParameter {
            provider: "test".to_string(),
            param: "ttl".to_string(),
            detail: "must be > 0".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Invalid parameter 'ttl': must be > 0");
    }

    #[test]
    fn display_unsupported_record_type() {
        let e = ProviderError::UnsupportedRecordType {
            provider: "test".to_string(),
            record_type: "LOC".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Unsupported record type: LOC");
    }

    #[test]
    fn display_quota_exceeded() {
        let e = ProviderError::QuotaExceeded {
            provider: "test".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[test] Quota exceeded");
    }

    #[test]
    fn display_rate_limited_with_retry() {
        let e = ProviderError::RateLimited {
            provider: "cloudflare".to_string(),
            retry_after: Some(30),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[cloudflare] Rate limited (retry after 30s)");
    }

    #[test]
    fn display_rate_limited_without_retry() {
        let e = ProviderError::RateLimited {
            provider: "aliyun".to_string(),
            retry_after: None,
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[aliyun] Rate limited");
    }

    #[test]
    fn display_timeout() {
        let e = ProviderError::Timeout {
            provider: "test".to_string(),
            detail: "30s elapsed".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Request timeout: 30s elapsed");
    }

    #[test]
    fn display_domain_not_found_with_message() {
        let e = ProviderError::DomainNotFound {
            provider: "test".to_string(),
            domain: "example.com".to_string(),
            raw_message: Some("no such zone".to_string()),
        };
        assert_eq!(
            e.to_string(),
            "[test] Domain 'example.com' not found: no such zone"
        );
    }

    #[test]
    fn display_domain_not_found_without_message() {
        let e = ProviderError::DomainNotFound {
            provider: "test".to_string(),
            domain: "example.com".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[test] Domain 'example.com' not found");
    }

    #[test]
    fn display_domain_locked() {
        let e = ProviderError::DomainLocked {
            provider: "test".to_string(),
            domain: "example.com".to_string(),
            raw_message: None,
        };
        assert_eq!(e.to_string(), "[test] Domain 'example.com' is locked");
    }

    #[test]
    fn display_permission_denied() {
        let e = ProviderError::PermissionDenied {
            provider: "test".to_string(),
            raw_message: Some("no access".to_string()),
        };
        assert_eq!(e.to_string(), "[test] Permission denied: no access");
    }

    #[test]
    fn display_parse_error() {
        let e = ProviderError::ParseError {
            provider: "test".to_string(),
            detail: "bad json".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Parse error: bad json");
    }

    #[test]
    fn display_serialization_error() {
        let e = ProviderError::SerializationError {
            provider: "test".to_string(),
            detail: "failed".to_string(),
        };
        assert_eq!(e.to_string(), "[test] Serialization error: failed");
    }

    #[test]
    fn display_unknown() {
        let e = ProviderError::Unknown {
            provider: "test".to_string(),
            raw_code: Some("E001".to_string()),
            raw_message: "something broke".to_string(),
        };
        assert_eq!(e.to_string(), "[test] something broke");
    }

    #[test]
    fn serialize_json_round_trip() {
        let e = ProviderError::RateLimited {
            provider: "cloudflare".to_string(),
            retry_after: Some(60),
            raw_message: Some("too many requests".to_string()),
        };
        let json_res = serde_json::to_string(&e);
        assert!(
            json_res.is_ok(),
            "serde_json::to_string failed: {json_res:?}"
        );
        let Ok(json) = json_res else {
            return;
        };
        assert!(json.contains("\"code\":\"RateLimited\""));
        assert!(json.contains("\"retry_after\":60"));
    }

    #[test]
    fn deserialize_json_round_trip() {
        let original = ProviderError::NetworkError {
            provider: "aliyun".to_string(),
            detail: "connection refused".to_string(),
        };
        let json_res = serde_json::to_string(&original);
        assert!(
            json_res.is_ok(),
            "serde_json::to_string failed: {json_res:?}"
        );
        let Ok(json) = json_res else {
            return;
        };

        let deserialized_res: serde_json::Result<ProviderError> = serde_json::from_str(&json);
        assert!(
            deserialized_res.is_ok(),
            "serde_json::from_str failed: {deserialized_res:?}"
        );
        let Ok(deserialized) = deserialized_res else {
            return;
        };
        assert_eq!(deserialized.to_string(), original.to_string());
    }

    #[test]
    fn deserialize_all_variants() {
        let variants: Vec<ProviderError> = vec![
            ProviderError::NetworkError {
                provider: "t".into(),
                detail: "d".into(),
            },
            ProviderError::InvalidCredentials {
                provider: "t".into(),
                raw_message: None,
            },
            ProviderError::RecordExists {
                provider: "t".into(),
                record_name: "www".into(),
                raw_message: None,
            },
            ProviderError::RecordNotFound {
                provider: "t".into(),
                record_id: "1".into(),
                raw_message: None,
            },
            ProviderError::InvalidParameter {
                provider: "t".into(),
                param: "ttl".into(),
                detail: "bad".into(),
            },
            ProviderError::UnsupportedRecordType {
                provider: "t".into(),
                record_type: "LOC".into(),
            },
            ProviderError::QuotaExceeded {
                provider: "t".into(),
                raw_message: None,
            },
            ProviderError::RateLimited {
                provider: "t".into(),
                retry_after: Some(30),
                raw_message: None,
            },
            ProviderError::Timeout {
                provider: "t".into(),
                detail: "30s".into(),
            },
            ProviderError::DomainNotFound {
                provider: "t".into(),
                domain: "x.com".into(),
                raw_message: None,
            },
            ProviderError::DomainLocked {
                provider: "t".into(),
                domain: "x.com".into(),
                raw_message: None,
            },
            ProviderError::PermissionDenied {
                provider: "t".into(),
                raw_message: None,
            },
            ProviderError::ParseError {
                provider: "t".into(),
                detail: "bad".into(),
            },
            ProviderError::SerializationError {
                provider: "t".into(),
                detail: "fail".into(),
            },
            ProviderError::Unknown {
                provider: "t".into(),
                raw_code: Some("E1".into()),
                raw_message: "oops".into(),
            },
        ];

        for v in &variants {
            let json_res = serde_json::to_string(v);
            assert!(
                json_res.is_ok(),
                "serde_json::to_string failed: {json_res:?}"
            );
            let Ok(json) = json_res else {
                return;
            };

            let back_res: serde_json::Result<ProviderError> = serde_json::from_str(&json);
            assert!(
                back_res.is_ok(),
                "serde_json::from_str failed: {back_res:?}"
            );
            let Ok(back) = back_res else {
                return;
            };
            assert_eq!(back.to_string(), v.to_string());
        }
    }

    #[test]
    fn is_retryable_variants() {
        // Introduce the is_retryable logic of http_client to do equivalence testing
        let retryable = |e: &ProviderError| {
            matches!(
                e,
                ProviderError::NetworkError { .. }
                    | ProviderError::Timeout { .. }
                    | ProviderError::RateLimited { .. }
            )
        };

        assert!(retryable(&ProviderError::NetworkError {
            provider: "t".into(),
            detail: "x".into(),
        }));
        assert!(retryable(&ProviderError::Timeout {
            provider: "t".into(),
            detail: "x".into(),
        }));
        assert!(retryable(&ProviderError::RateLimited {
            provider: "t".into(),
            retry_after: None,
            raw_message: None,
        }));
        assert!(!retryable(&ProviderError::QuotaExceeded {
            provider: "t".into(),
            raw_message: None,
        }));
        assert!(!retryable(&ProviderError::InvalidCredentials {
            provider: "t".into(),
            raw_message: None,
        }));
        assert!(!retryable(&ProviderError::RecordNotFound {
            provider: "t".into(),
            record_id: "x".into(),
            raw_message: None,
        }));
    }
}
