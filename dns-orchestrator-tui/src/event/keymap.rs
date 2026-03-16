//! 快捷键配置
//!
//! 定义可配置的快捷键映射（未来可支持用户自定义）

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 快捷键绑定
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub modifiers: KeyModifiers,
    pub code: KeyCode,
}

impl KeyBinding {
    pub const fn new(modifiers: KeyModifiers, code: KeyCode) -> Self {
        Self { modifiers, code }
    }

    pub const fn key(code: KeyCode) -> Self {
        Self::new(KeyModifiers::NONE, code)
    }

    pub const fn alt(code: KeyCode) -> Self {
        Self::new(KeyModifiers::ALT, code)
    }

    pub const fn ctrl(code: KeyCode) -> Self {
        Self::new(KeyModifiers::CONTROL, code)
    }

    /// 检查按键事件是否匹配此快捷键绑定
    pub fn matches(&self, key: &KeyEvent) -> bool {
        key.modifiers == self.modifiers && key.code == self.code
    }
}

/// 默认快捷键配置
pub struct DefaultKeymap;

impl DefaultKeymap {
    // 全局
    pub const FORCE_QUIT: KeyBinding = KeyBinding::ctrl(KeyCode::Char('c'));
    pub const HELP: KeyBinding = KeyBinding::alt(KeyCode::Char('h'));
    pub const REFRESH: KeyBinding = KeyBinding::alt(KeyCode::Char('r'));
    pub const BACK: KeyBinding = KeyBinding::key(KeyCode::Esc);

    // 操作
    pub const ACTION_ADD: KeyBinding = KeyBinding::alt(KeyCode::Char('a'));
    pub const ACTION_EDIT: KeyBinding = KeyBinding::alt(KeyCode::Char('e'));
    pub const ACTION_DELETE: KeyBinding = KeyBinding::alt(KeyCode::Char('d'));
    pub const ACTION_IMPORT: KeyBinding = KeyBinding::alt(KeyCode::Char('i'));
    pub const ACTION_EXPORT: KeyBinding = KeyBinding::alt(KeyCode::Char('x'));
}
