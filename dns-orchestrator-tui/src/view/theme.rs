//! 主题和样式定义

use ratatui::style::{Color, Modifier, Style};

/// 主题枚举
#[derive(Debug, Clone, Copy, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/// 获取当前主题的颜色方案
impl Theme {
    pub fn colors(&self) -> ThemeColors {
        match self {
            Theme::Dark => ThemeColors::dark(),
            Theme::Light => ThemeColors::light(),
        }
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
            bg: Color::Rgb(255, 255, 255),
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

/// 常用样式
pub struct Styles;

impl Styles {
    /// 普通边框样式
    pub fn border() -> Style {
        Style::default().fg(Color::Rgb(62, 62, 62))
    }

    /// 焦点边框样式
    pub fn border_focused() -> Style {
        Style::default().fg(Color::Rgb(0, 122, 204))
    }

    /// 选中项样式
    pub fn selected() -> Style {
        Style::default()
            .bg(Color::Rgb(38, 79, 120))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    /// 标题样式
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Rgb(212, 212, 212))
            .add_modifier(Modifier::BOLD)
    }

    /// 状态栏样式
    pub fn statusbar() -> Style {
        Style::default()
            .bg(Color::Rgb(0, 122, 204))
            .fg(Color::White)
    }

    /// 快捷键提示样式
    pub fn hint_key() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    /// 快捷键说明样式
    pub fn hint_desc() -> Style {
        Style::default().fg(Color::Rgb(180, 180, 180))
    }
}