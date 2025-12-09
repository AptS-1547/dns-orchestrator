use serde::Serialize;
use thiserror::Error;

/// Provider 统一错误类型
/// 用于将各 DNS Provider 的原始错误映射到统一的错误类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    /// 网络请求失败
    NetworkError { provider: String, detail: String },

    /// 凭证无效
    InvalidCredentials { provider: String },

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

    /// 配额超限
    QuotaExceeded {
        provider: String,
        raw_message: Option<String>,
    },

    /// 域名不存在
    DomainNotFound { provider: String, domain: String },

    /// 响应解析失败
    ParseError { provider: String, detail: String },

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
            Self::InvalidCredentials { provider } => {
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
            Self::QuotaExceeded { provider, .. } => {
                write!(f, "[{provider}] Quota exceeded")
            }
            Self::DomainNotFound { provider, domain } => {
                write!(f, "[{provider}] Domain '{domain}' not found")
            }
            Self::ParseError { provider, detail } => {
                write!(f, "[{provider}] Parse error: {detail}")
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

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum DnsError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Credential error: {0}")]
    CredentialError(String),

    #[error("API error: {provider} - {message}")]
    ApiError { provider: String, message: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Import/Export error: {0}")]
    ImportExportError(String),

    #[error("{0}")]
    Provider(#[from] ProviderError),
}

pub type Result<T> = std::result::Result<T, DnsError>;
