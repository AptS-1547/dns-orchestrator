//! 账号数据模型
//!
//! 对应 dns-orchestrator-core/src/types/account.rs
//! 和 dns-orchestrator-provider/src/types.rs 中的 ProviderType

/// DNS 服务商类型
///
/// 对应 dns-orchestrator-provider 中的 ProviderType 枚举
/// TUI 版本不使用 feature flags，直接包含所有服务商
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    Cloudflare,
    Aliyun,
    Dnspod,
    Huaweicloud,
}

impl ProviderType {
    /// 获取服务商显示名称（用于 UI 渲染）
    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderType::Cloudflare => "Cloudflare",
            ProviderType::Aliyun => "Aliyun",
            ProviderType::Dnspod => "DNSPod",
            ProviderType::Huaweicloud => "Huawei Cloud",
        }
    }

    /// 获取服务商简称（用于紧凑显示）
    pub fn short_name(&self) -> &'static str {
        match self {
            ProviderType::Cloudflare => "CF",
            ProviderType::Aliyun => "Ali",
            ProviderType::Dnspod => "DP",
            ProviderType::Huaweicloud => "HW",
        }
    }
}

/// 账号状态
///
/// 对应 dns-orchestrator-core 中的 AccountStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccountStatus {
    #[default]
    Active,
    Error,
}

/// 账号
///
/// 对应 dns-orchestrator-core 中的 Account
/// TUI 版本使用 String 存储时间戳（简化处理，避免引入 chrono）
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub provider: ProviderType,
    /// 账号状态（可选，None 表示未知）
    pub status: Option<AccountStatus>,
    /// 错误信息（状态为 Error 时）
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}