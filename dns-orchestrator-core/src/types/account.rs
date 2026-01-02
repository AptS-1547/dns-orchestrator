//! 账户相关类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use dns_orchestrator_provider::{ProviderCredentials, ProviderType};

/// 账户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// 活跃状态
    Active,
    /// 错误状态（凭证失效等）
    Error,
}

/// 账户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 账户 ID (UUID)
    pub id: String,
    /// 账户名称
    pub name: String,
    /// DNS 服务商类型
    pub provider: ProviderType,
    /// 创建时间
    #[serde(rename = "createdAt")]
    #[serde(with = "crate::utils::datetime")]
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[serde(rename = "updatedAt")]
    #[serde(with = "crate::utils::datetime")]
    pub updated_at: DateTime<Utc>,
    /// 账户状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,
    /// 错误信息（状态为 Error 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 创建账户请求（v1.7.0 类型安全重构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// 账户名称
    pub name: String,
    /// DNS 服务商类型
    pub provider: ProviderType,
    /// 凭证（结构化类型）
    pub credentials: ProviderCredentials,
}

/// 更新账户请求（v1.7.0 类型安全重构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    /// 账户 ID
    pub id: String,
    /// 新的账户名称（可选）
    pub name: Option<String>,
    /// 新的凭证（可选，提供时会覆盖原有凭证）
    pub credentials: Option<ProviderCredentials>,
}
