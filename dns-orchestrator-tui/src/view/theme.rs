//! 主题和样式定义

use ratatui::style::Color;
use std::sync::atomic::{AtomicU8, Ordering};

// 默认为 0 (Dark)，相应地，1 为 Light
static CURRENT_THEME: AtomicU8 = AtomicU8::new(0);

/// 设置主题（通过索引值）
/// 定义索引值 0 = Dark, 1 = Light
/// 这个函数接受 u8 而不是 Theme 类型
pub fn set_theme_index(index: u8) {
    CURRENT_THEME.store(index, Ordering::Relaxed);
}

/// 获取当前主题的颜色方案
pub fn colors() -> ThemeColors {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        0 => ThemeColors::dark(),
        _ => ThemeColors::light(),
    }
}

/// 主题颜色
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub border_focused: Color,
    pub highlight: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
}

impl ThemeColors {
    /// 深色主题
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 30),
            fg: Color::Rgb(212, 212, 212),
            border: Color::Rgb(62, 62, 62),
            border_focused: Color::Rgb(0, 122, 204),
            highlight: Color::Rgb(0, 122, 204),
            selected_bg: Color::Rgb(38, 79, 120),
            selected_fg: Color::White,
            success: Color::Rgb(78, 201, 176),
            warning: Color::Rgb(206, 145, 120),
            error: Color::Rgb(244, 135, 113),
            muted: Color::Rgb(128, 128, 128),
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            bg: Color::Rgb(250, 250, 250),
            fg: Color::Rgb(51, 51, 51),
            border: Color::Rgb(204, 204, 204),
            border_focused: Color::Rgb(0, 102, 204),
            highlight: Color::Rgb(0, 102, 204),
            selected_bg: Color::Rgb(204, 232, 255),
            selected_fg: Color::Black,
            success: Color::Rgb(34, 134, 58),
            warning: Color::Rgb(176, 136, 0),
            error: Color::Rgb(215, 58, 73),
            muted: Color::Rgb(128, 128, 128),
        }
    }
}

