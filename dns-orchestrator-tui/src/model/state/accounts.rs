//! 账号页面状态

use crate::model::domain::{Account, ProviderType};

/// 账号页面状态
#[derive(Debug, Default)]
pub struct AccountsState {
    /// 账号列表
    pub accounts: Vec<Account>,
    /// 当前选中的索引
    pub selected: usize,
    /// 是否正在加载
    pub loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl AccountsState {
    /// 创建新的账号状态
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
        if !self.accounts.is_empty() && self.selected < self.accounts.len() - 1 {
            self.selected += 1;
        }
    }

    /// 选择第一项
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// 选择最后一项
    pub fn select_last(&mut self) {
        if !self.accounts.is_empty() {
            self.selected = self.accounts.len() - 1;
        }
    }

    /// 获取当前选中的账号
    pub fn selected_account(&self) -> Option<&Account> {
        self.accounts.get(self.selected)
    }

    /// 设置账号列表
    pub fn set_accounts(&mut self, accounts: Vec<Account>) {
        self.accounts = accounts;
        self.selected = 0;
        self.loading = false;
        self.error = None;
    }

    /// 添加模拟数据（开发测试用）
    pub fn load_mock_data(&mut self) {
        self.accounts = vec![
            Account {
                id: "acc_1".to_string(),
                name: "Aliyun Main".to_string(),
                provider: ProviderType::Aliyun,
                status: None,
                error: None,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Account {
                id: "acc_2".to_string(),
                name: "Cloudflare".to_string(),
                provider: ProviderType::Cloudflare,
                status: None,
                error: None,
                created_at: "2024-01-02".to_string(),
                updated_at: "2024-01-02".to_string(),
            },
            Account {
                id: "acc_3".to_string(),
                name: "DNSPod".to_string(),
                provider: ProviderType::Dnspod,
                status: None,
                error: None,
                created_at: "2024-01-03".to_string(),
                updated_at: "2024-01-03".to_string(),
            },
        ];
        self.loading = false;
    }
}
