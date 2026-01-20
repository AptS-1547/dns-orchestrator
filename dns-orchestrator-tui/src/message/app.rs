//! 应用主消息枚举

use super::{ContentMessage, ModalMessage, NavigationMessage};

/// 应用主消息
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// 退出应用
    Quit,

    /// 切换焦点面板（左右切换）
    ToggleFocus,

    /// 导航相关消息
    Navigation(NavigationMessage),

    /// 内容面板相关消息
    Content(ContentMessage),

    /// 弹窗相关消息
    Modal(ModalMessage),

    /// 返回上一页
    GoBack,

    /// 刷新当前页面
    Refresh,

    /// 显示帮助
    ShowHelp,

    /// 清除状态消息
    ClearStatus,

    /// 无操作（用于忽略未处理的事件）
    Noop,
}