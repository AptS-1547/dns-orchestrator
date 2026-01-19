//! 工具箱页面状态

/// 工具箱标签页
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolboxTab {
    #[default]
    Whois,
    DnsLookup,
    IpLookup,
    SslCheck,
    HttpHeaderCheck,
    DnsPropagation,
    DnssecCheck,
}

impl ToolboxTab {
    /// 获取标签页名称
    pub fn name(&self) -> &'static str {
        match self {
            ToolboxTab::Whois => "WHOIS",
            ToolboxTab::DnsLookup => "DNS Lookup",
            ToolboxTab::IpLookup => "IP Lookup",
            ToolboxTab::SslCheck => "SSL Check",
            ToolboxTab::HttpHeaderCheck => "HTTP Headers",
            ToolboxTab::DnsPropagation => "DNS Propagation",
            ToolboxTab::DnssecCheck => "DNSSEC Check",
        }
    }

    /// 获取所有标签页
    pub fn all() -> &'static [ToolboxTab] {
        &[
            ToolboxTab::Whois,
            ToolboxTab::DnsLookup,
            ToolboxTab::IpLookup,
            ToolboxTab::SslCheck,
            ToolboxTab::HttpHeaderCheck,
            ToolboxTab::DnsPropagation,
            ToolboxTab::DnssecCheck,
        ]
    }

    /// 切换到下一个标签页
    pub fn next(&self) -> ToolboxTab {
        match self {
            ToolboxTab::Whois            => ToolboxTab::DnsLookup,
            ToolboxTab::DnsLookup        => ToolboxTab::IpLookup,
            ToolboxTab::IpLookup         => ToolboxTab::SslCheck,
            ToolboxTab::SslCheck         => ToolboxTab::HttpHeaderCheck,
            ToolboxTab::HttpHeaderCheck  => ToolboxTab::DnsPropagation,
            ToolboxTab::DnsPropagation   => ToolboxTab::DnssecCheck,
            ToolboxTab::DnssecCheck      => ToolboxTab::Whois,
        }
    }

    /// 切换到上一个标签页
    pub fn prev(&self) -> ToolboxTab {
        match self {
            ToolboxTab::Whois           => ToolboxTab::DnssecCheck,
            ToolboxTab::DnsLookup       => ToolboxTab::Whois,
            ToolboxTab::IpLookup        => ToolboxTab::DnsLookup,
            ToolboxTab::SslCheck        => ToolboxTab::IpLookup,
            ToolboxTab::HttpHeaderCheck => ToolboxTab::SslCheck,
            ToolboxTab::DnsPropagation  => ToolboxTab::HttpHeaderCheck,
            ToolboxTab::DnssecCheck     => ToolboxTab::DnsPropagation,
        }
    }

    /// 获取标签页的索引
    pub fn index(&self) -> usize {
        match self {
            ToolboxTab::Whois           => 0,
            ToolboxTab::DnsLookup       => 1,
            ToolboxTab::IpLookup        => 2,
            ToolboxTab::SslCheck        => 3,
            ToolboxTab::HttpHeaderCheck => 4,
            ToolboxTab::DnsPropagation  => 5,
            ToolboxTab::DnssecCheck     => 6,
        }
    }
}

/// 工具箱页面状态
#[derive(Debug, Default)]
pub struct ToolboxState {
    /// 当前选中的标签页
    pub current_tab: ToolboxTab,
    /// 可见标签页的起始索引（用于滚动）
    pub visible_start: usize,
    /// 输入内容
    pub input: String,
    /// 是否正在执行
    pub loading: bool,
    /// 执行结果
    pub result: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 可见标签页数量
const VISIBLE_TAB_COUNT: usize = 5;

impl ToolboxState {
    /// 创建新的工具箱状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 切换到下一个标签页
    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
        self.adjust_visible_range();
        self.clear_result();
    }

    /// 切换到上一个标签页
    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.prev();
        self.adjust_visible_range();
        self.clear_result();
    }

    /// 调整可见范围，确保当前标签页在可见区域内
    fn adjust_visible_range(&mut self) {
        let current_index = self.current_tab.index();
        let total_tabs = ToolboxTab::all().len();

        // 如果当前标签在可见范围左侧，向左滚动
        if current_index < self.visible_start {
            self.visible_start = current_index;
        }
        // 如果当前标签在可见范围右侧，向右滚动
        else if current_index >= self.visible_start + VISIBLE_TAB_COUNT {
            self.visible_start = current_index.saturating_sub(VISIBLE_TAB_COUNT - 1);
        }

        // 确保 visible_start 不会导致显示超出范围
        let max_start = total_tabs.saturating_sub(VISIBLE_TAB_COUNT);
        if self.visible_start > max_start {
            self.visible_start = max_start;
        }
    }

    /// 获取可见标签页数量
    pub fn visible_tab_count() -> usize {
        VISIBLE_TAB_COUNT
    }

    /// 清除结果
    pub fn clear_result(&mut self) {
        self.result = None;
        self.error = None;
    }

    /// 设置输入内容
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    /// 获取当前工具的提示文本
    pub fn placeholder(&self) -> &'static str {
        match self.current_tab {
            ToolboxTab::Whois            => "Enter domain (e.g., example.com)",
            ToolboxTab::DnsLookup        => "Enter domain (e.g., example.com)",
            ToolboxTab::IpLookup         => "Enter IP or domain",
            ToolboxTab::SslCheck         => "Enter domain (e.g., example.com)",
            ToolboxTab::HttpHeaderCheck  => "Enter URL (e.g., https://example.com)",
            ToolboxTab::DnsPropagation   => "Enter domain (e.g., example.com)",
            ToolboxTab::DnssecCheck      => "Enter domain (e.g., example.com)",
        }
    }
}
