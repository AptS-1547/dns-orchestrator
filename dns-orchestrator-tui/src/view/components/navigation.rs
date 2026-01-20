//! 左侧导航面板组件

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::i18n::t;
use crate::model::NavItemId;
use crate::model::App;
use crate::view::theme::Styles;

/// 渲染导航面板
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let is_focused = app.focus.is_navigation();

    // 边框样式
    let border_style = if is_focused {
        Styles::border_focused()
    } else {
        Styles::border()
    };

    let block = Block::default()
        .title(format!(" {} ", texts.nav.home))
        .title_style(Styles::title())
        .borders(Borders::ALL)
        .border_style(border_style);

    // 构建导航项列表
    let items: Vec<ListItem> = app
        .navigation
        .items
        .iter()
        .enumerate()
        .map(|(i, nav_item)| {
            let is_selected = i == app.navigation.selected;
            let prefix = if is_selected { "▶ " } else { "  " };

            // 根据导航项 ID 获取翻译后的标签
            let label = match nav_item.id {
                NavItemId::Home => texts.nav.home,
                NavItemId::Domains => texts.nav.domains,
                NavItemId::Accounts => texts.nav.accounts,
                NavItemId::Toolbox => texts.nav.toolbox,
                NavItemId::Settings => texts.nav.settings,
            };

            let content = format!("{}{} {}", prefix, nav_item.icon, label);

            let style = if is_selected {
                Styles::selected()
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Styles::selected());

    // 使用 ListState 来跟踪选中状态
    let mut state = ListState::default();
    state.select(Some(app.navigation.selected));

    frame.render_stateful_widget(list, area, &mut state);
}