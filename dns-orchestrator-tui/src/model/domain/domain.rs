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
    /// 是否已收藏
    pub is_favorite: bool,
}
