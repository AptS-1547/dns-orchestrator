//! 工具箱页面视图

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::model::{App, ToolboxTab, ToolboxState};

/// 渲染工具箱页面
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let current_tab = app.toolbox.current_tab;
    let visible_start = app.toolbox.visible_start;
    let all_tabs = ToolboxTab::all();
    let visible_count = ToolboxState::visible_tab_count();

    let mut lines = vec![Line::from("")];

    // 渲染选项卡（带滚动指示器）
    let mut tab_spans = vec![Span::raw("  ")];

    // 左侧滚动指示器
    if visible_start > 0 {
        tab_spans.push(Span::styled("◀ ", Style::default().fg(Color::Yellow)));
    } else {
        tab_spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Gray)
        };
        tab_spans.push(Span::styled(tab.name(), style));
    }

    // 右侧滚动指示器
    if visible_end < all_tabs.len() {
        tab_spans.push(Span::styled(" ▶", Style::default().fg(Color::Yellow)));
    } else {
        tab_spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
    }

    lines.push(Line::from(tab_spans));

    lines.push(Line::from(""));
    lines.push(Line::styled(
        "  ────────────────────────────────────────",
        Style::default().fg(Color::DarkGray),
    ));
    lines.push(Line::from(""));

    // 根据当前标签页渲染内容
    match current_tab {
        ToolboxTab::Whois => {
            lines.push(Line::styled(
                "  WHOIS Lookup",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Query domain registration information.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::DnsLookup => {
            lines.push(Line::styled(
                "  DNS Lookup",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Query DNS records for a domain.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::IpLookup => {
            lines.push(Line::styled(
                "  IP Lookup",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Get geolocation and ISP information for an IP.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::SslCheck => {
            lines.push(Line::styled(
                "  SSL Certificate Check",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Check SSL certificate status and expiry.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::HttpHeaderCheck => {
            lines.push(Line::styled(
                "  HTTP Header Check",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Inspect HTTP response headers from a URL.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::DnsPropagation => {
            lines.push(Line::styled(
                "  DNS Propagation Check",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Check DNS propagation across global DNS servers.",
                Style::default().fg(Color::Gray),
            ));
        }
        ToolboxTab::DnssecCheck => {
            lines.push(Line::styled(
                "  DNSSEC Validation",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "  Verify DNSSEC configuration for a domain.",
                Style::default().fg(Color::Gray),
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!("  Placeholder: {}", app.toolbox.placeholder()),
        Style::default().fg(Color::DarkGray),
    ));
    lines.push(Line::from(""));
    lines.push(Line::styled(
        "  Press Tab to switch tools, Enter to execute.",
        Style::default().fg(Color::DarkGray),
    ));

    // 显示结果或错误
    if let Some(ref result) = app.toolbox.result {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "  Result:",
            Style::default().fg(Color::Green),
        ));
        lines.push(Line::styled(
            format!("  {}", result),
            Style::default().fg(Color::White),
        ));
    }

    if let Some(ref error) = app.toolbox.error {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!("  Error: {}", error),
            Style::default().fg(Color::Red),
        ));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
