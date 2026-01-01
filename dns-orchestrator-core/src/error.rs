//! 统一错误类型定义

use serde::Serialize;
use thiserror::Error;

// Re-export 库错误类型
pub use dns_orchestrator_provider::{CredentialValidationError, ProviderError};

/// 核心层错误类型
#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "details")]
pub enum CoreError {
    /// Provider 未找到
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    /// 账户未找到
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// 域名未找到
    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    /// 记录未找到
    #[error("Record not found: {0}")]
    RecordNotFound(String),

    /// 凭证存储错误
    #[error("Credential error: {0}")]
    CredentialError(String),

    /// 凭证验证错误（结构化，支持字段级别错误）
    #[error("{0}")]
    CredentialValidation(CredentialValidationError),

    /// API 错误
    #[error("API error: {provider} - {message}")]
    ApiError { provider: String, message: String },

    /// 凭证无效
    #[error("Invalid credentials for: {0}")]
    InvalidCredentials(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// 验证错误
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 导入导出错误
    #[error("Import/Export error: {0}")]
    ImportExportError(String),

    /// 没有选中任何账号（导出时）
    #[error("No accounts selected")]
    NoAccountsSelected,

    /// 不支持的文件版本（导入时）
    #[error("Unsupported file version")]
    UnsupportedFileVersion,

    /// 存储层错误
    #[error("Storage error: {0}")]
    StorageError(String),

    /// 网络错误
    #[error("Network error: {0}")]
    NetworkError(String),

    /// 需要迁移数据格式（v1.7.0 凭证格式升级）
    #[error("Credential data migration required")]
    MigrationRequired,

    /// 迁移失败
    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    /// Provider 错误（从库转换）
    #[error("{0}")]
    Provider(#[from] ProviderError),
}

/// 核心层 Result 类型别名
pub type CoreResult<T> = std::result::Result<T, CoreError>;
