//! 工具箱页面视图

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::i18n::keys::ToolboxTabTexts;
use crate::i18n::t;
use crate::model::{App, ToolboxState, ToolboxTab};
use crate::view::theme::colors;

/// 获取工具箱标签页的翻译名称
fn get_tab_name(tab: ToolboxTab, tabs: &ToolboxTabTexts) -> &'static str {
    match tab {
        ToolboxTab::Whois => tabs.whois,
        ToolboxTab::DnsLookup => tabs.dns_lookup,
        ToolboxTab::IpLookup => tabs.ip_lookup,
        ToolboxTab::SslCheck => tabs.ssl_check,
        ToolboxTab::HttpHeaderCheck => tabs.http_headers,
        ToolboxTab::DnsPropagation => tabs.dns_propagation,
        ToolboxTab::DnssecCheck => tabs.dnssec_check,
    }
}

/// 渲染工具箱页面
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let c = colors();
    let current_tab = app.toolbox.current_tab;
    let visible_start = app.toolbox.visible_start;
    let all_tabs = ToolboxTab::all();
    let visible_count = ToolboxState::visible_tab_count();

    let mut lines = vec![Line::from("")];

    // 渲染选项卡（带滚动指示器）
    let mut tab_spans = vec![Span::raw("  ")];

    // 左侧滚动指示器
    if visible_start > 0 {
        tab_spans.push(Span::styled("< ", Style::default().fg(c.muted)));
    } else {
        tab_spans.push(Span::styled("  ", Style::default().fg(c.muted)));
    }

    // 渲染可见的标签页
    let visible_end = (visible_start + visible_count).min(all_tabs.len());
    for (i, tab) in all_tabs[visible_start..visible_end].iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw(" | "));
        }
        let is_selected = *tab == current_tab;
        let style = if is_selected {
            Style::default()
                .fg(c.highlight)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(c.muted)
        };
        tab_spans.push(Span::styled(get_tab_name(*tab, &texts.toolbox.tabs), style));
    }

    // 右侧滚动指示器
    if visible_end < all_tabs.len() {
        tab_spans.push(Span::styled(" >", Style::default().fg(c.muted)));
    } else {
        tab_spans.push(Span::styled("  ", Style::default().fg(c.muted)));
    }

    lines.push(Line::from(tab_spans));

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "  ────────────────────────────────────────",
        Style::default().fg(c.muted),
    ));
    lines.push(Line::from(""));

    // 根据当前标签页渲染内容
    match current_tab {
        ToolboxTab::Whois => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.whois),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_domain),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::DnsLookup => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.dns_lookup),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_domain),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::IpLookup => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.ip_lookup),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_ip_or_domain),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::SslCheck => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.ssl_check),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_domain),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::HttpHeaderCheck => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.http_header),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_url),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::DnsPropagation => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.dns_propagation),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_domain),
                Style::default().fg(c.muted),
            ));
        }
        ToolboxTab::DnssecCheck => {
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.titles.dnssec),
                Style::default().fg(c.fg).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                format!("  {}", texts.modal.tools.placeholders.enter_domain),
                Style::default().fg(c.muted),
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}", texts.hints.keys.arrows_lr),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!(" {} | ", texts.hints.actions.switch_option),
            Style::default().fg(c.muted),
        ),
        Span::styled(texts.hints.keys.enter, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!(" {}", texts.common.confirm),
            Style::default().fg(c.muted),
        ),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
