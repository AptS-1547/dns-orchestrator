//! 导航更新逻辑

use crate::message::NavigationMessage;
use crate::model::{App, NavItemId, Page};

/// 处理导航消息
pub fn update(app: &mut App, msg: NavigationMessage) {
    match msg {
        NavigationMessage::SelectPrevious => {
            app.navigation.select_previous();
        }

        NavigationMessage::SelectNext => {
            app.navigation.select_next();
        }

        NavigationMessage::Confirm => {
            if let Some(id) = app.navigation.current_id() {
                app.current_page = page_from_nav_id(id);
                app.clear_status(); // 切换页面时清除状态消息
            }
        }

        NavigationMessage::SelectFirst => {
            app.navigation.selected = 0;
        }

        NavigationMessage::SelectLast => {
            let len = app.navigation.items.len();
            if len > 0 {
                app.navigation.selected = len - 1;
            }
        }
    }
}

/// 根据导航项 ID 获取对应的页面
fn page_from_nav_id(id: NavItemId) -> Page {
    match id {
        NavItemId::Home => Page::Home,
        NavItemId::Domains => Page::Domains,
        NavItemId::Accounts => Page::Accounts,
        NavItemId::Toolbox => Page::Toolbox,
        NavItemId::Settings => Page::Settings,
    }
}