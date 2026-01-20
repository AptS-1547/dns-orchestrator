//! DNS 记录页面视图

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::model::App;

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
    let content = vec![
        Line::from(""),
        Line::styled(
            "  No DNS records found.",
            Style::default().fg(Color::Gray),
        ),
        Line::from(""),
        Line::styled(
            "  Press Alt+a to add a new record, or Esc to go back.",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

/// 渲染 DNS 记录列表
fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .dns_records
        .records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let is_selected = i == app.dns_records.selected;
            let record_type = record.data.record_type().as_str();
            let record_value = record.data.display_value();

            // 截断过长的值
            let max_value_len = 40;
            let display_value = if record_value.len() > max_value_len {
                format!("{}...", &record_value[..max_value_len])
            } else {
                record_value
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let type_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let value_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:6}", record_type), type_style),
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
