//! 设置页面视图

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::i18n::t;
use crate::model::state::{PaginationMode, Theme};
use crate::model::App;

/// 设置项的标签宽度（用于对齐，基于显示宽度）
const LABEL_WIDTH: usize = 20;
/// 值区域的宽度（包含 < > 符号）
const VALUE_WIDTH: usize = 20;

/// 渲染设置页面
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let settings = &app.settings;

    let mut lines = vec![Line::from("")];

    // === 主题设置 ===
    let theme_value = match settings.theme {
        Theme::Dark => texts.settings.theme.dark,
        Theme::Light => texts.settings.theme.light,
    };
    lines.push(render_setting_row(
        texts.settings.theme.label,
        theme_value,
        settings.selected_index == 0,
    ));

    // === 语言设置 ===
    let lang_value = settings.language.display_name();
    lines.push(render_setting_row(
        texts.settings.language.label,
        lang_value,
        settings.selected_index == 1,
    ));

    // === 分页模式设置 ===
    let pagination_value = match settings.pagination_mode {
        PaginationMode::InfiniteScroll => texts.settings.pagination.infinite_scroll,
        PaginationMode::Traditional => texts.settings.pagination.traditional,
    };
    lines.push(render_setting_row(
        texts.settings.pagination.label,
        pagination_value,
        settings.selected_index == 2,
    ));

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // 操作提示
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", texts.hints.keys.arrows_ud), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {} | ", texts.hints.actions.move_up_down),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(texts.hints.keys.arrows_lr, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {} | ", texts.hints.actions.switch_option),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(texts.hints.keys.tab, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {}", texts.hints.actions.switch_panel),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// 渲染单行设置项
fn render_setting_row<'a>(label: &'a str, value: &'a str, is_selected: bool) -> Line<'a> {
    let prefix = if is_selected { "▶ " } else { "  " };

    let label_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let value_style = if is_selected {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    // 使用 unicode-width 计算显示宽度
    let label_display = label.to_string();
    let label_width = label.width();
    let label_padding = LABEL_WIDTH.saturating_sub(label_width);

    // 计算值的填充（居中显示在 < > 之间）
    let value_display = value.to_string();
    let value_width = value.width();
    let available_space = VALUE_WIDTH.saturating_sub(4); // 减去 "< " 和 " >" 的空间
    let left_padding = (available_space.saturating_sub(value_width)) / 2;
    let right_padding = available_space.saturating_sub(value_width).saturating_sub(left_padding);

    if is_selected {
        // 选中时显示 < value >
        Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled(format!("  {}", label_display), label_style),
            Span::styled(format!("{:width$}", "", width = label_padding), Style::default()),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled("◀ ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:>width$}", "", width = left_padding), Style::default()),
            Span::styled(value_display, value_style),
            Span::styled(format!("{:width$}", "", width = right_padding), Style::default()),
            Span::styled(" ▶", Style::default().fg(Color::Yellow)),
        ])
    } else {
        // 未选中时只显示值，但保持对齐
        Line::from(vec![
            Span::styled(prefix, label_style),
            Span::styled(format!("  {}", label_display), label_style),
            Span::styled(format!("{:width$}", "", width = label_padding), Style::default()),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled("  ", Style::default()), // 占位符，与 "◀ " 对齐
            Span::styled(format!("{:>width$}", "", width = left_padding), Style::default()),
            Span::styled(value_display, value_style),
            Span::styled(format!("{:width$}", "", width = right_padding), Style::default()),
            Span::styled("  ", Style::default()), // 占位符，与 " ▶" 对齐
        ])
    }
}