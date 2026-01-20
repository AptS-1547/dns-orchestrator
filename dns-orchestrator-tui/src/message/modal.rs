//! 弹窗消息类型

/// 弹窗相关消息
#[derive(Debug, Clone)]
pub enum ModalMessage {
    /// 关闭弹窗
    Close,

    /// 下一个输入字段
    NextField,

    /// 上一个输入字段
    PrevField,

    /// 切换服务商（左）
    PrevProvider,

    /// 切换服务商（右）
    NextProvider,

    /// 确认/提交
    Confirm,

    /// 在确认删除弹窗中切换焦点
    ToggleDeleteFocus,

    /// 输入字符
    Input(char),

    /// 删除字符（Backspace）
    Backspace,

    /// 删除光标后的字符（Delete）
    Delete,

    /// 切换密码可见性
    ToggleSecrets,
}
