//！┌─────────────────────────────────────────────────────────────────────────┐
//！│                           主循环 (app.rs)                                │
//！│                                                                         │
//！│    ┌─────────┐     ┌─────────┐     ┌──────────┐     ┌─────────┐         │
//！│    │ 用户按键 │ ─▶ │  Event  │ ─▶ │ Message  │ ──▶ │ Update  │         │
//！│    └─────────┘     │  层     │     │   层     │     │   层    │          │
//！│         ▲          └─────────┘     └──────────┘     └────┬────┘         │
//！│         │                                                │              │
//！│         │          ┌─────────┐     ┌──────────┐          ▼              │
//！│         │          │  Util   │     │  Model   │ ◀───────────           │
//！│         │          │  层     │     │   层     │                         │
//！│         │          └─────────┘     └────┬─────┘                         │
//！│         │                               │                               │
//！│         │          ┌─────────┐          ▼                               │
//！│         └──────────│  View   │ ◀── 读取状态                             │
//！│           屏幕输出  │   层    │                                          │
//！│                    └─────────┘                                          │
//！└─────────────────────────────────────────────────────────────────────────┘


//!
//! src/util/mod.rs
//! Util 层：基础设施和工具函数
//!
//! Util 层提供与业务逻辑无关的基础设施代码，
//! 主要负责终端的初始化和恢复。
//!
//!
//! 有模块结构：
//!     src/util/mod.rs
//!         mod terminal;       // 终端初始化和恢复
//!
//!         pub use terminal::{init_terminal, restore_terminal, Term};
//!
//!
//!     终端类型定义：
//!         在 src/util/terminal.rs 中，有：
//!
//!             // 类型别名，简化长类型名
//!             pub type Term = Terminal<CrosstermBackend<Stdout>>;
//!
//!         这样在其他地方就可以使用 Term 而不是完整的类型名：
//!
//!             // 不用别名
//!             fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, ...) { }
//!
//!             // 使用别名
//!             fn run(terminal: &mut Term, ...) { }
//!
//!
//!     初始化终端：
//!         在 src/util/terminal.rs 中，有：
//!
//!             pub fn init_terminal() -> Result<Term> {
//!                 enable_raw_mode()?;                     // 1. 启用原始模式
//!                 let mut stdout = io::stdout();
//!                 execute!(stdout, EnterAlternateScreen)?;   // 2. 进入备用屏幕
//!
//!                 let backend = CrosstermBackend::new(stdout);
//!                 let terminal = Terminal::new(backend)?;    // 3. 创建终端对象
//!
//!                 Ok(terminal)
//!             }
//!
//!         关键概念：
//!
//!         · Raw Mode（原始模式）
//!             - 关闭行缓冲：无需按 Enter，每个按键立即生效
//!             - 关闭字符回显：按键不会显示在终端上
//!             - 捕获所有按键：包括 Ctrl+C、箭头键等特殊键
//!
//!         · Alternate Screen（备用屏幕）
//!             - 终端有两个缓冲区：主屏幕和备用屏幕
//!             - TUI 应用在备用屏幕运行
//!             - 退出后自动恢复主屏幕内容（不会覆盖原有内容）
//!             - 类似 vim、htop 等工具的行为
//!
//!
//!     恢复终端：
//!         在 src/util/terminal.rs 中，有：
//!
//!             pub fn restore_terminal(terminal: &mut Term) -> Result<()> {
//!                 disable_raw_mode()?;                       // 1. 禁用原始模式
//!                 execute!(terminal.backend_mut(), LeaveAlternateScreen)?;  // 2. 离开备用屏幕
//!                 terminal.show_cursor()?;                   // 3. 显示光标
//!                 Ok(())
//!             }
//!
//!         注意：无论程序是正常退出还是发生错误，都必须调用此函数！
//!               否则终端会保持在原始模式，用户输入不会正常显示。
//!
//!
//!     使用方式：
//!         在 src/main.rs 中，有：
//!
//!             fn main() -> Result<(), anyhow::Error> {
//!                 // 1. 初始化终端
//!                 let mut terminal = init_terminal()?;
//!
//!                 // 2. 创建应用实例
//!                 let mut app = model::App::new();
//!
//!                 // 3. 运行主循环
//!                 let result = app::run(&mut terminal, &mut app);
//!
//!                 // 4. 恢复终端（无论成功失败都执行）
//!                 restore_terminal(&mut terminal)?;
//!
//!                 // 5. 返回结果
//!                 return result;
//!             }
//!
//!         关键：第 4 步在 app::run 之后立即执行，即使 run 返回错误，
//!               也会先恢复终端，再返回错误结果。
//!
//!
//! Util 层在应用启动时初始化终端，在应用退出时恢复终端。
//! 主循环在初始化后的终端中运行。
//!     —— 去往 src/app.rs 主循环吧
//!

mod terminal;

pub use terminal::{init_terminal, restore_terminal, Term};
