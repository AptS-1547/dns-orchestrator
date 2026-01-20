//! 首页视图

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::i18n::t;
use crate::model::App;

/// 渲染首页
pub fn render(_app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();

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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", texts.home.welcome_desc),
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
    ];

    let welcome_widget = Paragraph::new(welcome);
    frame.render_widget(welcome_widget, layout[0]);

    // 统计信息（目前是占位符）
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(layout[1]);

    // 账号统计
    let accounts_block = Block::default()
        .title(format!(" {} ", texts.nav.accounts))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let accounts_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("  0", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {}", texts.home.manage_accounts), Style::default().fg(Color::Gray))),
    ])
    .block(accounts_block);

    frame.render_widget(accounts_content, stats_layout[0]);

    // 域名统计
    let domains_block = Block::default()
        .title(format!(" {} ", texts.nav.domains))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let domains_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("  0", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("  {}", texts.home.manage_domains), Style::default().fg(Color::Gray))),
    ])
    .block(domains_block);

    frame.render_widget(domains_content, stats_layout[1]);
}