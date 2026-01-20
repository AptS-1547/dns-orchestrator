//! 
//! app.rs
//! 应用主循环
//! 
//! 
//! 
//! 在应用启动时，创建终端并初始化为以下状态：
//! 
//! App {
//! 
//!     should_quit: bool = false,                      // 决定应用是否应该退出
//!     focus: FocusPanel::Navigation,                  // 当前焦点在哪个面板
//!     navigation: NavigationState{
//!         items: [Home , Domains , Accounts , Toolbox , Settings],
//!         selected = 0                                    // 当前选中第几项，默认为 0
//!     },
//!     current_page = Page::Home,                      // 当前应该显示哪个页面，默认为 Home
//!     status_message = None,                          // 状态栏消息
//! 
//! }
//! 
//! 
//! 主循环大约每 100 ms 执行一次（取决于有无事件）
//! 应用的主循环中有：
//! loop {
//! 
//!     terminal.draw(|f| view::render(&app , f))       // 渲染 UI
//!     if app.should_quit{ break }                     // 检查 APP 是否应该退出
//!     if let Some(event) = poll_event() {             // 轮询获取输入，在此等待 100ms
//!                                                     // 若用户按键，返回 Some(Event::Key(...))，否则为 None
//!         let msg = handle_event(event , &app);           // 接收原始事件并分发消息
//!         update::update(&mut app , msg)                  // 更新终端状态
//!     }
//! }

use std::time::Duration;

use anyhow::Result;

use crate::event;
use crate::model::App;
use crate::update;
use crate::util::Term;
use crate::view;

/// 运行应用主循环
pub fn run(terminal: &mut Term, app: &mut App) -> Result<()> {
    loop {
        // 1. 渲染 UI
        terminal.draw(|frame| {
            view::render(app, frame);
        })?;

        // 2. 检查是否应该退出
        if app.should_quit {
            break;
        }

        // 3. 轮询事件（100ms 超时）
        if let Some(event) = event::poll_event(Duration::from_millis(100))? {
            // 4. 处理事件，获取消息
            let msg = event::handle_event(event, app);

            // 5. 更新状态
            update::update(app, msg);
        }
    }

    Ok(())
}