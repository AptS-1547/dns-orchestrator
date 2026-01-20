//! 焦点状态定义

/// 焦点面板枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPanel {
    /// 左侧导航面板
    #[default]
    Navigation,
    /// 右侧内容面板
    Content,
}

impl FocusPanel {
    /// 切换到另一个面板
    pub fn toggle(&self) -> Self {
        match self {
            FocusPanel::Navigation => FocusPanel::Content,
            FocusPanel::Content => FocusPanel::Navigation,
        }
    }

    /// 是否是导航面板
    pub fn is_navigation(&self) -> bool {
        matches!(self, FocusPanel::Navigation)
    }

    /// 是否是内容面板
    pub fn is_content(&self) -> bool {
        matches!(self, FocusPanel::Content)
    }
}