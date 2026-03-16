//! 域名列表页面视图

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
};

use crate::i18n::t;
use crate::model::App;
use crate::view::theme::colors;

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
    let c = colors();
    let content = vec![
        Line::from(""),
        Line::styled(texts.domains.no_domains, Style::default().fg(c.muted)),
        Line::from(""),
        Line::styled(texts.domains.hint_add_account, Style::default().fg(c.muted)),
    ];

    let paragraph = ratatui::widgets::Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

/// 渲染域名列表
fn render_list(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let c = colors();
    let items: Vec<ListItem> = app
        .domains
        .domains
        .iter()
        .enumerate()
        .map(|(i, domain)| {
            let is_selected = i == app.domains.selected;
            let status_icon = match domain.status {
                crate::model::domain::DomainStatus::Active => "●",
                _ => "○",
            };
            let status_color = match domain.status {
                crate::model::domain::DomainStatus::Active => Color::Green,
                crate::model::domain::DomainStatus::Pending => Color::Yellow,
                _ => c.muted,
            };
            let record_count = domain
                .record_count
                .map(|count| format!(" ({} {})", count, texts.domains.record_count))
                .unwrap_or_default();

            let style = if is_selected {
                Style::default()
                    .fg(c.selected_fg)
                    .bg(c.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(c.fg)
            };

            let status_style = if is_selected {
                Style::default().fg(status_color).bg(c.selected_bg)
            } else {
                Style::default().fg(status_color)
            };

            let dim_style = if is_selected {
                Style::default().fg(c.selected_fg).bg(c.selected_bg)
            } else {
                Style::default().fg(c.muted)
            };

            let fav_icon = if domain.is_favorite { "★ " } else { "  " };
            let fav_style = if is_selected {
                Style::default().fg(Color::Yellow).bg(c.selected_bg)
            } else {
                Style::default().fg(Color::Yellow)
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(fav_icon, fav_style),
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
