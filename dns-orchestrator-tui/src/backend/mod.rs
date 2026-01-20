//！┌─────────────────────────────────────────────────────────────────────────────┐
//！│                              主循环 (app.rs)                               │
//！│                                                                            │
//！│  ┌────────────────────────────── UI 层 ───────────────────────────────┐   │
//！│  │                                                                     │   │
//！│  │   ┌─────────┐          ┌───────────┐          ┌──────────┐         │   │
//！│  │   │  Event  │ ───────▶ │  Message  │ ───────▶ │  Update  │         │   │
//！│  │   │   层    │   翻译    │    层     │   消费    │    层    │         │   │
//！│  │   └─────────┘          │           │          └────┬─────┘         │   │
//！│  │        ▲               │ AppMessage│               │ 修改          │   │
//！│  │        │               │ ModalMsg  │               ▼               │   │
//！│  │   ┌─────────┐          │ ContentMsg│          ┌──────────┐         │   │
//！│  │   │  View   │          │ NavMsg    │   ┌───── │  Model   │         │   │
//！│  │   │   层    │          └───────────┘   │      │    层    │         │   │
//！│  │   └────┬────┘ ◀──────── 读取 ──────────┘      └────┬─────┘         │   │
//！│  │        │                                           │               │   │
//！│  └────────│───────────────────────────────────────────│───────────────┘   │
//！│           │                                           │ 异步调用          │
//！│           ▼                                           ▼                   │
//！│      ┌─────────┐                                ┌──────────┐              │
//！│      │  终端   │                                │ Backend  │              │
//！│      │ (Util)  │                                │    层    │              │
//！│      └─────────┘                                └────┬─────┘              │
//！│                                                      │                    │
//！│                                                      ▼                    │
//！│                                           ┌───────────────────┐           │
//！│                                           │dns-orchestrator-  │           │
//！│                                           │      core         │           │
//！│                                           └───────────────────┘           │
//！└─────────────────────────────────────────────────────────────────────────────┘
//!
//!
//! src/backend/mod.rs
//! Backend 层：业务服务
//!
//! Backend 层与 UI 完全解耦，负责所有的业务逻辑。
//! 通过 dns-orchestrator-core 库实现真实的 DNS 管理功能。
//!
//!
//! 有模块结构：
//!     src/backend/mod.rs
//!         mod core_service;               // 核心服务入口
//!         mod credential_service;         // 凭证存储（keyring）
//!         mod account_repository;         // 账号持久化（JSON 文件）
//!         mod domain_metadata_repository; // 域名元数据存储（内存）
//!
//!         mod account_service;            // 账号服务（Mock，用于测试）
//!         mod config_service;             // 配置服务（Mock，用于测试）
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 一、核心服务（CoreService）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/backend/core_service.rs 中定义：
//!
//!         CoreService 是 Backend 层的入口，封装了 dns-orchestrator-core 库。
//!         它负责初始化各种仓库和服务，并提供给 UI 层使用。
//!
//!         创建流程：
//!             1. 创建 CredentialStore（凭证存储）
//!             2. 创建 AccountRepository（账号仓库）
//!             3. 创建 ProviderRegistry（服务商注册表）
//!             4. 创建 DomainMetadataRepository（域名元数据仓库）
//!             5. 组装 ServiceContext（服务上下文）
//!             6. 基于 ServiceContext 创建各种业务服务
//!
//!         提供的服务：
//!             - account_lifecycle()   账号生命周期管理
//!             - account_metadata()    账号元数据
//!             - domain()              域名管理
//!             - dns()                 DNS 记录管理
//!             - provider_metadata()   服务商元数据
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 二、凭证存储（KeyringCredentialStore）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/backend/credential_service.rs 中定义：
//!
//!         实现 dns-orchestrator-core 的 CredentialStore trait。
//!         使用系统 keyring（钥匙串）安全存储 API 凭证。
//!
//!         主要方法：
//!             - get(account_id)           获取指定账号的凭证
//!             - set(account_id, creds)    保存账号凭证
//!             - remove(account_id)        删除账号凭证
//!             - load_all()                加载所有凭证
//!             - save_all(credentials)     保存所有凭证
//!
//!         安全性：
//!             - 凭证加密存储在系统 keyring 中
//!             - 不会以明文形式保存到文件
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 三、账号仓库（JsonAccountRepository）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/backend/account_repository.rs 中定义：
//!
//!         实现 dns-orchestrator-core 的 AccountRepository trait。
//!         将账号信息持久化到 JSON 文件。
//!
//!         存储位置：~/.config/dns-orchestrator/accounts.json
//!
//!         主要方法：
//!             - list()                    列出所有账号
//!             - find(account_id)          查找指定账号
//!             - save(account)             保存账号
//!             - delete(account_id)        删除账号
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 四、域名元数据仓库（InMemoryDomainMetadataRepository）
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     在 src/backend/domain_metadata_repository.rs 中定义：
//!
//!         实现 dns-orchestrator-core 的 DomainMetadataRepository trait。
//!         用于存储域名的附加信息（收藏、标签、颜色、备注）。
//!
//!         当前实现：内存存储（程序重启后数据丢失）
//!         TODO: 后续可改为持久化存储
//!
//!
//! ═══════════════════════════════════════════════════════════════════════════
//! 五、数据流
//! ═══════════════════════════════════════════════════════════════════════════
//!
//!     用户在弹窗中点击"确认"
//!         ↓
//!     Update 层处理 ModalMessage::Confirm
//!         ↓
//!     调用 CoreService 的相应方法（异步）
//!         ↓
//!     CoreService 调用 dns-orchestrator-core 的服务
//!         ↓
//!     dns-orchestrator-core 调用实际的 API（Cloudflare、阿里云等）
//!         ↓
//!     返回结果
//!         ↓
//!     Update 层更新 Model 状态
//!         ↓
//!     View 层重新渲染
//!

mod account_repository;
mod account_service;
mod config_service;
mod core_service;
mod credential_service;
mod domain_metadata_repository;

// 旧的 Mock 服务（保留用于测试）
pub use account_service::{AccountService, MockAccountService};
pub use config_service::{ConfigService, LocalConfigService};

// 新的核心服务
pub use account_repository::JsonAccountRepository;
pub use core_service::CoreService;
pub use credential_service::KeyringCredentialStore;
pub use domain_metadata_repository::InMemoryDomainMetadataRepository;