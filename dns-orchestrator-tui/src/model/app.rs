//! 应用主状态结构

use std::sync::Arc;

use tokio::runtime::Handle;

use crate::backend::CoreService;

use super::{
    AccountsState, DnsRecordsState, DomainsState, FavoritesState, FocusPanel, ModalState,
    NavigationState, Page, SettingsState, ToolboxState,
};

/// 应用主状态
///
/// 持有所有 UI 状态和后端服务引用。
/// `core_service` 提供业务逻辑调用，`rt_handle` 用于在同步事件循环中
/// 通过 `Handle::block_on` 桥接 async 后端方法。
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
    /// 收藏页面状态
    pub favorites: FavoritesState,
    /// DNS 记录页面状态
    pub dns_records: DnsRecordsState,
    /// 工具箱页面状态
    pub toolbox: ToolboxState,
    /// 设置页面状态
    pub settings: SettingsState,

    /// 弹窗状态
    pub modal: ModalState,

    // === 后端服务 ===
    /// 核心业务服务（账号、域名、DNS 等）
    pub core_service: Arc<CoreService>,
    /// Tokio runtime handle，用于在同步上下文中调用 async 后端
    pub rt_handle: Handle,
}

impl App {
    /// 创建新的应用实例
    pub fn new(core_service: Arc<CoreService>, rt_handle: Handle) -> Self {
        let mut app = Self {
            should_quit: false,
            focus: FocusPanel::Navigation,
            navigation: NavigationState::new(),
            current_page: Page::Home,
            status_message: None,
            accounts: AccountsState::new(),
            domains: DomainsState::new(),
            favorites: FavoritesState::new(),
            dns_records: DnsRecordsState::new(),
            toolbox: ToolboxState::new(),
            settings: SettingsState::new(),
            modal: ModalState::new(),
            core_service,
            rt_handle,
        };

        // 开发阶段：加载域名模拟数据
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
