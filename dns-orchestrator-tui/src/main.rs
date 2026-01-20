//! DNS Orchestrator TUI
//!
//! ## 架构
//!
//! 采用 Elm Architecture (TEA) 模式：
//! - **Model**: 应用状态 (`model/`)
//! - **Message**: 事件消息 (`message/`)
//! - **Update**: 状态更新 (`update/`)
//! - **View**: UI 渲染 (`view/`)
//! - **Event**: 输入处理 (`event/`)
//! - **Backend**: 业务服务 (`backend/`)
//! 
//! 
//! main.rs
//! DNS Orchestrator TUI 的程序入口
//! 
//! 其执行：
//! fn main() {
//! 
//!     init_terminal()         // 首先初始化终端，以为 terminal: Terminal<...>
//!     model::App:new()        // 创建 APP 实例
//!     app::run()              // 运行 app.rs 主循环
//!     restore_terminal()      // 无论成功与否，都恢复终端
//! 
//! }
//! 
//! 
//! 
//! 当启动程序时，main.rs：
//!     init_terminal()         // from util/terminal.rs
//! 
//!     有：
//!         · enable_raw_mode()
//!             - 以关闭终端行缓冲模式、关闭回显与允许读取单个按键事件
//!         · execute!(io::stdout , EnterAlternateScreen)?
//!             - 切换到 备用屏幕
//!         · 返回 Terminal 对象
//! 
//! 
//!     App:new()               // from model/app.rs
//!     创建终端初始状态（在 /app.rs 下细嗦）
//! 
//! 
//!     进入主循环 app::run()   // from /app.rs




mod app;
mod backend;
mod event;
pub mod i18n;
mod message;
mod model;
mod update;
mod util;
mod view;

use anyhow::Result;

use util::{init_terminal, restore_terminal};

fn main() -> Result<() , anyhow::Error> {
    // 1. 初始化终端
    let mut terminal = init_terminal()?;

    // 2. 创建应用实例
    let mut app = model::App::new();

    // 3. 运行主循环
    let result = app::run(&mut terminal, &mut app);

    // 4. 恢复终端（无论成功失败都执行）
    restore_terminal(&mut terminal)?;

    // 5. 返回结果
    return result;
}