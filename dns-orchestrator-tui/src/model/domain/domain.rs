//! 域名数据模型

use super::ProviderType;

/// 域名状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainStatus {
    Active,
    Paused,
    Pending,
    Error,
    Unknown,
}

impl DomainStatus {
    /// 获取状态显示文本
    pub fn display_name(&self) -> &'static str {
        match self {
            DomainStatus::Active => "正常",
            DomainStatus::Paused => "已暂停",
            DomainStatus::Pending => "待生效",
            DomainStatus::Error => "异常",
            DomainStatus::Unknown => "未知",
        }
    }
}

/// 域名（来自服务商）
#[derive(Debug, Clone)]
pub struct Domain {
    pub id: String,
    pub name: String,
    /// 所属账号 ID
    pub account_id: String,
    pub provider: ProviderType,
    pub status: DomainStatus,
    /// 该域名下的记录数量
    pub record_count: Option<u32>,
}