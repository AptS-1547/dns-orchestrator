//! 翻译键定义
//!
//! 定义所有翻译文本的结构体，提供编译期类型检查。
//!
//! ## 分类标准
//!
//! 1. **按 UI 组件位置分类**：文本归属于它出现的 UI 组件
//! 2. **弹窗内容归 `modal.*`**：所有弹窗（Modal）的内容都放在 modal 下
//! 3. **页面内容归对应页面**：如 `home.*`, `settings.*`
//! 4. **跨组件复用归 `common.*`**：多处使用的通用词汇
//! 5. **键盘提示归 `hints.*`**：按键名称和操作提示

/// 所有翻译文本的根结构
pub struct Translations {
    /// 通用文本（跨多处复用）
    pub common: CommonTexts,
    /// 键盘提示（按键名称 + 动作词）
    pub hints: HintTexts,
    /// 导航栏文本
    pub nav: NavTexts,
    /// 主页文本
    pub home: HomeTexts,
    /// 账号页面文本
    pub accounts: AccountsTexts,
    /// 域名页面文本
    pub domains: DomainsTexts,
    /// DNS 记录页面文本
    pub dns_records: DnsRecordsTexts,
    /// 工具箱页面文本（仅主页面，不含弹窗）
    pub toolbox: ToolboxTexts,
    /// 设置页面文本
    pub settings: SettingsTexts,
    /// 弹窗文本（所有弹窗的内容）
    pub modal: ModalTexts,
    /// 状态栏文本
    pub status_bar: StatusBarTexts,
    /// 帮助页面文本
    pub help: HelpTexts,
}

// ============================================================================
// 通用文本
// ============================================================================

/// 通用文本（跨多处复用的词汇）
pub struct CommonTexts {
    pub app_name: &'static str,
    // 操作动词
    pub add: &'static str,
    pub edit: &'static str,
    pub delete: &'static str,
    pub cancel: &'static str,
    pub save: &'static str,
    pub confirm: &'static str,
    pub close: &'static str,
    pub search: &'static str,
    pub query: &'static str,
    pub check: &'static str,
    pub lookup: &'static str,
    pub quit: &'static str,
    // 状态词
    pub loading: &'static str,
    pub no_data: &'static str,
    pub result: &'static str,
    pub error: &'static str,
    // 是/否
    pub yes: &'static str,
    pub no: &'static str,
    // 导航词
    pub back: &'static str,
    pub next: &'static str,
    pub prev: &'static str,
}

// ============================================================================
// 键盘提示
// ============================================================================

/// 键盘提示文本
pub struct HintTexts {
    /// 按键名称
    pub keys: KeyNames,
    /// 动作描述
    pub actions: ActionTexts,
}

/// 按键名称
pub struct KeyNames {
    pub enter: &'static str,
    pub esc: &'static str,
    pub tab: &'static str,
    pub arrows_lr: &'static str,  // "←→"
    pub arrows_ud: &'static str,  // "↑↓"
    pub tab_arrows: &'static str, // "Tab/↑↓"
}

/// 动作描述（用于组合提示）
pub struct ActionTexts {
    pub navigate: &'static str,      // "导航" / "Navigate"
    pub switch_option: &'static str, // "切换选项" / "Switch"
    pub switch_panel: &'static str,  // "切换面板" / "Switch panel"
    pub move_up_down: &'static str,  // "上下移动" / "Move"
    pub change_method: &'static str, // "切换方法" / "Change method"
    pub change_type: &'static str,   // "切换类型" / "Change type"
}

// ============================================================================
// 导航栏
// ============================================================================

/// 导航栏文本
pub struct NavTexts {
    pub home: &'static str,
    pub accounts: &'static str,
    pub domains: &'static str,
    pub toolbox: &'static str,
    pub settings: &'static str,
}

// ============================================================================
// 页面文本
// ============================================================================

/// 主页文本
pub struct HomeTexts {
    pub welcome: &'static str,
    pub welcome_desc: &'static str,
    pub quick_actions: &'static str,
    pub manage_domains: &'static str,
    pub use_tools: &'static str,
    pub manage_accounts: &'static str,
    pub configure_settings: &'static str,
}

/// 账号页面文本
pub struct AccountsTexts {
    pub title: &'static str,
    pub add_account: &'static str,
    pub edit_account: &'static str,
    pub delete_account: &'static str,
    pub no_accounts: &'static str,
    pub provider: &'static str,
    pub account_name: &'static str,
    pub account_name_optional: &'static str,
    pub delete_confirm: &'static str,
}

/// 域名页面文本
pub struct DomainsTexts {
    pub title: &'static str,
    pub no_domains: &'static str,
    pub record_count: &'static str,
    pub status_active: &'static str,
    pub status_paused: &'static str,
    pub status_pending: &'static str,
    pub status_error: &'static str,
}

/// DNS 记录页面文本
pub struct DnsRecordsTexts {
    pub title: &'static str,
    pub add_record: &'static str,
    pub edit_record: &'static str,
    pub delete_record: &'static str,
    pub no_records: &'static str,
    pub name: &'static str,
    pub value: &'static str,
    pub ttl: &'static str,
    pub proxy: &'static str,
}

/// 工具箱页面文本（仅主页面标签，弹窗内容在 modal.tools）
pub struct ToolboxTexts {
    pub title: &'static str,
    /// 工具标签名称
    pub tabs: ToolboxTabTexts,
}

pub struct ToolboxTabTexts {
    pub whois: &'static str,
    pub dns_lookup: &'static str,
    pub ip_lookup: &'static str,
    pub ssl_check: &'static str,
    pub http_headers: &'static str,
    pub dns_propagation: &'static str,
    pub dnssec_check: &'static str,
}

/// 设置页面文本
pub struct SettingsTexts {
    pub title: &'static str,
    /// 主题设置
    pub theme: ThemeTexts,
    /// 语言设置
    pub language: LanguageTexts,
    /// 分页设置
    pub pagination: PaginationTexts,
}

pub struct ThemeTexts {
    pub label: &'static str,
    pub dark: &'static str,
    pub light: &'static str,
    pub system: &'static str,
}

pub struct LanguageTexts {
    pub label: &'static str,
    pub description: &'static str,
}

pub struct PaginationTexts {
    pub label: &'static str,
    pub infinite_scroll: &'static str,
    pub traditional: &'static str,
}

// ============================================================================
// 弹窗文本
// ============================================================================

/// 弹窗文本（所有弹窗的内容都在这里）
pub struct ModalTexts {
    /// 添加账号弹窗
    pub add_account: AddAccountModalTexts,
    /// 确认删除弹窗
    pub confirm_delete: ConfirmDeleteTexts,
    /// 工具弹窗（DNS查询、WHOIS、SSL检查等）
    pub tools: ToolModalsTexts,
}

/// 添加账号弹窗
pub struct AddAccountModalTexts {
    pub title: &'static str,
    pub provider: &'static str,
    pub provider_hint: &'static str,
    pub account_name: &'static str,
    pub account_name_example: &'static str,
    pub main_account: &'static str,
    pub api_token: &'static str,
    pub api_token_hint: &'static str,
    pub accesskey_id: &'static str,
    pub accesskey_id_hint: &'static str,
    pub accesskey_secret: &'static str,
    pub accesskey_secret_hint: &'static str,
    pub secretid: &'static str,
    pub secretid_hint: &'static str,
    pub secretkey: &'static str,
    pub secretkey_hint: &'static str,
    pub access_key_id: &'static str,
    pub access_key_id_hint: &'static str,
    pub secret_access_key: &'static str,
    pub secret_access_key_hint: &'static str,
}

/// 确认删除弹窗
pub struct ConfirmDeleteTexts {
    pub title: &'static str,
    pub message: &'static str,
    pub confirm_button: &'static str,
    pub cancel_button: &'static str,
}

/// 工具弹窗文本（所有工具弹窗共用）
pub struct ToolModalsTexts {
    /// 弹窗标题
    pub titles: ToolModalTitles,
    /// 表单标签
    pub labels: ToolModalLabels,
    /// 输入框占位符
    pub placeholders: ToolModalPlaceholders,
    /// 状态文本
    pub status: ToolModalStatus,
    /// 结果标签
    pub result_label: &'static str,
}

/// 工具弹窗标题
pub struct ToolModalTitles {
    pub dns_lookup: &'static str,
    pub whois: &'static str,
    pub ssl_check: &'static str,
    pub ip_lookup: &'static str,
    pub http_header: &'static str,
    pub dns_propagation: &'static str,
    pub dnssec: &'static str,
}

/// 工具弹窗表单标签
pub struct ToolModalLabels {
    pub domain: &'static str,
    pub record_type: &'static str,
    pub dns_server: &'static str,
    pub url: &'static str,
    pub method: &'static str,
    pub ip_or_domain: &'static str,
}

/// 工具弹窗输入框占位符
pub struct ToolModalPlaceholders {
    pub enter_domain: &'static str,
    pub enter_ip: &'static str,
    pub enter_ip_or_domain: &'static str,
    pub enter_url: &'static str,
}

/// 工具弹窗状态文本
pub struct ToolModalStatus {
    pub querying: &'static str,
    pub checking: &'static str,
    pub checking_propagation: &'static str,
    pub checking_dnssec: &'static str,
    pub looking_up: &'static str,
}

// ============================================================================
// 其他组件
// ============================================================================

/// 状态栏文本
pub struct StatusBarTexts {
    pub ready: &'static str,
    pub loading: &'static str,
    pub error: &'static str,
}

/// 帮助页面文本
pub struct HelpTexts {
    pub title: &'static str,
    pub global_shortcuts: &'static str,
    pub operation_shortcuts: &'static str,
    pub close_hint: &'static str,
    /// 快捷键动作描述
    pub actions: HelpActionTexts,
}

/// 帮助页面快捷键动作描述
pub struct HelpActionTexts {
    pub switch_panel: &'static str,
    pub move_up_down: &'static str,
    pub confirm: &'static str,
    pub back_cancel: &'static str,
    pub quit: &'static str,
    pub add: &'static str,
    pub edit: &'static str,
    pub delete: &'static str,
}
