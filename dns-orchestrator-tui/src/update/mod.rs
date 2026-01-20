//！┌─────────────────────────────────────────────────────────────────────────────┐
//！│                              主循环 (app.rs)                               │
//！│                                                                            │
//！│  ┌────────────────────────────── UI 层 ───────────────────────────────┐   │
//！│  │                                                                     │   │
//！│  │   ┌─────────┐          ┌───────────┐          ┌──────────┐         │   │
//！│  │   │  Event  │ ───────▶ │  Message  │ ───────▶ │  Update  │         │   │
//！│  │   │   层    │   翻译    │    层     │   消费    │    层    │         │   │
//！│  │   └─────────┘          │           │          └────┬─────┘         │   │
//！│  │        ▲               │ AppMessage│               │ 修改          │   │
//！│  │        │               │ ModalMsg  │               ▼               │   │
//！│  │   ┌─────────┐          │ ContentMsg│          ┌──────────┐         │   │
//！│  │   │  View   │          │ NavMsg    │   ┌───── │  Model   │         │   │
//！│  │   │   层    │          └───────────┘   │      │    层    │         │   │
//！│  │   └────┬────┘ ◀──────── 读取 ──────────┘      └────┬─────┘         │   │
//！│  │        │                                           │               │   │
//！│  └────────│───────────────────────────────────────────│───────────────┘   │
//！│           │                                           │ 异步调用          │
//！│           ▼                                           ▼                   │
//！│      ┌─────────┐                                ┌──────────┐              │
//！│      │  终端   │                                │ Backend  │              │
//！│      │ (Util)  │                                │    层    │              │
//！│      └─────────┘                                └────┬─────┘              │
//！│                                                      │                    │
//！│                                                      ▼                    │
//！│                                           ┌───────────────────┐           │
//！│                                           │dns-orchestrator-  │           │
//！│                                           │      core         │           │
//！│                                           └───────────────────┘           │
//！└─────────────────────────────────────────────────────────────────────────────┘


//!
//! src/update/mod.rs
//! Update 层：状态更新逻辑
//!
//! Update 层负责处理 Message，更新 Model 状态。
//! 是唯一可以修改 Model 的地方。
//!
//!
//! 有模块结构：
//!     src/update/mod.rs
//!         mod navigation;         // 导航子消息处理
//!         mod content;            // 内容面板子消息处理
//!         mod modal;              // 弹窗子消息处理
//!
//!         use crate::message::{AppMessage , NavigationMessage};
//!         use crate::model::{App , NavItemId , Page};
//!
//!         pub fn update(app: &mut App , msg: AppMessage) {...}
//!
//!
//!         有：
//!             pub fn update(app: &mut App, msg: AppMessage) {
//!                 match msg {
//!                     AppMessage::Quit => {
//!                         app.should_quit = true;
//!                     }
//!                     AppMessage::Navigation(nav_msg) => {
//!                         navigation::update(app, nav_msg);
//!                     }
//!                     AppMessage::Content(content_msg) => {
//!                         content::update(app, content_msg);
//!                     }
//!                     AppMessage::Modal(modal_msg) => {
//!                         modal::update(app, modal_msg);
//!                     }
//!                     ...
//!                 }
//!             }
//!
//!         —— 的主更新函数。
//!             使用 match 进行穷举，其中每个 Message 变体都对应一个状态变更。
//!             复杂的子消息委托给子模块处理（navigation、content、modal）。
//!             通过 &mut App 直接修改状态，避免不必要的复制。
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 弹窗更新（modal.rs）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/update/modal.rs 中定义：
//!
//!         根据当前弹窗类型分发到具体的处理函数：
//!             - handle_add_account()      处理添加账号弹窗
//!             - handle_confirm_delete()   处理确认删除弹窗
//!             - handle_dns_lookup()       处理 DNS 查询工具
//!             - handle_whois_lookup()     处理 WHOIS 查询工具
//!             - ... 其他工具弹窗
//!
//!         主要处理的消息：
//!             - ModalMessage::Close       关闭弹窗
//!             - ModalMessage::NextField   切换到下一个输入字段
//!             - ModalMessage::Input(c)    输入字符
//!             - ModalMessage::Confirm     确认/执行操作
//!
//!         在 Confirm 时，可能会调用 Backend 层执行实际操作。
//!
//!
//! Update 完成后，控制权返回主循环（app.rs）。
//! 下一轮循环时，View 层会读取更新后的 Model 来重新渲染。
//! 




mod content;
mod modal;
mod navigation;

use crate::message::AppMessage;
use crate::model::{App, NavItemId, Page};




/// 处理应用消息，更新状态
pub fn update(app: &mut App, msg: AppMessage) {
    match msg {
        AppMessage::Quit => {
            app.should_quit = true;
        }

        AppMessage::ToggleFocus => {
            // 如果有弹窗打开，不切换焦点
            if !app.modal.is_open() {
                app.focus = app.focus.toggle();
            }
        }

        AppMessage::Navigation(nav_msg) => {
            navigation::update(app, nav_msg);
        }

        AppMessage::Content(content_msg) => {
            content::update(app, content_msg);
        }

        AppMessage::Modal(modal_msg) => {
            modal::update(app, modal_msg);
        }

        AppMessage::GoBack => {
            // 如果有弹窗打开，先关闭弹窗
            if app.modal.is_open() {
                app.modal.close();
                app.clear_status();
            } else if app.current_page.is_detail_page() {
                // 如果在详情页，返回列表页
                app.current_page = Page::Domains;
                app.clear_status();
            }
        }

        AppMessage::Refresh => {
            app.set_status("Refreshing...");
            // TODO: 触发数据刷新
        }

        AppMessage::ShowHelp => {
            // 显示帮助弹窗
            app.modal.show_help();
        }

        AppMessage::ClearStatus => {
            app.clear_status();
        }

        AppMessage::Noop => {}
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

pub use navigation::update as update_navigation;