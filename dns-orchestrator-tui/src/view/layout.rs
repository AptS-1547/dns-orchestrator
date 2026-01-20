//! 主布局渲染

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::i18n::t;
use crate::model::{App, Page};

use super::components;
use super::pages;
use super::theme::Styles;

/// 渲染主布局
pub fn render(app: &App, frame: &mut Frame) {
    let size = frame.area();

    // 三层布局：标题栏 + 主内容区 + 状态栏
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // 标题栏
            Constraint::Min(1),    // 主内容区
            Constraint::Length(1), // 状态栏
        ])
        .split(size);

    let title_area = main_layout[0];
    let content_area = main_layout[1];
    let status_area = main_layout[2];

    // 渲染标题栏
    render_title_bar(frame, title_area);

    // 左右分栏布局
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // 左侧导航
            Constraint::Percentage(80), // 右侧内容
        ])
        .split(content_area);

    let nav_area = columns[0];
    let page_area = columns[1];

    // 渲染左侧导航
    components::navigation::render(app, frame, nav_area);

    // 渲染右侧内容
    render_page_content(app, frame, page_area);

    // 渲染状态栏
    components::statusbar::render(app, frame, status_area);

    // 渲染弹窗（在最上层）
    components::modal::render(app, frame);
}

/// 渲染标题栏
fn render_title_bar(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(" DNS Orchestrator v0.1.0")
        .style(Style::default().bg(Color::Rgb(40, 40, 40)).fg(Color::White));
    frame.render_widget(title, area);
}

/// 根据当前页面渲染内容
fn render_page_content(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();

    // 内容区域的边框
    let is_focused = app.focus.is_content();
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border()
    };

    // 根据当前页面获取 i18n 标题
    let page_title = match &app.current_page {
        Page::Home => texts.nav.home,
        Page::Domains => texts.nav.domains,
        Page::DnsRecords { .. } => texts.dns_records.title,
        Page::Accounts => texts.nav.accounts,
        Page::Toolbox => texts.nav.toolbox,
        Page::Settings => texts.nav.settings,
    };

    let block = Block::default()
        .title(format!(" {} ", page_title))
        .title_style(Styles::title())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // 根据当前页面渲染具体内容
    match &app.current_page {
        Page::Home => pages::home::render(app, frame, inner_area),
        Page::Domains => pages::domains::render(app, frame, inner_area),
        Page::Accounts => pages::accounts::render(app, frame, inner_area),
        Page::Toolbox => pages::toolbox::render(app, frame, inner_area),
        Page::Settings => pages::settings::render(app, frame, inner_area),
        Page::DnsRecords { .. } => pages::dns_records::render(app, frame, inner_area),
    }
}