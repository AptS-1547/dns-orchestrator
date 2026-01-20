//! 配置服务

use anyhow::Result;

use crate::view::theme::Theme;

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub theme: Theme,
    pub language: String,
    pub pagination_mode: PaginationMode,
}

/// 分页模式
#[derive(Debug, Clone, Copy, Default)]
pub enum PaginationMode {
    #[default]
    Paginated,
    Infinite,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            language: "en-US".to_string(),
            pagination_mode: PaginationMode::default(),
        }
    }
}

/// 配置服务 trait
pub trait ConfigService: Send + Sync {
    /// 加载配置
    fn load(&self) -> Result<AppConfig>;

    /// 保存配置
    fn save(&self, config: &AppConfig) -> Result<()>;
}

/// 本地配置服务
pub struct LocalConfigService;

impl ConfigService for LocalConfigService {
    fn load(&self) -> Result<AppConfig> {
        // TODO: 从配置文件加载
        Ok(AppConfig::default())
    }

    fn save(&self, _config: &AppConfig) -> Result<()> {
        // TODO: 保存到配置文件
        Ok(())
    }
}