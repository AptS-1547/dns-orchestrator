# DNS Orchestrator TUI - 设计文档

## 一、整体架构

### 1.1 布局结构

```
┌──────────────────────────────────────────────────────────────┐
│ 标题栏: DNS Orchestrator TUI v1.0           [主题] [连接状态] │
├─────────────┬────────────────────────────────────────────────┤
│             │                                                │
│  导航面板    │              主内容区                           │
│  (20%)      │              (80%)                            │
│             │                                                │
│ ● Home      │  动态内容区域：                                 │
│   Domains   │  - 根据左侧导航项切换显示                        │
│   Accounts  │  - 支持多层级导航（如域名→DNS记录）              │
│   Toolbox   │  - 支持表格、列表、表单等多种组件                │
│   Settings  │                                                │
│             │                                                │
├─────────────┴────────────────────────────────────────────────┤
│ 状态栏: 快捷键提示 | 状态消息 | 错误提示                        │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 模块组织 (Elm Architecture)

采用 Elm Architecture (TEA) 模式，实现严格的分层：

```
src/
├── main.rs                  # 入口点（最小化，只做启动和清理）
├── app.rs                   # 应用主循环（协调各层）
│
├── model/                   # ═══ Model 层：纯数据状态 ═══
│   ├── mod.rs
│   ├── app.rs               # App 主状态结构
│   ├── focus.rs             # 焦点状态
│   ├── navigation.rs        # 导航状态
│   ├── page.rs              # 页面枚举和状态
│   │
│   └── domain/              # 业务领域模型（与 UI 无关）
│       ├── mod.rs
│       ├── account.rs       # Account 数据模型
│       ├── domain.rs        # Domain 数据模型
│       └── dns_record.rs    # DnsRecord 数据模型
│
├── message/                 # ═══ Message 层：事件定义 ═══
│   ├── mod.rs
│   ├── app.rs               # AppMessage 主消息枚举
│   ├── navigation.rs        # 导航相关消息
│   ├── accounts.rs          # 账号页面消息
│   ├── domains.rs           # 域名页面消息
│   └── toolbox.rs           # 工具箱消息
│
├── update/                  # ═══ Update 层：状态变更逻辑 ═══
│   ├── mod.rs               # update(app, msg) -> 主分发
│   ├── navigation.rs        # 导航更新逻辑
│   ├── accounts.rs          # 账号页面更新
│   ├── domains.rs           # 域名页面更新
│   ├── dns.rs               # DNS 记录更新
│   ├── toolbox.rs           # 工具箱更新
│   └── settings.rs          # 设置更新
│
├── view/                    # ═══ View 层：纯 UI 渲染 ═══
│   ├── mod.rs               # view(app, frame) -> 主渲染入口
│   ├── layout.rs            # 主布局结构
│   ├── theme.rs             # 主题和样式定义
│   │
│   ├── components/          # 可复用 UI 组件（纯函数）
│   │   ├── mod.rs
│   │   ├── navigation.rs    # 左侧导航面板
│   │   ├── statusbar.rs     # 底部状态栏
│   │   ├── table.rs         # 通用表格组件
│   │   ├── tree.rs          # 树形列表组件
│   │   ├── dialog.rs        # 对话框组件
│   │   ├── input.rs         # 输入框组件
│   │   └── tabs.rs          # 标签页组件
│   │
│   └── pages/               # 页面视图（组合组件）
│       ├── mod.rs
│       ├── home.rs          # 首页视图
│       ├── accounts.rs      # 账号页面视图
│       ├── domains.rs       # 域名选择视图
│       ├── dns_records.rs   # DNS 记录视图
│       ├── toolbox.rs       # 工具箱视图
│       └── settings.rs      # 设置页面视图
│
├── event/                   # ═══ Event 层：输入事件处理 ═══
│   ├── mod.rs
│   ├── handler.rs           # 事件 -> Message 映射
│   └── keymap.rs            # 快捷键配置
│
├── backend/                 # ═══ Backend 层：业务服务 ═══
│   ├── mod.rs               # （与 UI 完全解耦）
│   ├── account_service.rs   # 账号服务
│   ├── domain_service.rs    # 域名服务
│   ├── dns_service.rs       # DNS 记录服务
│   ├── toolbox_service.rs   # 工具箱服务
│   └── config_service.rs    # 配置服务
│
└── util/                    # ═══ 工具层 ═══
    ├── mod.rs
    └── terminal.rs          # 终端初始化/清理
```

### 1.3 架构分层职责

```
┌────────────────────────────────────────────────────────────────┐
│                          main.rs                               │
│  · 初始化终端                                                   │
│  · 创建 App 实例                                               │
│  · 调用 app::run()                                             │
│  · 清理退出                                                    │
└────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────┐
│                           app.rs                               │
│  主循环：                                                       │
│  loop {                                                        │
│      terminal.draw(|f| view::render(&app, f));  // 渲染       │
│      let event = event::poll();                 // 获取输入    │
│      let msg = event::handle(event, &app);      // 转换消息    │
│      update::update(&mut app, msg);             // 更新状态    │
│  }                                                             │
└────────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    View      │    │    Event     │    │    Update    │
│  (纯渲染)    │    │ (输入映射)   │    │  (状态逻辑)  │
│              │    │              │    │              │
│ · 读取 Model │    │ · 读取按键   │    │ · 修改 Model │
│ · 返回 UI    │    │ · 返回 Msg   │    │ · 调用 Backend│
│ · 无副作用   │    │ · 无副作用   │    │              │
└──────────────┘    └──────────────┘    └──────┬───────┘
                                               │
                                               ▼
                                    ┌──────────────────┐
                                    │     Backend      │
                                    │   (业务服务)     │
                                    │                  │
                                    │ · API 调用       │
                                    │ · 数据持久化     │
                                    │ · 与 core 对接   │
                                    └──────────────────┘
```

### 1.4 关键设计原则

**1. View 层原则**
```rust
// ✅ 正确：纯函数，只读取状态，返回 UI
fn render_accounts(state: &AccountsState, area: Rect, buf: &mut Buffer) {
    // 根据 state 渲染 UI，无任何副作用
}

// ❌ 错误：在 View 中修改状态或调用服务
fn render_accounts(state: &mut AccountsState, ...) {
    state.selected += 1;  // 不应该在这里修改
    backend::fetch_accounts();  // 不应该在这里调用
}
```

**2. Update 层原则**
```rust
// 所有状态变更都通过 Message 触发
pub fn update(app: &mut App, msg: AppMessage) {
    match msg {
        AppMessage::Navigation(nav_msg) => {
            navigation::update(&mut app.navigation, nav_msg);
        }
        AppMessage::Accounts(acc_msg) => {
            accounts::update(&mut app.accounts_state, acc_msg);
        }
        // ...
    }
}
```

**3. Backend 层原则**
```rust
// Backend 完全不知道 UI 的存在
// 后期可直接替换为 dns-orchestrator-core 的调用

pub trait AccountService {
    async fn list_accounts(&self) -> Result<Vec<Account>>;
    async fn create_account(&self, req: CreateAccountRequest) -> Result<Account>;
    async fn delete_account(&self, id: &str) -> Result<()>;
}

// 当前实现：Mock 数据
pub struct MockAccountService;

// 未来实现：对接 core
pub struct CoreAccountService {
    core: dns_orchestrator_core::AccountService,
}
```

## 二、状态管理设计

### 2.1 核心状态结构

```rust
/// 应用主状态
pub struct App {
    /// 是否应该退出
    pub should_quit: bool,

    /// 当前焦点面板
    pub focus: FocusPanel,

    /// 导航状态
    pub navigation: NavigationState,

    /// 当前页面
    pub current_page: Page,

    /// 各页面的状态
    pub home_state: HomeState,
    pub domains_state: DomainsState,
    pub accounts_state: AccountsState,
    pub toolbox_state: ToolboxState,
    pub settings_state: SettingsState,

    /// 主题设置
    pub theme: Theme,

    /// 状态栏消息
    pub status_message: Option<String>,
}

/// 焦点面板
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPanel {
    /// 左侧导航面板
    Navigation,
    /// 右侧内容面板
    Content,
}

/// 页面枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    Home,
    Domains,
    DomainDetail { account_id: usize, domain_id: usize },
    DnsRecords { account_id: usize, domain_id: usize },
    Accounts,
    Toolbox,
    Settings,
}

/// 导航状态
pub struct NavigationState {
    /// 导航项列表
    pub items: Vec<NavItem>,
    /// 当前选中的索引
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub struct NavItem {
    pub id: NavItemId,
    pub label: String,
    pub icon: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NavItemId {
    Home,
    Domains,
    Accounts,
    Toolbox,
    Settings,
}
```

### 2.2 页面状态示例

```rust
/// Accounts 页面状态
pub struct AccountsState {
    /// 账号列表
    pub accounts: Vec<Account>,
    /// 当前选中的索引
    pub selected: usize,
    /// 是否处于多选模式
    pub multi_select_mode: bool,
    /// 多选的账号索引集合
    pub selected_indices: HashSet<usize>,
    /// 滚动偏移
    pub scroll_offset: usize,
}

/// Domains 页面状态
pub struct DomainsState {
    /// 账号树（账号 -> 域名列表）
    pub accounts: Vec<AccountNode>,
    /// 当前选中的路径
    pub selected_path: Vec<usize>,
    /// 展开的账号索引
    pub expanded_accounts: HashSet<usize>,
}

pub struct AccountNode {
    pub account: Account,
    pub domains: Vec<Domain>,
    pub is_loading: bool,
}
```

## 三、键盘交互设计

### 3.1 全局快捷键

| 按键           | 功能               | 说明                          |
|----------------|--------------------|-----------------------------|
| `←` `→`        | 切换面板焦点        | 在导航面板和内容面板间切换      |
| `q`            | 退出/返回          | 在根页面退出，子页面返回上一级  |
| `Esc`          | 取消/返回          | 取消当前操作或返回上一级       |
| `Ctrl+C`       | 强制退出           | 立即退出程序                  |
| `Alt+h` / `?`  | 显示帮助           | 弹出帮助对话框                |
| `Alt+r`        | 刷新               | 刷新当前页面数据              |

### 3.2 导航面板快捷键（焦点在 Navigation）

| 按键       | 功能           |
|-----------|----------------|
| `↑` / `k` | 向上选择       |
| `↓` / `j` | 向下选择       |
| `Enter`   | 进入选中页面   |
| `Home`    | 跳到第一项     |
| `End`     | 跳到最后一项   |

### 3.3 内容面板快捷键（焦点在 Content）

#### 通用操作

| 按键               | 功能           |
|-------------------|----------------|
| `↑` / `k`         | 向上移动       |
| `↓` / `j`         | 向下移动       |
| `Tab`             | 下一个可聚焦项 |
| `Shift+Tab`       | 上一个可聚焦项 |
| `PageUp`          | 向上翻页       |
| `PageDown`        | 向下翻页       |
| `Home`            | 跳到顶部       |
| `End`             | 跳到底部       |
| `Enter`           | 确认/进入      |

#### Domains 页面

| 按键          | 功能                       |
|--------------|---------------------------|
| `Enter`      | 展开/折叠账号，或进入域名   |
| `→` / `l`    | 展开账号                   |
| `←` / `h`    | 折叠账号                   |
| `/`          | 搜索域名                   |
| `Esc`        | 返回域名列表（从DNS记录）   |

#### Accounts 页面

| 按键       | 功能           |
|-----------|----------------|
| `Alt+a`   | 添加账号       |
| `Alt+e`   | 编辑选中账号   |
| `Alt+d`   | 删除选中账号   |
| `Alt+i`   | 导入账号       |
| `Alt+x`   | 导出账号       |
| `Space`   | 切换选择模式   |
| `Ctrl+a`  | 全选/取消全选  |

#### DNS Records 页面

| 按键       | 功能           |
|-----------|----------------|
| `Alt+a`   | 添加记录       |
| `Alt+e`   | 编辑选中记录   |
| `Alt+d`   | 删除选中记录   |
| `/`       | 搜索记录       |
| `f`       | 过滤记录类型   |
| `s`       | 排序选项       |

#### Toolbox 页面

| 按键           | 功能                   |
|---------------|------------------------|
| `Tab`         | 切换工具标签            |
| `Shift+Tab`   | 反向切换工具标签        |
| `Enter`       | 执行查询               |
| `Ctrl+h`      | 显示历史记录           |

#### Settings 页面

| 按键           | 功能           |
|---------------|----------------|
| `↑` / `↓`     | 选择配置项     |
| `Enter`       | 编辑配置项     |
| `Space`       | 切换布尔值     |

## 四、UI 组件设计

### 4.1 导航面板组件

```rust
// 特性：
// - 高亮显示当前选中项
// - 支持图标（使用 Unicode/Emoji）
// - 支持键盘导航
// - 焦点状态可视化（边框高亮）

[焦点状态]
┌─────────────┐
│ ● Home      │  ← 选中项（高亮）
│   Domains   │
│   Accounts  │
│   Toolbox   │
│   Settings  │
└─────────────┘
```

### 4.2 状态栏组件

```rust
// 动态显示当前焦点的快捷键提示

[Navigation 焦点时]
│ ↑↓ 导航 │ Enter 进入 │ ← 切换到内容 │ q 退出 │

[Content 焦点时 - Accounts 页面]
│ Alt+a 添加 │ Alt+e 编辑 │ Alt+d 删除 │ → 切换到导航 │ q 返回 │

[Content 焦点时 - DNS Records 页面]
│ Alt+a 添加记录 │ / 搜索 │ f 过滤 │ Esc 返回 │
```

### 4.3 表格组件

```rust
// 用于显示账号列表、DNS记录等

┌────┬──────────┬──────────┬──────────┬─────────┐
│ ●  │   名称   │  服务商  │   状态   │ 创建时间 │
├────┼──────────┼──────────┼──────────┼─────────┤
│ ▶  │ Prod CF  │ Cloudflr │  ● 正常  │ 2024-01 │
│    │ Test Ali │ Aliyun   │  ● 正常  │ 2024-02 │
│    │ Dev Pod  │ DNSPod   │  ○ 错误  │ 2024-03 │
└────┴──────────┴──────────┴──────────┴─────────┘
     ↑
   选中标记
```

### 4.4 树形组件

```rust
// 用于 Domains 页面的账号-域名树

┌─────────────────────────────────┐
│ ▼ Prod CloudFlare (12 domains)  │  ← 展开的账号
│   ● example.com                  │  ← 选中的域名
│     test.com                     │
│     demo.org                     │
│ ▶ Test Aliyun (5 domains)       │  ← 折叠的账号
│ ▶ Dev DNSPod (3 domains)         │
└─────────────────────────────────┘
```

### 4.5 对话框组件

```rust
// 用于添加/编辑/确认操作

         ┌───────────────────────────┐
         │    添加账号                │
         ├───────────────────────────┤
         │ 名称: [____________]      │
         │ 服务商: [选择 ▼]          │
         │ API Key: [____________]   │
         │ API Secret: [__________]  │
         │                           │
         │   [确定]     [取消]        │
         └───────────────────────────┘
```

## 五、主题设计

### 5.1 颜色方案

```rust
pub enum Theme {
    Light,
    Dark,
    // 可扩展：Custom(ColorScheme)
}

// 深色主题（默认）
Dark:
  - 背景: #1e1e1e
  - 前景: #d4d4d4
  - 边框: #3e3e3e
  - 高亮: #007acc
  - 选中: #264f78
  - 成功: #4ec9b0
  - 警告: #ce9178
  - 错误: #f48771

// 浅色主题
Light:
  - 背景: #ffffff
  - 前景: #333333
  - 边框: #cccccc
  - 高亮: #0066cc
  - 选中: #cce8ff
  - 成功: #22863a
  - 警告: #b08800
  - 错误: #d73a49
```

### 5.2 样式规范

```rust
// 边框样式
- 焦点面板: 双线边框/加粗边框
- 非焦点面板: 单线边框
- 对话框: 圆角边框（如果终端支持）

// 文本样式
- 标题: 加粗
- 选中项: 反色显示
- 禁用项: 灰色/斜体
- 链接/可点击: 下划线（可选）
```

## 六、性能优化

### 6.1 渲染优化

- 使用 `ratatui` 的智能 diff 算法，只重绘变化区域
- 大列表使用虚拟滚动（只渲染可见项）
- 防抖搜索输入（300ms）

### 6.2 数据加载优化

- 懒加载：域名列表按需加载
- 分页：DNS 记录分页显示
- 缓存：缓存已加载的数据，减少 API 调用

## 七、开发路线图

### Phase 1: 基础框架 ✅
- [x] 搭建 Ratatui + Crossterm 基础
- [x] 实现基本布局（左右分栏）
- [ ] 实现状态管理结构
- [ ] 实现键盘事件路由

### Phase 2: 导航和页面
- [ ] 实现左侧导航面板
- [ ] 实现 Home 页面（静态仪表板）
- [ ] 实现页面切换逻辑
- [ ] 实现状态栏和快捷键提示

### Phase 3: Accounts 模块
- [ ] 实现账号列表显示
- [ ] 实现添加账号（Mock 数据）
- [ ] 实现编辑/删除账号
- [ ] 集成 dns-orchestrator-core

### Phase 4: Domains 模块
- [ ] 实现账号-域名树形视图
- [ ] 实现域名选择和导航
- [ ] 实现 DNS 记录列表
- [ ] 实现 DNS 记录 CRUD

### Phase 5: Toolbox & Settings
- [ ] 实现 DNS 查询工具
- [ ] 实现 WHOIS 查询
- [ ] 实现 SSL 检查
- [ ] 实现 Settings 页面

### Phase 6: 完善和优化
- [ ] 主题切换
- [ ] 错误处理和友好提示
- [ ] 帮助系统
- [ ] 性能优化
- [ ] 测试和文档

## 八、技术决策

### 8.1 依赖选择

- **ratatui**: 成熟的 TUI 框架，积极维护
- **crossterm**: 跨平台终端控制，支持 Windows/Linux/macOS
- **tokio**: 异步运行时，支持后期网络请求
- **anyhow**: 简化错误处理

### 8.2 架构原则

- **关注点分离**: UI / 状态 / 业务逻辑 分离
- **可测试性**: 状态和逻辑独立，易于单元测试
- **可扩展性**: 模块化设计，易于添加新功能
- **可维护性**: 清晰的目录结构和命名规范

## 九、待定问题

1. **异步加载**: 如何在 TUI 中优雅地显示加载状态？
   - 方案: 使用 Spinner 组件 + 异步任务队列

2. **长时间操作**: 如何处理耗时的 API 调用？
   - 方案: 后台任务 + 进度条 + 可取消

3. **多语言支持**: 是否需要国际化？
   - 暂定: 先支持中文，后期可扩展

4. **配置持久化**: 配置文件存储位置？
   - 方案: 使用 `dirs` crate 获取标准配置目录

---

**文档版本**: v0.1
**最后更新**: 2025-01-XX
**维护者**: AptS-1547
