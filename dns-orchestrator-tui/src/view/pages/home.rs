//! 首页视图

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::i18n::t;
use crate::model::App;
use crate::view::theme::colors;

/// 渲染首页
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let c = colors();

    // 首页布局：欢迎信息 + 统计信息
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // 欢迎区域
            Constraint::Min(1),    // 统计区域
        ])
        .split(area);

    // 欢迎信息
    let welcome = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", texts.home.welcome),
            Style::default()
                .fg(c.highlight)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", texts.home.welcome_desc),
            Style::default().fg(c.muted),
        )),
        Line::from(""),
    ];

    let welcome_widget = Paragraph::new(welcome);
    frame.render_widget(welcome_widget, layout[0]);

    // 统计信息（目前是占位符）
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // 账号统计
    let accounts_block = Block::default()
        .title(format!(" {} ", texts.nav.accounts))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(c.border));

    let accounts_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", app.accounts.accounts.len()),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", texts.home.manage_accounts),
            Style::default().fg(c.muted),
        )),
    ])
    .block(accounts_block);

    frame.render_widget(accounts_content, stats_layout[0]);

    // 域名统计
    let domains_block = Block::default()
        .title(format!(" {} ", texts.nav.domains))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(c.border));

    let domains_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", app.domains.domains.len()),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", texts.home.manage_domains),
            Style::default().fg(c.muted),
        )),
    ])
    .block(domains_block);

    frame.render_widget(domains_content, stats_layout[1]);
}
