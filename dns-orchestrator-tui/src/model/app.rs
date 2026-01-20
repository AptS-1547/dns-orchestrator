//! 应用主状态结构

use super::{
    AccountsState, DnsRecordsState, DomainsState, FocusPanel, ModalState, NavigationState, Page,
    SettingsState, ToolboxState,
};

/// 应用主状态
pub struct App {
    /// 是否应该退出
    pub should_quit: bool,

    /// 当前焦点面板
    pub focus: FocusPanel,

    /// 导航状态
    pub navigation: NavigationState,

    /// 当前页面
    pub current_page: Page,

    /// 状态栏消息
    pub status_message: Option<String>,

    // === 各页面状态 ===
    /// 账号页面状态
    pub accounts: AccountsState,
    /// 域名页面状态
    pub domains: DomainsState,
    /// DNS 记录页面状态
    pub dns_records: DnsRecordsState,
    /// 工具箱页面状态
    pub toolbox: ToolboxState,
    /// 设置页面状态
    pub settings: SettingsState,

    /// 弹窗状态
    pub modal: ModalState,
}

impl App {
    /// 创建新的应用实例
    pub fn new() -> Self {
        let mut app = Self {
            should_quit: false,
            focus: FocusPanel::Navigation,
            navigation: NavigationState::new(),
            current_page: Page::Home,
            status_message: None,
            accounts: AccountsState::new(),
            domains: DomainsState::new(),
            dns_records: DnsRecordsState::new(),
            toolbox: ToolboxState::new(),
            settings: SettingsState::new(),
            modal: ModalState::new(),
        };

        // 加载模拟数据（开发阶段）
        app.accounts.load_mock_data();
        app.domains.load_mock_data();

        app
    }

    /// 设置状态消息
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// 清除状态消息
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}