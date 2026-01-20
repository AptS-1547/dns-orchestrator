//! 导航状态定义
/// 导航项 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItemId {
    Home,
    Domains,
    Accounts,
    Toolbox,
    Settings,
}

/// 导航项
#[derive(Debug, Clone)]
pub struct NavItem {
    pub id: NavItemId,
    pub label: &'static str,
    pub icon: &'static str,
}

/// 导航状态
pub struct NavigationState {
    /// 导航项列表
    pub items: Vec<NavItem>,
    /// 当前选中的索引
    pub selected: usize,
}

impl NavigationState {
    /// 创建默认导航状态
    pub fn new() -> Self {
        Self {
            items: vec![
                NavItem {
                    id: NavItemId::Home,
                    label: "Home",
                    icon: "⌂",
                },
                NavItem {
                    id: NavItemId::Domains,
                    label: "Domains",
                    icon: "●",
                },
                NavItem {
                    id: NavItemId::Accounts,
                    label: "Accounts",
                    icon: "@",
                },
                NavItem {
                    id: NavItemId::Toolbox,
                    label: "Toolbox",
                    icon: "+",
                },
                NavItem {
                    id: NavItemId::Settings,
                    label: "Settings",
                    icon: "≡",
                },
            ],
            selected: 0,
        }
    }

    /// 选择上一项
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// 获取当前选中的导航项
    pub fn current_item(&self) -> Option<&NavItem> {
        self.items.get(self.selected)
    }

    /// 获取当前选中的导航项 ID
    pub fn current_id(&self) -> Option<NavItemId> {
        self.current_item().map(|item| item.id)
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}