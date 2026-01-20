//! 导航相关消息

/// 导航消息
#[derive(Debug, Clone)]
pub enum NavigationMessage {
    /// 选择上一项
    SelectPrevious,
    /// 选择下一项
    SelectNext,
    /// 确认选择（进入选中的页面）
    Confirm,
    /// 跳转到第一项
    SelectFirst,
    /// 跳转到最后一项
    SelectLast,
}