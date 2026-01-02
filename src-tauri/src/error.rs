use serde::Serialize;
use thiserror::Error;

// ============ Re-export 库错误类型 ============

pub use dns_orchestrator_provider::{
    // 凭证验证错误
    CredentialValidationError,
    // Provider 错误
    ProviderError,
};

// Re-export core error
pub use dns_orchestrator_core::error::CoreError;

// ============ 应用层错误类型 ============

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

    /// 凭证验证错误（结构化，支持字段级别错误）
    #[error("{0}")]
    CredentialValidation(CredentialValidationError),

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

    /// 没有选中任何账号（导出时）
    #[error("No accounts selected")]
    NoAccountsSelected,

    /// 不支持的文件版本（导入时）
    #[error("Unsupported file version")]
    UnsupportedFileVersion,

    /// Provider 错误（从库转换）
    #[error("{0}")]
    Provider(#[from] ProviderError),
}

/// 从 `CoreError` 转换为 `DnsError`
impl From<CoreError> for DnsError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::ProviderNotFound(s) => Self::ProviderNotFound(s),
            CoreError::AccountNotFound(s) => Self::AccountNotFound(s),
            CoreError::DomainNotFound(s) => Self::DomainNotFound(s),
            CoreError::RecordNotFound(s) => Self::RecordNotFound(s),
            CoreError::CredentialError(s) | CoreError::StorageError(s) => Self::CredentialError(s),
            CoreError::CredentialValidation(e) => Self::CredentialValidation(e),
            CoreError::ApiError { provider, message } => Self::ApiError { provider, message },
            CoreError::InvalidCredentials(_) => Self::InvalidCredentials,
            CoreError::SerializationError(s) => Self::SerializationError(s),
            CoreError::ValidationError(s) => Self::ValidationError(s),
            CoreError::ImportExportError(s) => Self::ImportExportError(s),
            CoreError::NoAccountsSelected => Self::NoAccountsSelected,
            CoreError::UnsupportedFileVersion => Self::UnsupportedFileVersion,
            CoreError::NetworkError(s) => Self::ApiError {
                provider: "network".to_string(),
                message: s,
            },
            // v1.7.0 迁移相关错误
            CoreError::MigrationRequired => {
                Self::CredentialError("Credential migration required".to_string())
            }
            CoreError::MigrationFailed(s) => {
                Self::CredentialError(format!("Migration failed: {s}"))
            }
            CoreError::Provider(e) => Self::Provider(e),
        }
    }
}
