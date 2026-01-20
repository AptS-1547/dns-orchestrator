//! 弹窗组件

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::i18n::t;
use crate::model::state::{
    get_all_dns_servers, get_all_providers, get_all_record_types, get_credential_fields, Modal,
};
use crate::model::App;

/// 渲染带光标的输入框
///
/// - `value`: 输入的值
/// - `placeholder`: 占位符文本
/// - `is_focused`: 是否聚焦（显示光标）
/// - `cursor_pos`: 光标位置（通常是 value.len()）
fn render_input_with_cursor<'a>(
    value: &'a str,
    placeholder: &'a str,
    is_focused: bool,
) -> Vec<Span<'a>> {
    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);

    if value.is_empty() {
        if is_focused {
            // 空输入框聚焦时：光标 + placeholder
            vec![
                Span::styled(" ", cursor_style),
                Span::styled(placeholder, Style::default().fg(Color::DarkGray)),
            ]
        } else {
            // 空输入框未聚焦：只显示 placeholder
            vec![Span::styled(placeholder, Style::default().fg(Color::DarkGray))]
        }
    } else {
        if is_focused {
            // 有内容聚焦时：内容 + 光标
            vec![
                Span::styled(value, Style::default().fg(Color::White)),
                Span::styled(" ", cursor_style),
            ]
        } else {
            // 有内容未聚焦：只显示内容
            vec![Span::styled(value, Style::default().fg(Color::White))]
        }
    }
}

/// 渲染弹窗（如果有活动弹窗）
pub fn render(app: &App, frame: &mut Frame) {
    let Some(ref modal) = app.modal.active else {
        return;
    };

    match modal {
        Modal::AddAccount { .. } => render_add_account(app, frame, modal),
        Modal::ConfirmDelete { .. } => render_confirm_delete(frame, modal),
        Modal::DnsLookup { .. } => render_dns_lookup(frame, modal),
        Modal::WhoisLookup { .. } => render_whois_lookup(frame, modal),
        Modal::SslCheck { .. } => render_ssl_check(frame, modal),
        Modal::IpLookup { .. } => render_ip_lookup(frame, modal),
        Modal::HttpHeaderCheck { .. } => render_http_header_check(frame, modal),
        Modal::DnsPropagation { .. } => render_dns_propagation(frame, modal),
        Modal::DnssecCheck { .. } => render_dnssec_check(frame, modal),
        Modal::Error { title, message } => render_error(frame, title, message),
        Modal::Help => render_help(frame),
        _ => {}
    }
}

/// 计算居中弹窗区域
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// 渲染添加账号弹窗
fn render_add_account(_app: &App, frame: &mut Frame, modal: &Modal) {
    let Modal::AddAccount {
        provider_index,
        name,
        credential_values,
        focus,
        show_secrets,
        error,
    } = modal
    else {
        return;
    };

    let texts = t();
    let providers = get_all_providers();
    let current_provider = &providers[*provider_index];
    let credential_fields = get_credential_fields(current_provider);

    // 计算弹窗高度：标题(3) + 服务商(3) + 名称(3) + 凭证字段(每个3) + 错误(2) + 按钮(3) + 边框(2)
    let height = 3 + 3 + 3 + (credential_fields.len() as u16 * 3) + 2 + 3 + 2;
    let area = centered_rect(50, height, frame.area());

    // 清除背景
    frame.render_widget(Clear, area);

    // 弹窗边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.add_account.title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    // 内容区域
    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);

    let mut lines = Vec::new();

    // === 服务商选择 ===
    let provider_focused = *focus == 0;
    let provider_style = if provider_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    lines.push(Line::from(vec![
        Span::styled(texts.modal.add_account.provider, Style::default().fg(Color::Gray)),
        if provider_focused {
            Span::styled(format!(" {}", texts.modal.add_account.provider_hint), Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]));

    let provider_display = format!(
        "  {} {} {}",
        if provider_focused { "◀" } else { " " },
        current_provider.display_name(),
        if provider_focused { "▶" } else { " " }
    );
    lines.push(Line::styled(provider_display, provider_style));
    lines.push(Line::from(""));

    // === 账号名称 ===
    let name_focused = *focus == 1;
    let name_style = if name_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    lines.push(Line::from(Span::styled(
        texts.modal.add_account.account_name,
        Style::default().fg(Color::Gray),
    )));

    let name_display = if name.is_empty() && !name_focused {
        format!("  {}{} {}", texts.modal.add_account.account_name_example, current_provider.display_name(), texts.modal.add_account.main_account)
    } else if name_focused {
        format!("  {}▎", name)
    } else {
        format!("  {}", name)
    };

    let name_line_style = if name.is_empty() && !name_focused {
        Style::default().fg(Color::DarkGray)
    } else {
        name_style
    };
    lines.push(Line::styled(name_display, name_line_style));
    lines.push(Line::from(""));

    // === 凭证字段 ===
    for (i, field) in credential_fields.iter().enumerate() {
        let field_focused = *focus == 2 + i;
        let field_style = if field_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let label = if field.is_secret {
            format!("{} ⊖", field.label)
        } else {
            field.label.to_string()
        };
        lines.push(Line::from(Span::styled(label, Style::default().fg(Color::Gray))));

        let value = credential_values.get(i).map(|s| s.as_str()).unwrap_or("");
        let display_value = if field.is_secret && !show_secrets && !value.is_empty() {
            "•".repeat(value.len().min(20))
        } else {
            value.to_string()
        };

        let value_display = if value.is_empty() && !field_focused {
            format!("  {}", field.placeholder)
        } else if field_focused {
            format!("  {}▎", display_value)
        } else {
            format!("  {}", display_value)
        };

        let value_style = if value.is_empty() && !field_focused {
            Style::default().fg(Color::DarkGray)
        } else {
            field_style
        };
        lines.push(Line::styled(value_display, value_style));
        lines.push(Line::from(""));
    }

    // === 错误信息 ===
    if let Some(err) = error {
        lines.push(Line::styled(
            format!("  ⚠ {}", err),
            Style::default().fg(Color::Red),
        ));
    }

    // === 操作提示 ===
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("  {}", texts.hints.keys.tab), Style::default().fg(Color::Yellow)),
        Span::styled(format!(" {} | ", texts.common.next), Style::default().fg(Color::DarkGray)),
        Span::styled(texts.hints.keys.enter, Style::default().fg(Color::Yellow)),
        Span::styled(format!(" {} | ", texts.common.confirm), Style::default().fg(Color::DarkGray)),
        Span::styled(texts.hints.keys.esc, Style::default().fg(Color::Yellow)),
        Span::styled(format!(" {}", texts.common.cancel), Style::default().fg(Color::DarkGray)),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染确认删除弹窗
fn render_confirm_delete(frame: &mut Frame, modal: &Modal) {
    let Modal::ConfirmDelete {
        item_type,
        item_name,
        focus,
        ..
    } = modal
    else {
        return;
    };

    let texts = t();
    let area = centered_rect(40, 9, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", texts.modal.confirm_delete.title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);

    let cancel_style = if *focus == 0 {
        Style::default().fg(Color::Black).bg(Color::White)
    } else {
        Style::default().fg(Color::White)
    };

    let confirm_style = if *focus == 1 {
        Style::default().fg(Color::Black).bg(Color::Red)
    } else {
        Style::default().fg(Color::Red)
    };

    let lines = vec![
        Line::from(""),
        Line::styled(
            format!("  {} {} ?", texts.modal.confirm_delete.message, item_type),
            Style::default().fg(Color::White),
        ),
        Line::styled(
            format!("  \"{}\"", item_name),
            Style::default().fg(Color::Yellow),
        ),
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(format!(" {} ", texts.modal.confirm_delete.cancel_button), cancel_style),
            Span::raw("    "),
            Span::styled(format!(" {} ", texts.modal.confirm_delete.confirm_button), confirm_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染错误弹窗
fn render_error(frame: &mut Frame, title: &str, message: &str) {
    let texts = t();
    let area = centered_rect(50, 8, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);

    let lines = vec![
        Line::styled(message, Style::default().fg(Color::White)),
        Line::from(""),
        Line::styled(
            format!("{}/{}: {}", texts.hints.keys.esc, texts.hints.keys.enter, texts.common.close),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

/// 渲染帮助弹窗
fn render_help(frame: &mut Frame) {
    let texts = t();
    let area = centered_rect(55, 18, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", texts.help.title))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);

    let lines = vec![
        Line::styled(texts.help.global_shortcuts, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ←→     ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.switch_panel, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓/jk  ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.move_up_down, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter  ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.confirm, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc    ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.back_cancel, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  q      ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.quit, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::styled(texts.help.operation_shortcuts, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Alt+a  ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.add, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Alt+e  ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.edit, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Alt+d  ", Style::default().fg(Color::Yellow)),
            Span::styled(texts.help.actions.delete, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::styled(texts.help.close_hint, Style::default().fg(Color::DarkGray)),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 DNS 查询工具弹窗
fn render_dns_lookup(frame: &mut Frame, modal: &Modal) {
    let Modal::DnsLookup {
        domain,
        record_type_index,
        dns_server_index,
        focus,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let record_types = get_all_record_types();
    let dns_servers = get_all_dns_servers();

    let area = frame.area();
    let modal_area = centered_rect(70, 20, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.dns_lookup))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // 域名输入框
    let domain_style = if *focus == 0 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.domain, domain_style)]));

    let mut domain_spans = vec![Span::styled("  ", Style::default())];
    domain_spans.extend(render_input_with_cursor(
        domain,
        texts.modal.tools.placeholders.enter_domain,
        *focus == 0,
    ));
    lines.push(Line::from(domain_spans));
    lines.push(Line::from(""));

    // 记录类型选择
    let record_type = &record_types[*record_type_index];
    let record_style = if *focus == 1 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.record_type, record_style)]));
    lines.push(Line::from(vec![
        Span::styled("  < ", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(
            record_type.name(),
            if *focus == 1 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        Span::styled(" >", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
    ]));
    lines.push(Line::from(""));

    // DNS 服务器选择
    let dns_server = &dns_servers[*dns_server_index];
    let server_style = if *focus == 2 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.dns_server, server_style)]));
    lines.push(Line::from(vec![
        Span::styled("  < ", if *focus == 2 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(
            dns_server.name(),
            if *focus == 2 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        Span::styled(" >", if *focus == 2 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
    ]));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.querying, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}/{}: {} | {}: {} | {}: {} | {}: {} ",
            texts.hints.keys.tab,
            texts.hints.keys.arrows_ud,
            texts.hints.actions.navigate,
            texts.hints.keys.arrows_lr,
            texts.hints.actions.switch_option,
            texts.hints.keys.enter,
            texts.common.query,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 WHOIS 查询工具弹窗
fn render_whois_lookup(frame: &mut Frame, modal: &Modal) {
    let Modal::WhoisLookup {
        domain,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let area = frame.area();
    let modal_area = centered_rect(70, 18, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.whois))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // 域名输入框
    lines.push(Line::from(vec![Span::styled(
        texts.modal.tools.labels.domain,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));

    let mut input_spans = vec![Span::styled("  ", Style::default())];
    input_spans.extend(render_input_with_cursor(
        domain,
        texts.modal.tools.placeholders.enter_domain,
        true, // WHOIS 弹窗只有一个输入框，始终聚焦
    ));
    lines.push(Line::from(input_spans));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(
            texts.modal.tools.status.querying,
            Style::default().fg(Color::Yellow))
        );
    } else if let Some(ref res) = result {
        lines.push(Line::styled(
            format!("{}", texts.modal.tools.result_label),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        );
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}: {} | {}: {} ",
            texts.hints.keys.enter,
            texts.common.query,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 SSL 证书检查工具弹窗
fn render_ssl_check(frame: &mut Frame, modal: &Modal) {
    let Modal::SslCheck {
        domain,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let area = frame.area();
    let modal_area = centered_rect(70, 18, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.ssl_check))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // 域名输入框
    lines.push(Line::from(vec![Span::styled(
        texts.modal.tools.labels.domain,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));

    let mut input_spans = vec![Span::styled("  ", Style::default())];
    input_spans.extend(render_input_with_cursor(
        domain,
        texts.modal.tools.placeholders.enter_domain,
        true,
    ));
    lines.push(Line::from(input_spans));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.checking, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}: {} | {}: {} ",
            texts.hints.keys.enter,
            texts.common.check,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 IP 查询工具弹窗
fn render_ip_lookup(frame: &mut Frame, modal: &Modal) {
    let Modal::IpLookup {
        input,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let area = frame.area();
    let modal_area = centered_rect(70, 18, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.ip_lookup))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // IP 或域名输入框
    lines.push(Line::from(vec![Span::styled(
        texts.modal.tools.labels.ip_or_domain,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));

    let mut input_spans = vec![Span::styled("  ", Style::default())];
    input_spans.extend(render_input_with_cursor(
        input,
        texts.modal.tools.placeholders.enter_ip_or_domain,
        true,
    ));
    lines.push(Line::from(input_spans));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.looking_up, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}: {} | {}: {} ",
            texts.hints.keys.enter,
            texts.common.lookup,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 HTTP 头检查工具弹窗
fn render_http_header_check(frame: &mut Frame, modal: &Modal) {
    let Modal::HttpHeaderCheck {
        url,
        method_index,
        focus,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let methods = ["GET", "HEAD", "POST", "PUT", "DELETE", "OPTIONS"];

    let area = frame.area();
    let modal_area = centered_rect(70, 20, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.http_header))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // URL 输入框
    let url_style = if *focus == 0 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.url, url_style)]));

    let mut url_spans = vec![Span::styled("  ", Style::default())];
    url_spans.extend(render_input_with_cursor(
        url,
        texts.modal.tools.placeholders.enter_url,
        *focus == 0,
    ));
    lines.push(Line::from(url_spans));
    lines.push(Line::from(""));

    // HTTP 方法选择
    let method = methods[*method_index];
    let method_style = if *focus == 1 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.method, method_style)]));
    lines.push(Line::from(vec![
        Span::styled("  < ", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(
            method,
            if *focus == 1 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        Span::styled(" >", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
    ]));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.checking, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}/{}: {} | {}: {} | {}: {} | {}: {} ",
            texts.hints.keys.tab,
            texts.hints.keys.arrows_ud,
            texts.hints.actions.move_up_down,
            texts.hints.keys.arrows_lr,
            texts.hints.actions.change_method,
            texts.hints.keys.enter,
            texts.common.check,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 DNS 传播检查工具弹窗
fn render_dns_propagation(frame: &mut Frame, modal: &Modal) {
    let Modal::DnsPropagation {
        domain,
        record_type_index,
        focus,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let record_types = get_all_record_types();

    let area = frame.area();
    let modal_area = centered_rect(70, 20, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.dns_propagation))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // 域名输入框
    let domain_style = if *focus == 0 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.domain, domain_style)]));

    let mut domain_spans = vec![Span::styled("  ", Style::default())];
    domain_spans.extend(render_input_with_cursor(
        domain,
        texts.modal.tools.placeholders.enter_domain,
        *focus == 0,
    ));
    lines.push(Line::from(domain_spans));
    lines.push(Line::from(""));

    // 记录类型选择
    let record_type = &record_types[*record_type_index];
    let record_style = if *focus == 1 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![Span::styled(texts.modal.tools.labels.record_type, record_style)]));
    lines.push(Line::from(vec![
        Span::styled("  < ", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
        Span::styled(
            record_type.name(),
            if *focus == 1 {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        Span::styled(" >", if *focus == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) }),
    ]));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.checking_propagation, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}/{}: {} | {}: {} | {}: {} | {}: {} ",
            texts.hints.keys.tab,
            texts.hints.keys.arrows_ud,
            texts.hints.actions.navigate,
            texts.hints.keys.arrows_lr,
            texts.hints.actions.change_type,
            texts.hints.keys.enter,
            texts.common.check,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// 渲染 DNSSEC 验证工具弹窗
fn render_dnssec_check(frame: &mut Frame, modal: &Modal) {
    let Modal::DnssecCheck {
        domain,
        result,
        loading,
    } = modal
    else {
        return;
    };

    let texts = t();
    let area = frame.area();
    let modal_area = centered_rect(70, 18, area);

    // 清除弹窗区域
    frame.render_widget(Clear, modal_area);

    // 创建边框
    let block = Block::default()
        .title(format!(" {} ", texts.modal.tools.titles.dnssec))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // 构建弹窗内容
    let mut lines = vec![];

    // 域名输入框
    lines.push(Line::from(vec![Span::styled(
        texts.modal.tools.labels.domain,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));

    let mut input_spans = vec![Span::styled("  ", Style::default())];
    input_spans.extend(render_input_with_cursor(
        domain,
        texts.modal.tools.placeholders.enter_domain,
        true,
    ));
    lines.push(Line::from(input_spans));
    lines.push(Line::from(""));

    // 查询结果
    if *loading {
        lines.push(Line::styled(texts.modal.tools.status.checking_dnssec, Style::default().fg(Color::Yellow)));
    } else if let Some(ref res) = result {
        lines.push(Line::styled(format!("{}", texts.modal.tools.result_label), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        for line in res.lines() {
            lines.push(Line::from(line));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!(" {}: {} | {}: {} ",
            texts.hints.keys.enter,
            texts.common.check,
            texts.hints.keys.esc,
            texts.common.close
        ),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
