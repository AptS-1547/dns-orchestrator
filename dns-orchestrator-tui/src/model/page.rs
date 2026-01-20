//! 页面状态定义

/// 页面枚举
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Page {
    /// 首页
    #[default]
    Home,
    /// 域名列表
    Domains,
    /// DNS 记录页面
    DnsRecords {
        account_id: String,
        domain_id: String,
    },
    /// 账号管理
    Accounts,
    /// 工具箱
    Toolbox,
    /// 设置
    Settings,
}

impl Page {
    /// 获取页面标题
    pub fn title(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Domains => "Domains",
            Page::DnsRecords { .. } => "DNS Records",
            Page::Accounts => "Accounts",
            Page::Toolbox => "Toolbox",
            Page::Settings => "Settings",
        }
    }

    /// 是否是详情页面（需要返回按钮）
    pub fn is_detail_page(&self) -> bool {
        matches!(self, Page::DnsRecords { .. })
    }
}