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
//! src/event/mod.rs
//! Event 层：事件处理
//!
//! 负责将键盘/鼠标等输入事件转换为 Message。
//! 
//! 
//! 有模块结构：
//!     src/event/mod.rs
//!         mod handler;        // 事件处理器
//!         mod keymap;         // 快捷键映射
//!
//!         pub use handler::{handle_event , poll_event}; 
//! 
//! 
//!     其中有：
//!         · poll_event      事件轮询，受 ~/app.rs 调用
//!             
//!         pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
//! 
//!             if event::poll(timeout)? {                  // 此处阻塞以等待事件，最长等待 timeout
//!                 Ok(Some(event::read()?))
//!             } else {
//!                 Ok(None)
//!             }
//!         }
//! 
//! 
//!         · handle_event    事件分发
//! 
//!         接收以下 Event 类型：
//!             Event::Key(KeyEvent)                // 键盘事件，发至以下几个函数处理
//!             Event::Resize(Width , height)       // 终端窗口大小发生变化，重绘终端
//!             Event::Mouse(MouseEvent)            // 鼠标事件（暂不处理）
//!
//!             当接收到键盘事件时，转入 handle_key_events()
//!             判断：
//!                 - 有弹窗打开时，调用 handle_modal_keys 处理
//!                 - 全局快捷键，就地处理；
//!                 - 焦点位于导航面板，调用 handle_navigation_keys 处理
//!                 - 焦点位于内容面板，调用 handle_content_keys 处理
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 弹窗键盘处理
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/event/handler.rs 中定义：
//!
//!         当 app.modal.is_open() 为 true 时，优先处理弹窗键盘事件。
//!         根据弹窗类型分发到具体的处理函数：
//!             - handle_add_account_keys()     添加账号弹窗
//!             - handle_dns_lookup_keys()      DNS 查询工具
//!             - handle_simple_tool_keys()     简单工具（WHOIS、SSL 等）
//!             - ... 其他弹窗
//!
//!         常用键盘映射：
//!             Esc         → ModalMessage::Close
//!             Tab         → ModalMessage::NextField
//!             Shift+Tab   → ModalMessage::PrevField
//!             Enter       → ModalMessage::Confirm
//!             ←/→         → ModalMessage::PrevProvider / NextProvider
//!             字符输入     → ModalMessage::Input(c)
//!             Backspace   → ModalMessage::Backspace
//! 
//! 
//!     在 src/event/handler.rs 中，有：
//!         pub fn handle_event(event: Event , app: &App) -> AppMessage {
//!             ...                                          ↑↑↑↑↑↑↑↑↑↑
//!             ...                                          返回一个 AppMessage 类型
//!             match event {
//!                 Event::Key(key)
//!                 if key.code == ... => {
//!                     ...
//!                     return AppMessage::...          // 在此创建一个 AppMessage 枚举值并返回
//!                            ↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑          // 于是 "创建" 了 一条消息
//!                 }
//!                 _ => AppMessage::Noop
//!             }
//!         }
//!
//!     即，handler.rs 使用 message 层定义的 AppMessage 枚举类型，
//!     创建一个对应的枚举值并返回。
//!     在 src/app.rs 中，有：
//!         update::update(app , msg);
//!                              ↑↑↑                    // 在此作为参数传入 update 层
//!     于是在 src/update/mod.rs 中：
//!         pub fn update(app: &mut App , msg: AppMessage) {
//!                                       ↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑
//!                                       在此接收参数，执行对应操作
//!             match msg{...}
//!         }
//! 
//! 

mod handler;
mod keymap;

pub use handler::{handle_event, poll_event};