//! 事件处理器

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::message::{AppMessage, ContentMessage, ModalMessage, NavigationMessage};
use crate::model::{App, Page};




/// 轮询事件
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}




/// 处理事件，返回对应的消息
pub fn handle_event(event: Event, app: &App) -> AppMessage {
    match event {
        Event::Key(key_event) => handle_key_event(key_event, app),      // 键盘事件
        Event::Resize(_, _) => AppMessage::Noop,                                  // 终端窗口大小改变，自动重绘
        _ => AppMessage::Noop,
    }
}




/// 处理键盘事件
fn handle_key_event(key: KeyEvent, app: &App) -> AppMessage {
    // 重要：只处理 Press 事件，忽略 Release 和 Repeat
    // 避免 Windows 终端上按键重复问题的发生
    if key.kind != KeyEventKind::Press {
        return AppMessage::Noop;
    }

    // 如果有弹窗打开，优先处理弹窗输入
    if app.modal.is_open() {
        return handle_modal_keys(key, app);
    }

    // 全局快捷键（无论焦点在哪里）
    match (key.modifiers, key.code) {
        // Ctrl+C: 强制退出
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return AppMessage::Quit,

        // Tab: 切换焦点面板
        (KeyModifiers::NONE, KeyCode::Tab) => {
            return AppMessage::ToggleFocus;
        }

        // Alt+h 或 ?: 显示帮助
        (KeyModifiers::ALT, KeyCode::Char('h')) | (KeyModifiers::NONE, KeyCode::Char('?')) => {
            return AppMessage::ShowHelp;
        }

        // Alt+q: 退出
        (KeyModifiers::ALT, KeyCode::Char('q')) => return AppMessage::Quit,

        // Alt+r: 刷新
        (KeyModifiers::ALT, KeyCode::Char('r')) => return AppMessage::Refresh,

        // Esc: 返回/取消
        (KeyModifiers::NONE, KeyCode::Esc) => return AppMessage::GoBack,

        _ => {}
    }

    // 根据焦点位置处理按键
    if app.focus.is_navigation() {
        handle_navigation_keys(key)
    } else {
        handle_content_keys(key, app)
    }
}

/// 处理导航面板的按键
fn handle_navigation_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // ↑ 或 k: 上移
        KeyCode::Up | KeyCode::Char('k') => {
            AppMessage::Navigation(NavigationMessage::SelectPrevious)
        }

        // ↓ 或 j: 下移
        KeyCode::Down | KeyCode::Char('j') => {
            AppMessage::Navigation(NavigationMessage::SelectNext)
        }

        // Enter: 确认选择
        KeyCode::Enter => AppMessage::Navigation(NavigationMessage::Confirm),

        // Home: 跳到第一项
        KeyCode::Home => AppMessage::Navigation(NavigationMessage::SelectFirst),

        // End: 跳到最后一项
        KeyCode::End => AppMessage::Navigation(NavigationMessage::SelectLast),

        _ => AppMessage::Noop,
    }
}

/// 处理内容面板的按键
fn handle_content_keys(key: KeyEvent, app: &App) -> AppMessage {
    // 通用操作快捷键（Alt+键）
    match (key.modifiers, key.code) {
        // Alt+a: 添加
        (KeyModifiers::ALT, KeyCode::Char('a')) => {
            return AppMessage::Content(ContentMessage::Add);
        }
        // Alt+e: 编辑
        (KeyModifiers::ALT, KeyCode::Char('e')) => {
            return AppMessage::Content(ContentMessage::Edit);
        }
        // Alt+d: 删除
        (KeyModifiers::ALT, KeyCode::Char('d')) => {
            return AppMessage::Content(ContentMessage::Delete);
        }
        // Alt+i: 导入
        (KeyModifiers::ALT, KeyCode::Char('i')) => {
            return AppMessage::Content(ContentMessage::Import);
        }
        // Alt+x: 导出
        (KeyModifiers::ALT, KeyCode::Char('x')) => {
            return AppMessage::Content(ContentMessage::Export);
        }
        _ => {}
    }

    // 根据当前页面处理特定按键
    match &app.current_page {
        Page::Toolbox => handle_toolbox_keys(key),
        Page::Settings => handle_settings_keys(key),
        _ => handle_list_keys(key),
    }
}

/// 处理列表类页面的按键（通用）
fn handle_list_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // ↑ 或 k: 上一项
        KeyCode::Up | KeyCode::Char('k') => {
            AppMessage::Content(ContentMessage::SelectPrevious)
        }
        // ↓ 或 j: 下一项
        KeyCode::Down | KeyCode::Char('j') => {
            AppMessage::Content(ContentMessage::SelectNext)
        }
        // Enter: 确认选择
        KeyCode::Enter => {
            AppMessage::Content(ContentMessage::Confirm)
        }
        // Home: 跳到第一项
        KeyCode::Home => {
            AppMessage::Content(ContentMessage::SelectFirst)
        }
        // End: 跳到最后一项
        KeyCode::End => {
            AppMessage::Content(ContentMessage::SelectLast)
        }
        _ => AppMessage::Noop,
    }
}

/// 处理工具箱页面的按键
fn handle_toolbox_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // Enter: 执行工具
        KeyCode::Enter => {
            AppMessage::Content(ContentMessage::Execute)
        }
        // ← 或 ↑ 或 k: 上一个工具
        KeyCode::Left | KeyCode::Up | KeyCode::Char('k') => {
            AppMessage::Content(ContentMessage::SelectPrevious)
        }
        // → 或 ↓ 或 j: 下一个工具
        KeyCode::Right | KeyCode::Down | KeyCode::Char('j') => {
            AppMessage::Content(ContentMessage::SelectNext)
        }
        _ => AppMessage::Noop,
    }
}

/// 处理弹窗中的按键
fn handle_modal_keys(key: KeyEvent, app: &App) -> AppMessage {
    use crate::model::state::Modal;

    // Esc 和 Ctrl+C 始终可以关闭弹窗
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            return AppMessage::Modal(ModalMessage::Close);
        }
        (KeyModifiers::NONE, KeyCode::Esc) => {
            return AppMessage::Modal(ModalMessage::Close);
        }
        _ => {}
    }

    // 根据弹窗类型处理按键
    let Some(ref modal) = app.modal.active else {
        return AppMessage::Noop;
    };

    match modal {
        Modal::AddAccount { focus, .. } => handle_add_account_keys(key, *focus),
        Modal::ConfirmDelete { .. } => handle_confirm_delete_keys(key),
        Modal::DnsLookup { focus, .. } => handle_dns_lookup_keys(key, *focus),
        Modal::WhoisLookup { .. }
        | Modal::SslCheck { .. }
        | Modal::IpLookup { .. }
        | Modal::DnssecCheck { .. } => handle_simple_tool_keys(key),
        Modal::HttpHeaderCheck { focus, .. } => handle_http_header_check_keys(key, *focus),
        Modal::DnsPropagation { focus, .. } => handle_dns_propagation_keys(key, *focus),
        Modal::Help | Modal::Error { .. } => {
            // 帮助和错误弹窗只响应关闭按键
            match key.code {
                KeyCode::Enter | KeyCode::Esc => AppMessage::Modal(ModalMessage::Close),
                _ => AppMessage::Noop,
            }
        }
        _ => AppMessage::Noop,
    }
}

/// 处理添加账号弹窗的按键
fn handle_add_account_keys(key: KeyEvent, focus: usize) -> AppMessage {
    match key.code {
        // Tab: 下一个字段
        KeyCode::Tab => AppMessage::Modal(ModalMessage::NextField),

        // Shift+Tab: 上一个字段
        KeyCode::BackTab => AppMessage::Modal(ModalMessage::PrevField),

        // ↓: 下一个字段
        KeyCode::Down => AppMessage::Modal(ModalMessage::NextField),

        // ↑: 上一个字段
        KeyCode::Up => AppMessage::Modal(ModalMessage::PrevField),

        // ← →: 切换服务商（仅当焦点在服务商字段时）
        KeyCode::Left => {
            if focus == 0 {
                AppMessage::Modal(ModalMessage::PrevProvider)
            } else {
                AppMessage::Noop
            }
        }
        KeyCode::Right => {
            if focus == 0 {
                AppMessage::Modal(ModalMessage::NextProvider)
            } else {
                AppMessage::Noop
            }
        }

        // Enter: 确认
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        // Backspace: 删除字符
        KeyCode::Backspace => AppMessage::Modal(ModalMessage::Backspace),

        // Delete: 删除字符
        KeyCode::Delete => AppMessage::Modal(ModalMessage::Delete),

        // 字符输入
        KeyCode::Char(ch) => {
            // Alt+s 切换密码可见性
            if key.modifiers.contains(KeyModifiers::ALT) && ch == 's' {
                AppMessage::Modal(ModalMessage::ToggleSecrets)
            } else if key.modifiers.is_empty() {
                // 普通字符输入（仅当焦点不在服务商字段时）
                if focus > 0 {
                    AppMessage::Modal(ModalMessage::Input(ch))
                } else {
                    AppMessage::Noop
                }
            } else {
                AppMessage::Noop
            }
        }

        _ => AppMessage::Noop,
    }
}

/// 处理确认删除弹窗的按键
fn handle_confirm_delete_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // Tab 或 ← →: 切换焦点
        KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
            AppMessage::Modal(ModalMessage::ToggleDeleteFocus)
        }

        // Enter: 确认
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        _ => AppMessage::Noop,
    }
}

/// 处理 DNS Lookup 工具弹窗的按键
fn handle_dns_lookup_keys(key: KeyEvent, focus: usize) -> AppMessage {
    match key.code {
        // Tab / ↓: 下一个字段
        KeyCode::Tab | KeyCode::Down => AppMessage::Modal(ModalMessage::NextField),

        // Shift+Tab / ↑: 上一个字段
        KeyCode::BackTab | KeyCode::Up => AppMessage::Modal(ModalMessage::PrevField),

        // ← →: 切换选项（仅当焦点在记录类型或DNS服务器字段时）
        KeyCode::Left => {
            if focus == 1 || focus == 2 {
                AppMessage::Modal(ModalMessage::PrevProvider)
            } else {
                AppMessage::Noop
            }
        }
        KeyCode::Right => {
            if focus == 1 || focus == 2 {
                AppMessage::Modal(ModalMessage::NextProvider)
            } else {
                AppMessage::Noop
            }
        }

        // Enter: 执行查询
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        // Backspace: 删除字符
        KeyCode::Backspace => AppMessage::Modal(ModalMessage::Backspace),

        // Delete: 删除字符
        KeyCode::Delete => AppMessage::Modal(ModalMessage::Delete),

        // 字符输入（仅当焦点在域名输入框时）
        KeyCode::Char(ch) if key.modifiers.is_empty() && focus == 0 => {
            AppMessage::Modal(ModalMessage::Input(ch))
        }

        _ => AppMessage::Noop,
    }
}

/// 处理简单工具弹窗的按键（WHOIS、SSL、IP、DNSSEC）
fn handle_simple_tool_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // Enter: 执行查询
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        // Backspace: 删除字符
        KeyCode::Backspace => AppMessage::Modal(ModalMessage::Backspace),

        // Delete: 删除字符
        KeyCode::Delete => AppMessage::Modal(ModalMessage::Delete),

        // 字符输入
        KeyCode::Char(ch) if key.modifiers.is_empty() => {
            AppMessage::Modal(ModalMessage::Input(ch))
        }

        _ => AppMessage::Noop,
    }
}

/// 处理 HTTP Header Check 工具弹窗的按键
fn handle_http_header_check_keys(key: KeyEvent, focus: usize) -> AppMessage {
    match key.code {
        // Tab / ↓: 下一个字段
        KeyCode::Tab | KeyCode::Down => AppMessage::Modal(ModalMessage::NextField),

        // Shift+Tab / ↑: 上一个字段
        KeyCode::BackTab | KeyCode::Up => AppMessage::Modal(ModalMessage::PrevField),

        // ← →: 切换 HTTP 方法（仅当焦点在方法字段时）
        KeyCode::Left => {
            if focus == 1 {
                AppMessage::Modal(ModalMessage::PrevProvider)
            } else {
                AppMessage::Noop
            }
        }
        KeyCode::Right => {
            if focus == 1 {
                AppMessage::Modal(ModalMessage::NextProvider)
            } else {
                AppMessage::Noop
            }
        }

        // Enter: 执行查询
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        // Backspace: 删除字符
        KeyCode::Backspace => AppMessage::Modal(ModalMessage::Backspace),

        // Delete: 删除字符
        KeyCode::Delete => AppMessage::Modal(ModalMessage::Delete),

        // 字符输入（仅当焦点在 URL 输入框时）
        KeyCode::Char(ch) if key.modifiers.is_empty() && focus == 0 => {
            AppMessage::Modal(ModalMessage::Input(ch))
        }

        _ => AppMessage::Noop,
    }
}

/// 处理 DNS Propagation 工具弹窗的按键
fn handle_dns_propagation_keys(key: KeyEvent, focus: usize) -> AppMessage {
    match key.code {
        // Tab / ↓: 下一个字段
        KeyCode::Tab | KeyCode::Down => AppMessage::Modal(ModalMessage::NextField),

        // Shift+Tab / ↑: 上一个字段
        KeyCode::BackTab | KeyCode::Up => AppMessage::Modal(ModalMessage::PrevField),

        // ← →: 切换记录类型（仅当焦点在记录类型字段时）
        KeyCode::Left => {
            if focus == 1 {
                AppMessage::Modal(ModalMessage::PrevProvider)
            } else {
                AppMessage::Noop
            }
        }
        KeyCode::Right => {
            if focus == 1 {
                AppMessage::Modal(ModalMessage::NextProvider)
            } else {
                AppMessage::Noop
            }
        }

        // Enter: 执行查询
        KeyCode::Enter => AppMessage::Modal(ModalMessage::Confirm),

        // Backspace: 删除字符
        KeyCode::Backspace => AppMessage::Modal(ModalMessage::Backspace),

        // Delete: 删除字符
        KeyCode::Delete => AppMessage::Modal(ModalMessage::Delete),

        // 字符输入（仅当焦点在域名输入框时）
        KeyCode::Char(ch) if key.modifiers.is_empty() && focus == 0 => {
            AppMessage::Modal(ModalMessage::Input(ch))
        }

        _ => AppMessage::Noop,
    }
}

/// 处理设置页面的按键
fn handle_settings_keys(key: KeyEvent) -> AppMessage {
    match key.code {
        // ↑ 或 k: 上一个设置项
        KeyCode::Up | KeyCode::Char('k') => {
            AppMessage::Content(ContentMessage::SelectPrevious)
        }
        // ↓ 或 j: 下一个设置项
        KeyCode::Down | KeyCode::Char('j') => {
            AppMessage::Content(ContentMessage::SelectNext)
        }
        // ←: 切换到上一个值
        KeyCode::Left => {
            AppMessage::Content(ContentMessage::TogglePrev)
        }
        // →: 切换到下一个值
        KeyCode::Right => {
            AppMessage::Content(ContentMessage::ToggleNext)
        }
        _ => AppMessage::Noop,
    }
}
