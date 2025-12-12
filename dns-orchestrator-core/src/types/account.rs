//! 账户相关类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use dns_orchestrator_provider::ProviderType;

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
    /// 创建时间 (RFC3339)
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// 更新时间 (RFC3339)
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// 账户状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AccountStatus>,
    /// 错误信息（状态为 Error 时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 创建账户请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    /// 账户名称
    pub name: String,
    /// DNS 服务商类型
    pub provider: ProviderType,
    /// 凭证键值对
    pub credentials: HashMap<String, String>,
}
