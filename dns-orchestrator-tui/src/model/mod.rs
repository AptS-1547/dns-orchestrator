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
//! src/model/mod.rs
//! Model 层：应用状态定义
//!
//! Model 层是应用状态的 “唯一真相来源”。
//! 这一层只包含纯数据结构，不包含任何业务逻辑。
//! 所有状态变更都通过 Update 层来触发。
//!
//!
//! 有模块结构：
//!     src/model/mod.rs
//!         mod app;            // 主应用状态
//!         mod focus;          // 焦点状态（Navigation / Content）
//!         mod navigation;     // 导航栏状态
//!         mod page;           // 页面路由状态
//!
//!         pub mod domain;     // 领域模型（域名、DNS 记录等）
//!         pub mod state;      // 页面数据状态
//! 
//!     值得一提的是，虽说 page.rs 与 state/ 都表示页面状态，但两者有不同：
//!         - Page 是一个简单的枚举，表示当前应用处于哪个“页面”，相当于房间的门牌号，
//!             只负责标识位置，不存储任何业务数据；
//!         - State 是各个页面的业务数据容器，存储着列表、选中项、加载状态等，
//!             相当于储存了房间的内容。
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 一、主应用状态（App）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/model/app.rs 中定义：
//!
//!         pub struct App {
//!             pub should_quit: bool,              // 退出标志
//!             pub focus: FocusPanel,              // 当前焦点
//!             pub navigation: NavigationState,    // 导航状态
//!             pub current_page: Page,             // 当前页面
//!             pub status_message: Option<String>, // 状态栏消息（可选）
//! 
//!             // 以及各页面状态：
//!             pub accounts: AccountsState,        // 账号页面状态
//!             pub domains: DomainsState,          // 域名页面状态
//!             pub dns_records: DnsRecordsState,   // DNS 记录 页面状态
//!             pub toolbox: ToolboxState,          // 工具箱页面状态
//! 
//!             pub modal: ModalState               // 弹窗状态
//!         }
//!
//!     使用：
//!         - 在 main.rs 中创建：let mut app = model::App::new();
//!         - 在 update/mod.rs 中修改：app.should_quit = true;
//!         - 在 view/mod.rs 中读取：pub fn render(app: &App, ...)
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 二、焦点管理（FocusPanel）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/model/focus.rs 中定义焦点面板枚举：
//!         - Navigation：左侧导航面板
//!         - Content：右侧内容面板
//!
//!     核心方法：
//!         - toggle()：切换焦点（左 ↔ 右）
//!         - is_navigation()：判断焦点是否在导航栏
//!         - is_content()：判断焦点是否在内容区
//!
//!     数据流：
//!         用户按 ← 或 → 键
//!             ↓
//!         event/handler.rs 返回 AppMessage::ToggleFocus
//!             ↓
//!         update/mod.rs 执行 app.focus = app.focus.toggle()
//!             ↓
//!         view 层根据 app.focus.is_navigation() 设置边框颜色
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 三、导航状态（NavigationState）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/model/navigation.rs 中定义：
//!
//!         NavigationState {
//!             items: Vec<NavItem>,    // 导航项列表（Home, Domains, ...）
//!             selected: usize,        // 当前选中项的索引
//!         }
//!
//!         NavItem {
//!             id: NavItemId,          // 唯一标识
//!             label: &'static str,    // 显示文本
//!             icon: &'static str,     // 图标
//!         }
//!
//!     核心方法：
//!         - select_previous()：向上移动
//!         - select_next()：向下移动
//!         - current_id()：获取当前选中项的 ID
//!
//!     数据流：
//!         用户按 ↑ 或 ↓ 键（焦点在导航栏时）
//!             ↓
//!         event/handler.rs 返回 Navigation(SelectNext/SelectPrevious)
//!             ↓
//!         update/navigation.rs 调用 app.navigation.select_next()
//!             ↓
//!         view/components/navigation.rs 根据 selected 高亮对应项
//!
//!         用户按 Enter 确认
//!             ↓
//!         update/navigation.rs 根据 current_id() 切换页面
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 四、页面状态（Page）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/model/page.rs 中定义页面枚举：
//!         - Home, Domains, Accounts, Toolbox, Settings（列表页）
//!         - DnsRecords { account_id, domain_id }（携带数据的详情页）
//!
//!     核心方法：
//!         - title()：返回页面标题
//!         - is_detail_page()：判断是否是详情页
//!
//!     数据流：
//!         在导航栏按 Enter → 切换到对应的列表页
//!         在列表页按 Enter → 进入详情页（携带选中项的 ID）
//!         在详情页按 Esc → 返回列表页
//!
//!         view/layout.rs 根据 app.current_page 匹配并渲染对应页面
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 五、弹窗状态（ModalState）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/model/state/modal.rs 中定义：
//!
//!         Modal 枚举：每种弹窗都是一个变体，携带该弹窗的所有数据
//!             - AddAccount { provider_index, name, credential_values, focus, ... }
//!             - ConfirmDelete { item_type, item_name, item_id, focus }
//!             - DnsLookup { domain, record_type_index, result, loading, ... }
//!             - Help, Error { title, message }
//!             - ... 其他工具弹窗
//!
//!         ModalState 容器：管理当前活动的弹窗
//!             - active: Option<Modal>    // None = 无弹窗, Some = 有弹窗
//!             - show_xxx() 方法：初始化并显示特定弹窗
//!             - close() 方法：关闭弹窗
//!
//!     数据流：
//!         用户按 Alt+A（在账号页）
//!             ↓
//!         event/handler.rs 调用 app.modal.show_add_account()
//!             ↓
//!         ModalState.active = Some(Modal::AddAccount { ... })
//!             ↓
//!         view/components/modal.rs 检测到弹窗，渲染弹窗 UI
//!
//!
//! Model 层的数据被 Update 层修改，然后被 View 层读取并渲染成 UI。
//!

mod app;
mod focus;
mod navigation;
mod page;
pub mod state;

pub mod domain;

pub use app::App;
pub use focus::FocusPanel;
pub use navigation::{NavItem, NavItemId, NavigationState};
pub use page::Page;
pub use state::{
    AccountsState, DnsRecordsState, DomainsState, Modal, ModalState, SettingsState, ToolboxState, ToolboxTab,
};
