//! 应用配置模块

use serde::Deserialize;
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 安全配置
    pub security: SecurityConfig,
}

/// 服务器配置
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 工作线程数（0 表示使用 CPU 核心数）
    #[serde(default)]
    pub workers: usize,
}

/// 数据库配置
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// 数据库 URL
    /// 支持: sqlite://path, mysql://user:pass@host/db, postgres://user:pass@host/db
    #[serde(default = "default_database_url")]
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// 安全配置
#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    /// 凭证加密密钥（32 字节 hex 编码，64 字符）
    /// 如果未设置，将自动生成并保存
    pub encryption_key: Option<String>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_database_url() -> String {
    "sqlite://data.db?mode=rwc".to_string()
}

fn default_max_connections() -> u32 {
    10
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                workers: 0,
            },
            database: DatabaseConfig {
                url: default_database_url(),
                max_connections: default_max_connections(),
            },
            security: SecurityConfig {
                encryption_key: None,
            },
        }
    }
}

impl AppConfig {
    /// 从配置文件加载
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 创建默认配置文件
            let config = Self::default();
            let content = toml::to_string_pretty(&DefaultConfigTemplate::from(&config))?;
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, content)?;
            tracing::info!("已创建默认配置文件: {}", config_path.display());
            Ok(config)
        }
    }

    /// 获取配置文件路径
    fn config_path() -> PathBuf {
        std::env::var("DNS_ORCHESTRATOR_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config.toml"))
    }
}

/// 用于生成配置文件模板的结构
#[derive(serde::Serialize)]
struct DefaultConfigTemplate {
    server: ServerConfig,
    database: DatabaseConfig,
    security: SecurityConfigTemplate,
}

#[derive(serde::Serialize)]
struct SecurityConfigTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    encryption_key: Option<String>,
}

impl From<&AppConfig> for DefaultConfigTemplate {
    fn from(config: &AppConfig) -> Self {
        Self {
            server: config.server.clone(),
            database: config.database.clone(),
            security: SecurityConfigTemplate {
                encryption_key: config.security.encryption_key.clone(),
            },
        }
    }
}

impl serde::Serialize for ServerConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ServerConfig", 3)?;
        s.serialize_field("host", &self.host)?;
        s.serialize_field("port", &self.port)?;
        s.serialize_field("workers", &self.workers)?;
        s.end()
    }
}

impl serde::Serialize for DatabaseConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("DatabaseConfig", 2)?;
        s.serialize_field("url", &self.url)?;
        s.serialize_field("max_connections", &self.max_connections)?;
        s.end()
    }
}
