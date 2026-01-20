//! 底部状态栏组件

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::i18n::t;
use crate::model::{App, FocusPanel, Page};
use crate::view::theme::Styles;

/// 渲染状态栏
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // 根据当前焦点和页面生成快捷键提示
    let hints = get_hints(app);

    // 构建状态栏内容
    let mut spans = Vec::new();

    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(key.to_string(), Styles::hint_key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(desc.to_string(), Styles::hint_desc()));
    }

    // 如果有状态消息，显示在右侧
    if let Some(ref msg) = app.status_message {
        // Add分隔符
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(msg.clone(), Style::default().fg(Color::Yellow)));
    }

    let content = Line::from(spans);
    let paragraph = Paragraph::new(content).style(Styles::statusbar());

    frame.render_widget(paragraph, area);
}

/// 根据当前状态生成快捷键提示
fn get_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    let texts = t();
    let mut hints = Vec::new();

    // 全局快捷键 - 改为 Tab 切换面板
    hints.push((texts.hints.keys.tab, texts.hints.actions.switch_panel));

    // 根据焦点位置显示不同的快捷键
    match app.focus {
        FocusPanel::Navigation => {
            hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
            hints.push((texts.hints.keys.enter, texts.common.confirm));
        }
        FocusPanel::Content => {
            match &app.current_page {
                Page::Home => {
                    hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
                }
                Page::Accounts => {
                    hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
                    hints.push(("Alt+a", texts.common.add));
                    hints.push(("Alt+d", texts.common.delete));
                }
                Page::Domains => {
                    hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
                    hints.push((texts.hints.keys.enter, texts.common.confirm));
                }
                Page::DnsRecords { .. } => {
                    hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
                    hints.push(("Alt+a", texts.common.add));
                    hints.push((texts.hints.keys.esc, texts.common.back));
                }
                Page::Toolbox => {
                    hints.push((texts.hints.keys.arrows_lr, texts.hints.actions.switch_option));
                    hints.push((texts.hints.keys.enter, texts.common.confirm));
                }
                Page::Settings => {
                    hints.push((texts.hints.keys.arrows_ud, texts.hints.actions.move_up_down));
                    hints.push((texts.hints.keys.arrows_lr, texts.hints.actions.switch_option));
                }
            }
        }
    }

    // Quit
    hints.push(("Alt+q", texts.common.quit));

    hints
}