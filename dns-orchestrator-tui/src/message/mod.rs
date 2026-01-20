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
//! src/message/mod.rs
//! Message 层：事件消息定义
//!
//! 作为 Event —→ Update 之间的桥梁
//! 所有的用户操作和状态变更都通过 Message 来表达。
//! 相当于将形形色色的 Events 翻译成 Update 能够看懂的 Messages
//! Update 层根据 Message 来更新 Model。
//! 
//! 
//! 有模块结构：
//!     src/message/mod.rs
//!         mod app;
//!         mod modal;
//!         mod content;
//!         mod navigation;
//! 
//!         pub use app::AppMessage;
//!         pub use navigation::NavigationMessage;
//! 
//! 
//!     在 app::AppMessage 中进行主消息的枚举：
//!         #[derive{Debug , Clone}]
//! 
//!         pub enum AppMessage {
//!             Quit,                               // 退出应用
//!             ToggleFocus,                        // 切换焦点面板
//!             Navigation(NavigationMessage),      // 导航面板子消息，与主消息分离
//!             GoBack,                             // 返回上一页
//!             Refresh,                            // 刷新数据
//!             ShowHelp,                           // 显示帮助
//!             ClearStatus,                        // 清除状态栏消息
//!             Noop,                               // 无操作，用于代替 Option::None
//!         }
//! 
//! 
//!     分别分出
//!         content.rs          专门处理在内容面板中的子消息
//!         modal.rs            专门处理弹窗相关的子消息
//!         navigation.rs       专门处理在导航栏中的子消息
//! 
//!     它们都接受 app::AppMessage 的调用。
//! 
//! 
//! 
//!     在 src/event/handler.rs 中，有：
//!         pub fn handle_event(event: Event, app: &App) -> AppMessage {
//!             ...                                          ↑↑↑↑↑↑↑↑↑↑
//!             ...                                          返回一个 AppMessage 类型
//!             match event {
//!                 Event::Key(key)
//!                 if key.code == ... => {
//!                     ...
//!                     return AppMessage::...          // 在此从 message 获取、创建一个枚举值并返回
//!                            ↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑          // 于是 “创建” 了 一条消息
//!                 }
//!                 _ => AppMessage::Noop
//!             }
//!         }
//! 
//! 
//! 最后，Event 将从 Message 处获取的消息传入 Update 层进行处理。
//!     —— 去往 src/update/mod.rs 吧
//! 

mod app;
mod modal;
mod content;
mod navigation;

pub use app::AppMessage;
pub use content::ContentMessage;
pub use modal::ModalMessage;
pub use navigation::NavigationMessage;