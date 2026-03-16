//! 页面状态定义

/// 进入 `DnsRecords` 页面的来源
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsRecordsSource {
    Domains,
    Favorites,
}

/// 页面枚举
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Page {
    /// 首页
    #[default]
    Home,
    /// 域名列表
    Domains,
    /// 收藏域名
    Favorites,
    /// DNS 记录页面
    DnsRecords {
        account_id: String,
        domain_id: String,
        source: DnsRecordsSource,
    },
    /// 账号管理
    Accounts,
    /// 工具箱
    Toolbox,
    /// 设置
    Settings,
}

impl Page {
    /// 是否是详情页面（需要返回按钮）
    pub fn is_detail_page(&self) -> bool {
        matches!(self, Page::DnsRecords { .. })
    }
}
