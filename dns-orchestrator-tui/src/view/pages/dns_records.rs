//! DNS 记录页面视图

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::model::App;
use crate::view::theme::colors;

/// 渲染 DNS 记录页面
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    if app.dns_records.records.is_empty() {
        render_empty(frame, area);
    } else {
        render_list(app, frame, area);
    }
}

/// 渲染空状态
fn render_empty(frame: &mut Frame, area: Rect) {
    use crate::i18n::t;
    let texts = t();
    let c = colors();
    let content = vec![
        Line::from(""),
        Line::styled(texts.dns_records.no_records, Style::default().fg(c.muted)),
        Line::from(""),
        Line::styled(texts.dns_records.hint_add_record, Style::default().fg(c.muted)),
    ];

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

/// 渲染 DNS 记录列表
fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let c = colors();
    let items: Vec<ListItem> = app
        .dns_records
        .records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let is_selected = i == app.dns_records.selected;
            let record_type = record.data.record_type().as_str();
            let record_value = record.data.display_value();

            // 截断过长的值（按字符而非字节截断，避免 UTF-8 panic）
            let max_value_len = 40;
            let display_value = if record_value.chars().count() > max_value_len {
                format!("{}...", record_value.chars().take(max_value_len).collect::<String>())
            } else {
                record_value
            };

            let style = if is_selected {
                Style::default()
                    .fg(c.selected_fg)
                    .bg(c.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(c.fg)
            };

            let type_style = if is_selected {
                Style::default()
                    .fg(c.selected_fg)
                    .bg(c.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let value_style = if is_selected {
                Style::default().fg(c.selected_fg).bg(c.selected_bg)
            } else {
                Style::default().fg(c.muted)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{record_type:6}"), type_style),
                Span::raw(" "),
                Span::styled(format!("{:20}", record.name), style),
                Span::raw(" → "),
                Span::styled(display_value, value_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default());

    let mut state = ListState::default();
    state.select(Some(app.dns_records.selected));

    frame.render_stateful_widget(list, area, &mut state);
}
