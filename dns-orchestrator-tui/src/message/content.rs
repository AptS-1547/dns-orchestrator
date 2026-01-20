//! 内容面板消息
//!
//! 处理内容面板中的操作，如列表选择、增删改查等

/// 内容面板消息
#[derive(Debug, Clone)]
pub enum ContentMessage {
    // ========== 列表导航 ==========
    /// 选择上一项
    SelectPrevious,
    /// 选择下一项
    SelectNext,
    /// 跳转到第一项
    SelectFirst,
    /// 跳转到最后一项
    SelectLast,
    /// 确认选择（进入详情或执行操作）
    Confirm,

    // ========== CRUD 操作 ==========
    /// 添加新项目
    Add,
    /// 编辑当前选中项
    Edit,
    /// 删除当前选中项
    Delete,

    // ========== 导入导出 ==========
    /// 导入
    Import,
    /// 导出
    Export,

    // ========== 工具箱专用 ==========
    /// 切换工具标签页
    SwitchTab,
    /// 执行工具
    Execute,

    // ========== 设置页面专用 ==========
    /// 切换到上一个值（用于设置项）
    TogglePrev,
    /// 切换到下一个值（用于设置项）
    ToggleNext,
}
