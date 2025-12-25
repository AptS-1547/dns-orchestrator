use serde::Serialize;

/// Provider 统一错误类型
/// 用于将各 DNS Provider 的原始错误映射到统一的错误类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    /// 网络请求失败
    NetworkError { provider: String, detail: String },

    /// 凭证无效
    InvalidCredentials {
        provider: String,
        raw_message: Option<String>,
    },

    /// 记录已存在
    RecordExists {
        provider: String,
        record_name: String,
        raw_message: Option<String>,
    },

    /// 记录不存在
    RecordNotFound {
        provider: String,
        record_id: String,
        raw_message: Option<String>,
    },

    /// 参数无效（TTL、值等）
    InvalidParameter {
        provider: String,
        param: String,
        detail: String,
    },

    /// 不支持的记录类型
    UnsupportedRecordType {
        provider: String,
        record_type: String,
    },

    /// 配额超限
    QuotaExceeded {
        provider: String,
        raw_message: Option<String>,
    },

    /// 域名不存在
    DomainNotFound {
        provider: String,
        domain: String,
        raw_message: Option<String>,
    },

    /// 域名被锁定/禁用
    DomainLocked {
        provider: String,
        domain: String,
        raw_message: Option<String>,
    },

    /// 权限/操作被拒绝
    PermissionDenied {
        provider: String,
        raw_message: Option<String>,
    },

    /// 响应解析失败
    ParseError { provider: String, detail: String },

    /// 序列化/反序列化失败
    SerializationError { provider: String, detail: String },

    /// 未知错误（fallback）
    Unknown {
        provider: String,
        raw_code: Option<String>,
        raw_message: String,
    },
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError { provider, detail } => {
                write!(f, "[{provider}] Network error: {detail}")
            }
            Self::InvalidCredentials {
                provider,
                raw_message,
            } => {
                if let Some(msg) = raw_message {
                    write!(f, "[{provider}] Invalid credentials: {msg}")
                } else {
                    write!(f, "[{provider}] Invalid credentials")
                }
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

/// 库的统一 Result 类型
pub type Result<T> = std::result::Result<T, ProviderError>;
