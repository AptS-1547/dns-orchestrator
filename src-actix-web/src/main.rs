//! DNS Orchestrator Web 后端
//!
//! 基于 Actix-web 的 RESTful API 服务

mod config;
mod crypto;
mod entity;
mod error;
mod handlers;
mod state;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use sea_orm::{Database, DbErr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::AppConfig;
use crypto::CryptoManager;
use state::AppState;

/// 初始化数据库连接
async fn init_database(config: &config::DatabaseConfig) -> Result<sea_orm::DatabaseConnection, DbErr> {
    let mut opt = sea_orm::ConnectOptions::new(&config.url);
    opt.max_connections(config.max_connections);

    Database::connect(opt).await
}

/// 初始化加密管理器
fn init_crypto(config: &config::SecurityConfig) -> anyhow::Result<CryptoManager> {
    match &config.encryption_key {
        Some(key) => CryptoManager::from_hex_key(key).map_err(|e| anyhow::anyhow!("{e}")),
        None => {
            // 自动生成密钥
            let key = CryptoManager::generate_key();
            tracing::warn!(
                "未配置加密密钥，已自动生成。请将以下密钥添加到配置文件中:\n\
                [security]\n\
                encryption_key = \"{key}\""
            );
            CryptoManager::from_hex_key(&key).map_err(|e| anyhow::anyhow!("{e}"))
        }
    }
}

/// 初始化 Provider（从数据库加载账户）
async fn init_providers(state: &AppState) -> anyhow::Result<()> {
    use sea_orm::EntityTrait;

    let accounts = entity::Account::find().all(&state.db).await?;

    for account in accounts {
        // 解密凭证
        let credentials_json = state.crypto.decrypt(&account.encrypted_credentials)?;
        let credentials: dns_orchestrator_provider::ProviderCredentials =
            serde_json::from_str(&credentials_json)?;

        // 创建 Provider
        match dns_orchestrator_provider::create_provider(credentials) {
            Ok(provider) => {
                state.registry.register(account.id.clone(), provider).await;
                tracing::info!("已加载账户: {} ({})", account.name, account.provider_type);
            }
            Err(e) => {
                tracing::error!("加载账户 {} 失败: {}", account.name, e);
            }
        }
    }

    Ok(())
}

/// 配置路由
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/invoke", web::post().to(handlers::invoke::invoke_handler)),
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,dns_orchestrator_web=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("DNS Orchestrator Web 后端启动中...");

    // 加载配置
    let config = AppConfig::load()?;
    tracing::info!("配置加载完成");

    // 初始化数据库
    let db = init_database(&config.database).await?;
    tracing::info!("数据库连接成功: {}", config.database.url);

    // 运行数据库迁移
    migration::Migrator::up(&db, None).await?;
    tracing::info!("数据库迁移完成");

    // 初始化加密管理器
    let crypto = init_crypto(&config.security)?;

    // 创建应用状态
    let state = AppState::new(db, crypto);

    // 初始化 Provider
    init_providers(&state).await?;

    // 启动服务器
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let workers = if config.server.workers == 0 {
        num_cpus::get()
    } else {
        config.server.workers
    };

    tracing::info!("服务器启动于 http://{} (workers: {})", bind_addr, workers);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .workers(workers)
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
