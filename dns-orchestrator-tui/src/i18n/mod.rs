//! 国际化（i18n）模块
//!
//! 提供多语言支持，借鉴 DNS-Orchestrator 前端的翻译结构。
//! 使用纯 Rust 结构体方案，编译期类型检查，零运行时开销。

use std::sync::atomic::{AtomicUsize, Ordering};

mod en_us;
mod zh_cn;
pub mod keys;

pub use keys::*;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// 英语（美国）
    EnUs,
    /// 简体中文（中国）
    ZhCn,
}

impl Default for Language {
    fn default() -> Self {
        Language::EnUs
    }
}

impl Language {
    /// 获取所有支持的语言
    pub fn all() -> &'static [Language] {
        &[Language::EnUs, Language::ZhCn]
    }

    /// 获取语言的显示名称（使用该语言本身的文字）
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::EnUs => "English",
            Language::ZhCn => "简体中文",
        }
    }

    /// 获取语言代码（BCP 47 标准）
    pub fn code(&self) -> &'static str {
        match self {
            Language::EnUs => "en-US",
            Language::ZhCn => "zh-CN",
        }
    }

    /// 从语言代码解析
    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en-US" | "en" => Some(Language::EnUs),
            "zh-CN" | "zh" => Some(Language::ZhCn),
            _ => None,
        }
    }

    /// 获取下一个语言（用于循环切换）
    pub fn next(&self) -> Language {
        match self {
            Language::EnUs => Language::ZhCn,
            Language::ZhCn => Language::EnUs,
        }
    }

    /// 获取上一个语言（用于循环切换）
    pub fn prev(&self) -> Language {
        match self {
            Language::EnUs => Language::ZhCn,
            Language::ZhCn => Language::EnUs,
        }
    }
}

/// 当前语言索引（原子操作，线程安全）
static CURRENT_LANGUAGE: AtomicUsize = AtomicUsize::new(0); // 0 = EnUs

/// 获取当前语言的翻译
///
/// # Example
///
/// ```
/// use dns_orchestrator_tui::i18n::t;
///
/// let text = t().nav.home; // "Home" or "主页"
/// ```
pub fn t() -> &'static Translations {
    match CURRENT_LANGUAGE.load(Ordering::Relaxed) {
        1 => &zh_cn::TRANSLATIONS,
        _ => &en_us::TRANSLATIONS,
    }
}

/// 设置当前语言
///
/// # Example
///
/// ```
/// use dns_orchestrator_tui::i18n::{set_language, Language};
///
/// set_language(Language::ZhCn);
/// ```
pub fn set_language(lang: Language) {
    let index = match lang {
        Language::EnUs => 0,
        Language::ZhCn => 1,
    };
    CURRENT_LANGUAGE.store(index, Ordering::Relaxed);
}

/// 获取当前语言
pub fn current_language() -> Language {
    match CURRENT_LANGUAGE.load(Ordering::Relaxed) {
        1 => Language::ZhCn,
        _ => Language::EnUs,
    }
}
