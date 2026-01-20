//! 域名页面状态

use crate::model::domain::{Domain, DomainStatus, ProviderType};

/// 域名页面状态
#[derive(Debug, Default)]
pub struct DomainsState {
    /// 域名列表
    pub domains: Vec<Domain>,
    /// 当前选中的索引
    pub selected: usize,
    /// 是否正在加载
    pub loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl DomainsState {
    /// 创建新的域名状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 选择上一项
    pub fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if !self.domains.is_empty() && self.selected < self.domains.len() - 1 {
            self.selected += 1;
        }
    }

    /// 选择第一项
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// 选择最后一项
    pub fn select_last(&mut self) {
        if !self.domains.is_empty() {
            self.selected = self.domains.len() - 1;
        }
    }

    /// 获取当前选中的域名
    pub fn selected_domain(&self) -> Option<&Domain> {
        self.domains.get(self.selected)
    }

    /// 设置域名列表
    pub fn set_domains(&mut self, domains: Vec<Domain>) {
        self.domains = domains;
        self.selected = 0;
        self.loading = false;
        self.error = None;
    }

    /// 添加模拟数据（开发测试用）
    pub fn load_mock_data(&mut self) {
        self.domains = vec![
            Domain {
                id: "dom_1".to_string(),
                name: "example.com".to_string(),
                account_id: "acc_1".to_string(),
                provider: ProviderType::Aliyun,
                status: DomainStatus::Active,
                record_count: Some(15),
            },
            Domain {
                id: "dom_2".to_string(),
                name: "test.org".to_string(),
                account_id: "acc_1".to_string(),
                provider: ProviderType::Aliyun,
                status: DomainStatus::Active,
                record_count: Some(8),
            },
            Domain {
                id: "dom_3".to_string(),
                name: "mysite.io".to_string(),
                account_id: "acc_2".to_string(),
                provider: ProviderType::Cloudflare,
                status: DomainStatus::Pending,
                record_count: Some(3),
            },
        ];
        self.loading = false;
    }
}
