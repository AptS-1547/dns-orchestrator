//! 域名列表页面视图

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::i18n::t;
use crate::model::App;

/// 渲染域名列表页面
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    if app.domains.domains.is_empty() {
        render_empty(frame, area);
    } else {
        render_list(app, frame, area);
    }
}

/// 渲染空状态
fn render_empty(frame: &mut Frame, area: Rect) {
    let texts = t();
    let content = vec![
        Line::from(""),
        Line::styled(
            format!("  {}", texts.domains.no_domains),
            Style::default().fg(Color::Gray),
        ),
        Line::from(""),
        Line::styled(
            format!("  {}", texts.accounts.add_account),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let paragraph = ratatui::widgets::Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

/// 渲染域名列表
fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let items: Vec<ListItem> = app
        .domains
        .domains
        .iter()
        .enumerate()
        .map(|(i, domain)| {
            let is_selected = i == app.domains.selected;
            let status_icon = match domain.status {
                crate::model::domain::DomainStatus::Active => "●",
                crate::model::domain::DomainStatus::Pending => "○",
                _ => "○",
            };
            let status_color = match domain.status {
                crate::model::domain::DomainStatus::Active => Color::Green,
                crate::model::domain::DomainStatus::Pending => Color::Yellow,
                _ => Color::Gray,
            };
            let record_count = domain
                .record_count
                .map(|c| format!(" ({} {})", c, texts.domains.record_count))
                .unwrap_or_default();

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_style = if is_selected {
                Style::default().fg(status_color).bg(Color::Cyan)
            } else {
                Style::default().fg(status_color)
            };

            let dim_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(status_icon, status_style),
                Span::raw(" "),
                Span::styled(&domain.name, style),
                Span::styled(record_count, dim_style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default());

    let mut state = ListState::default();
    state.select(Some(app.domains.selected));

    frame.render_stateful_widget(list, area, &mut state);
}
