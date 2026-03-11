//! 收藏页面状态

use crate::model::domain::Domain;

/// 收藏页面状态
#[derive(Debug, Default)]
pub struct FavoritesState {
    /// 已收藏的域名列表
    pub domains: Vec<Domain>,
    /// 当前选中的索引
    pub selected: usize,
    /// 是否正在加载
    pub loading: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl FavoritesState {
    /// 创建新的收藏状态
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

    /// 从完整域名列表中重建收藏列表
    pub fn rebuild(&mut self, all_domains: &[Domain]) {
        self.domains = all_domains
            .iter()
            .filter(|d| d.is_favorite)
            .cloned()
            .collect();
        if self.domains.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.domains.len() - 1);
        }
        self.loading = false;
        self.error = None;
    }
}
