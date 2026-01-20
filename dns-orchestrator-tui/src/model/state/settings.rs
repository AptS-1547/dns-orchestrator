//! 设置页面状态

use crate::i18n::Language;

/// 主题枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    /// 获取所有主题选项
    pub fn all() -> &'static [Theme] {
        &[Theme::Dark, Theme::Light]
    }

    /// 获取下一个主题
    pub fn next(&self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    /// 获取上一个主题
    pub fn prev(&self) -> Theme {
        self.next() // 只有两个选项，prev 和 next 相同
    }
}

/// 分页模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationMode {
    #[default]
    InfiniteScroll,
    Traditional,
}

impl PaginationMode {
    /// 获取所有分页模式选项
    pub fn all() -> &'static [PaginationMode] {
        &[PaginationMode::InfiniteScroll, PaginationMode::Traditional]
    }

    /// 获取下一个分页模式
    pub fn next(&self) -> PaginationMode {
        match self {
            PaginationMode::InfiniteScroll => PaginationMode::Traditional,
            PaginationMode::Traditional => PaginationMode::InfiniteScroll,
        }
    }

    /// 获取上一个分页模式
    pub fn prev(&self) -> PaginationMode {
        self.next()
    }
}

/// 设置项枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    Theme,
    Language,
    PaginationMode,
}

impl SettingItem {
    /// 获取所有设置项
    pub fn all() -> &'static [SettingItem] {
        &[
            SettingItem::Theme,
            SettingItem::Language,
            SettingItem::PaginationMode,
        ]
    }

    /// 获取设置项的索引
    pub fn index(&self) -> usize {
        match self {
            SettingItem::Theme => 0,
            SettingItem::Language => 1,
            SettingItem::PaginationMode => 2,
        }
    }

    /// 从索引获取设置项
    pub fn from_index(index: usize) -> Option<SettingItem> {
        match index {
            0 => Some(SettingItem::Theme),
            1 => Some(SettingItem::Language),
            2 => Some(SettingItem::PaginationMode),
            _ => None,
        }
    }
}

/// 设置页面状态
#[derive(Debug)]
pub struct SettingsState {
    /// 当前选中的设置项索引
    pub selected_index: usize,
    /// 当前主题
    pub theme: Theme,
    /// 当前语言
    pub language: Language,
    /// 分页模式
    pub pagination_mode: PaginationMode,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            theme: Theme::default(),
            language: Language::default(),
            pagination_mode: PaginationMode::default(),
        }
    }
}

impl SettingsState {
    /// 创建新的设置状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取设置项数量
    pub fn item_count(&self) -> usize {
        SettingItem::all().len()
    }

    /// 选择上一个设置项
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.item_count() - 1;
        }
    }

    /// 选择下一个设置项
    pub fn select_next(&mut self) {
        if self.selected_index < self.item_count() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// 获取当前选中的设置项
    pub fn current_item(&self) -> Option<SettingItem> {
        SettingItem::from_index(self.selected_index)
    }

    /// 切换当前设置项到下一个值
    pub fn toggle_next(&mut self) {
        match self.current_item() {
            Some(SettingItem::Theme) => {
                self.theme = self.theme.next();
            }
            Some(SettingItem::Language) => {
                self.language = self.language.next();
                // 同步更新全局语言设置
                crate::i18n::set_language(self.language);
            }
            Some(SettingItem::PaginationMode) => {
                self.pagination_mode = self.pagination_mode.next();
            }
            None => {}
        }
    }

    /// 切换当前设置项到上一个值
    pub fn toggle_prev(&mut self) {
        match self.current_item() {
            Some(SettingItem::Theme) => {
                self.theme = self.theme.prev();
            }
            Some(SettingItem::Language) => {
                self.language = self.language.prev();
                // 同步更新全局语言设置
                crate::i18n::set_language(self.language);
            }
            Some(SettingItem::PaginationMode) => {
                self.pagination_mode = self.pagination_mode.prev();
            }
            None => {}
        }
    }
}