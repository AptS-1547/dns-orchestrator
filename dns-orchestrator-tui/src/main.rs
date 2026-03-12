#![allow(dead_code)] // TUI 处于开发阶段，许多模块尚未接入

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
//! fn `main()` {
//!
//!     Runtime::new()              // 创建 Tokio 异步运行时
//!     CoreService::new()          // 创建后端核心服务
//!     core_service.initialize()   // 异步初始化（恢复账号等）
//!     init_terminal()             // 初始化终端，TerminalGuard 在 drop 时自动恢复
//!     model::App::new(...)        // 创建 APP 实例（注入 CoreService 和 Handle）
//!     app::run()                  // 运行 app.rs 主循环
//!
//! }
//!
//! Runtime 在 main 栈帧上存活，事件循环不在 `block_on` 内运行，
//! 因此 Update 层可安全使用 `Handle::block_on` 调用 async 后端方法。

mod app;
mod backend;
mod event;
pub mod i18n;
mod message;
mod model;
mod update;
mod util;
mod view;

use std::sync::Arc;

use anyhow::Result;

use util::{TerminalGuard, init_terminal};

fn main() -> Result<(), anyhow::Error> {
    // 1. 创建异步运行时和后端服务
    let rt = tokio::runtime::Runtime::new()?;
    let core_service = Arc::new(backend::CoreService::new());

    // 2. 异步初始化（恢复账号等），不静默吞掉错误
    if let Err(e) = rt.block_on(core_service.initialize()) {
        eprintln!("Warning: initialization failed: {e}");
    }

    // 3. 初始化终端，TerminalGuard 在 drop 时自动恢复
    let mut guard = TerminalGuard(init_terminal()?);

    // 4. 创建应用实例（注入 CoreService 和 Runtime Handle）
    let mut app = model::App::new(core_service, rt.handle().clone());

    // 5. 运行主循环并返回结果
    app::run(&mut guard.0, &mut app)
}
