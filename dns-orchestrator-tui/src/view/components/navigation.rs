//! 左侧导航面板组件

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::i18n::t;
use crate::model::App;
use crate::model::NavItemId;
use crate::view::theme::colors;

/// 渲染导航面板
#[allow(clippy::cast_possible_truncation)]
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let texts = t();
    let c = colors();
    let is_focused = app.focus.is_navigation();
    let selected = app.navigation.selected;

    // 边框样式
    let border_style = if is_focused {
        Style::default().fg(c.border_focused)
    } else {
        Style::default().fg(c.border)
    };

    let block = Block::default()
        .title(format!(" {} ", texts.nav.home))
        .title_style(Style::default().fg(c.fg).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    // 先获取内部区域，再渲染边框（模式同 layout.rs:100-101）
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let highlight_style = Style::default()
        .bg(c.selected_bg)
        .fg(c.selected_fg)
        .add_modifier(Modifier::BOLD);

    // 构建单个 ListItem 的闭包
    let build_item = |global_index: usize, id: NavItemId, icon: &str| {
        let is_selected = global_index == selected;
        let prefix = if is_selected { "▶ " } else { "  " };

        let label = match id {
            NavItemId::Home => texts.nav.home,
            NavItemId::Domains => texts.nav.domains,
            NavItemId::Favorites => texts.nav.favorites,
            NavItemId::Accounts => texts.nav.accounts,
            NavItemId::Toolbox => texts.nav.toolbox,
            NavItemId::Settings => texts.nav.settings,
        };

        let content = format!("{prefix}{icon} {label}");

        let style = if is_selected {
            Style::default()
                .bg(c.selected_bg)
                .fg(c.selected_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(c.fg)
        };

        ListItem::new(Line::from(Span::styled(content, style)))
    };

    let main_count = app.navigation.main_items_count();
    let total = app.navigation.items.len();

    // 小终端降级：内部区域不够高时，退回为单个连续列表（同原始行为）
    if inner.height < total as u16 {
        let all_items: Vec<ListItem> = app
            .navigation
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| build_item(i, item.id, item.icon))
            .collect();

        let list = List::new(all_items).highlight_style(highlight_style);

        let mut state = ListState::default();
        state.select(Some(selected));

        frame.render_stateful_widget(list, inner, &mut state);
        return;
    }

    // 三段或四段布局：根据高度决定是否显示分隔线
    let bottom_count = total - main_count;
    let min_height_for_separator = (main_count + bottom_count + 1) as u16; // 需要额外 1 行给分隔线

    let (chunks, show_separator) = if inner.height >= min_height_for_separator {
        // 高度足够：四段布局（主列表 + 空白 + 分隔线 + 底部）
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(main_count as u16),
                Constraint::Min(0),
                Constraint::Length(1),                   // 分隔线
                Constraint::Length(bottom_count as u16),
            ])
            .split(inner);
        (chunks, true)
    } else {
        // 高度不足：三段布局（主列表 + 空白 + 底部），无分隔线
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(main_count as u16),
                Constraint::Min(0),
                Constraint::Length(bottom_count as u16),
            ])
            .split(inner);
        (chunks, false)
    };

    // 上部：主导航项
    let main_items: Vec<ListItem> = app
        .navigation
        .items[..main_count]
        .iter()
        .enumerate()
        .map(|(i, item)| build_item(i, item.id, item.icon))
        .collect();

    let main_list = List::new(main_items).highlight_style(highlight_style);

    let mut top_state = ListState::default();
    if selected < main_count {
        top_state.select(Some(selected));
    }
    frame.render_stateful_widget(main_list, chunks[0], &mut top_state);

    // 分隔线：仅在高度足够时显示
    if show_separator {
        let sep_width = chunks[2].width.saturating_sub(4).max(3); // 减去左右边距，最少 3 个字符
        let separator_line = "─".repeat(sep_width as usize);
        let separator = Paragraph::new(separator_line)
            .style(Style::default().fg(c.muted))
            .alignment(Alignment::Center);
        frame.render_widget(separator, chunks[2]);
    }

    // 下部：底部固定项（Toolbox / Settings）
    let bottom_chunk_index = if show_separator { 3 } else { 2 };
    let bottom_items: Vec<ListItem> = app
        .navigation
        .items[main_count..]
        .iter()
        .enumerate()
        .map(|(i, item)| build_item(main_count + i, item.id, item.icon))
        .collect();

    let bottom_list = List::new(bottom_items).highlight_style(highlight_style);

    let mut bottom_state = ListState::default();
    if selected >= main_count {
        bottom_state.select(Some(selected - main_count));
    }
    frame.render_stateful_widget(bottom_list, chunks[bottom_chunk_index], &mut bottom_state);
}
