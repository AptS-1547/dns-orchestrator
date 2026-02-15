# DNS Orchestrator Provider 架构文档

本文档描述 `dns-orchestrator-provider` crate 的架构设计。这是一个独立的 Rust 库，为 DNS Orchestrator 项目提供统一的 DNS 提供商抽象层。

## 概述

**dns-orchestrator-provider** 是一个类型安全、异步优先的 Rust 库，为多个云服务商的 DNS 管理提供统一接口：

- **Cloudflare**
- **阿里云 DNS (Aliyun)**
- **腾讯云 DNSPod**
- **华为云 DNS**

### 关键数据

| 指标 | 数值 |
|------|------|
| 总代码行数 | ~5,500 行 |
| 源文件数量 | 33 个 .rs 文件 |
| 支持的提供商 | 4 个 |
| 支持的记录类型 | 8 种 (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA) |

## 目录结构

```
dns-orchestrator-provider/
├── Cargo.toml                  # 依赖和 feature flags 配置
├── rustfmt.toml                # 代码格式化配置
├── docs/
│   ├── ARCHITECTURE.md         # 架构文档（英文）
│   ├── ARCHITECTURE.zh-CN.md   # 架构文档（本文档）
│   ├── TESTING.md              # 集成测试指南（英文）
│   └── TESTING.zh-CN.md        # 集成测试指南（中文）
├── src/
│   ├── lib.rs                  # 库入口，公开 API 导出
│   ├── traits.rs               # DnsProvider trait 定义
│   ├── types.rs                # 公共类型定义
│   ├── error.rs                # 统一错误类型
│   ├── factory.rs              # Provider 工厂函数
│   ├── http_client.rs          # HTTP 工具函数
│   ├── utils/
│   │   ├── mod.rs
│   │   └── datetime.rs         # 时间序列化工具
│   └── providers/
│       ├── mod.rs              # Provider 模块注册
│       ├── common.rs           # 共享工具
│       ├── cloudflare/         # Cloudflare 实现
│       ├── aliyun/             # 阿里云实现
│       ├── dnspod/             # 腾讯云实现
│       └── huaweicloud/        # 华为云实现
└── tests/
    ├── common/mod.rs           # 测试工具库
    └── *_test.rs               # 各提供商集成测试
```

## 核心架构

### 架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                           消费者层                                   │
│                  (Tauri Commands / Actix-Web API)                   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                           工厂层                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  create_provider(credentials) -> Arc<dyn DnsProvider>        │   │
│  │  get_all_provider_metadata() -> Vec<ProviderMetadata>        │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     DnsProvider Trait 层                             │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  async trait DnsProvider: Send + Sync                        │   │
│  │    - validate_credentials()    验证凭证                       │   │
│  │    - list_domains() / get_domain()    域名管理                │   │
│  │    - list_records() / create_record()    记录管理             │   │
│  │    - update_record() / delete_record()                       │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
        ┌─────────────┬───────────┼───────────┬─────────────┐
        ▼             ▼           ▼           ▼             ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ Cloudflare  │ │   Aliyun    │ │   DNSPod    │ │ Huaweicloud │
│  Provider   │ │  Provider   │ │  Provider   │ │  Provider   │
├─────────────┤ ├─────────────┤ ├─────────────┤ ├─────────────┤
│ provider.rs │ │ provider.rs │ │ provider.rs │ │ provider.rs │
│ http.rs     │ │ http.rs     │ │ http.rs     │ │ http.rs     │
│ error.rs    │ │ error.rs    │ │ error.rs    │ │ error.rs    │
│ types.rs    │ │ types.rs    │ │ types.rs    │ │ types.rs    │
│             │ │ sign.rs     │ │ sign.rs     │ │ sign.rs     │
└──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
       │               │               │               │
       ▼               ▼               ▼               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       HTTP 客户端层                                  │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  HttpUtils::execute_request()           执行请求              │   │
│  │  HttpUtils::execute_request_with_retry() 带重试执行           │   │
│  │  HttpUtils::parse_json()                解析 JSON             │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       reqwest + TLS                                  │
│              (桌面端用 native-tls，Android 用 rustls)                │
└─────────────────────────────────────────────────────────────────────┘
```

### 各层职责

| 层级 | 职责 |
|------|------|
| **工厂层** | 根据凭证创建 Provider 实例，暴露元数据 |
| **Trait 层** | 定义所有 Provider 的统一异步接口 |
| **Provider 层** | 实现特定提供商的逻辑、API 调用、错误映射 |
| **HTTP 客户端层** | 处理 HTTP 请求、重试、JSON 解析 |
| **TLS 层** | 安全传输（根据平台选择后端） |

## 核心 Trait 和类型

### DnsProvider Trait

定义所有 DNS 提供商统一接口的核心 trait：

```rust
#[async_trait]
pub trait DnsProvider: Send + Sync {
    // 元数据
    fn id(&self) -> &'static str;
    fn metadata() -> ProviderMetadata where Self: Sized;

    // 凭证验证
    async fn validate_credentials(&self) -> Result<bool>;

    // 域名管理
    async fn list_domains(&self, params: &PaginationParams)
        -> Result<PaginatedResponse<ProviderDomain>>;
    async fn get_domain(&self, domain_id: &str) -> Result<ProviderDomain>;

    // 记录管理
    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams)
        -> Result<PaginatedResponse<DnsRecord>>;
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;
    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest)
        -> Result<DnsRecord>;
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;

    // 批量操作（占位实现）
    async fn batch_create_records(...) -> Result<BatchCreateResult>;
    async fn batch_update_records(...) -> Result<BatchUpdateResult>;
    async fn batch_delete_records(...) -> Result<BatchDeleteResult>;
}
```

### 类型安全的 DNS 记录

DNS 记录数据使用带标签的枚举表示，确保编译时类型安全：

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RecordData {
    A { address: String },
    AAAA { address: String },
    CNAME { target: String },
    MX { priority: u16, exchange: String },
    TXT { text: String },
    NS { nameserver: String },
    SRV { priority: u16, weight: u16, port: u16, target: String },
    CAA { flags: u8, tag: String, value: String },
}
```

### Provider 凭证

类型安全的凭证定义，使用 serde 标签：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "credentials")]
pub enum ProviderCredentials {
    Cloudflare { api_token: String },
    Aliyun { access_key_id: String, access_key_secret: String },
    Dnspod { secret_id: String, secret_key: String },
    Huaweicloud { access_key_id: String, secret_access_key: String },
}
```

### 统一错误类型

所有 Provider 返回标准化错误类型，便于前端统一处理：

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    NetworkError { provider: String, detail: String },           // 网络错误
    InvalidCredentials { provider: String, ... },                // 凭证无效
    RecordExists { provider: String, record_name: String, ... }, // 记录已存在
    RecordNotFound { provider: String, record_id: String, ... }, // 记录不存在
    InvalidParameter { provider: String, param: String, ... },   // 参数无效
    UnsupportedRecordType { provider: String, ... },             // 不支持的记录类型
    QuotaExceeded { provider: String, ... },                     // 配额超限
    DomainNotFound { provider: String, domain: String, ... },    // 域名不存在
    DomainLocked { provider: String, domain: String, ... },      // 域名已锁定
    PermissionDenied { provider: String, ... },                  // 权限不足
    ParseError { provider: String, detail: String },             // 解析错误
    SerializationError { provider: String, detail: String },     // 序列化错误
    Unknown { provider: String, raw_code: Option<String>, ... }, // 未知错误
}
```

## Provider 实现

### 标准模块结构

每个 Provider 遵循统一的六层结构：

```
provider_name/
├── mod.rs          # Provider 结构定义，Builder 模式
├── provider.rs     # DnsProvider trait 实现
├── http.rs         # HTTP 请求方法（API 调用）
├── error.rs        # ProviderErrorMapper 实现
├── sign.rs         # 签名算法（云服务商特有，Cloudflare 无此文件）
└── types.rs        # API 响应类型定义
```

### Provider 对比

| 提供商 | 代码行数 | 认证方式 | API 风格 | 单页最大记录数 |
|--------|----------|----------|----------|----------------|
| **Cloudflare** | ~900 | Bearer Token | RESTful | 域名: 50, 记录: 100 |
| **阿里云** | ~1000 | ACS3-HMAC-SHA256 | RPC (Query String) | 100 |
| **DNSPod** | ~1070 | TC3-HMAC-SHA256 | RPC (JSON Body) | 100 |
| **华为云** | ~1070 | V4 签名 | RESTful | 500 |

### 签名算法

| 提供商 | 算法 | Header/位置 |
|--------|------|-------------|
| Cloudflare | 无（Bearer Token） | `Authorization: Bearer {token}` |
| 阿里云 | ACS3-HMAC-SHA256 | `Authorization` header，含规范请求 |
| DNSPod | TC3-HMAC-SHA256 | `Authorization` header，含派生密钥 |
| 华为云 | AWS Signature V4 变体 | `Authorization` header，含规范请求 |

### 错误映射

每个 Provider 实现 `ProviderErrorMapper` trait，将原始 API 错误转换为标准化的 `ProviderError`：

```rust
pub(crate) trait ProviderErrorMapper {
    fn provider_name(&self) -> &'static str;
    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError;
}
```

错误码映射示例：

| 提供商 | 错误码 | 映射结果 |
|--------|--------|----------|
| Cloudflare | 9109, 10000 | `InvalidCredentials` |
| Cloudflare | 81057 | `RecordExists` |
| Cloudflare | 81044 | `RecordNotFound` |
| 阿里云 | InvalidAccessKeyId.NotFound | `InvalidCredentials` |
| DNSPod | AuthFailure | `InvalidCredentials` |
| 华为云 | DNS.0003 | `RecordNotFound` |

## HTTP 客户端层

### HttpUtils

为所有 Provider 提供集中的 HTTP 工具：

```rust
pub struct HttpUtils;

impl HttpUtils {
    /// 执行 HTTP 请求，返回 (状态码, 响应文本)
    pub async fn execute_request(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
    ) -> Result<(u16, String), ProviderError>;

    /// 解析 JSON 响应，带错误处理
    pub fn parse_json<T>(response_text: &str, provider_name: &str) -> Result<T, ProviderError>;

    /// 带指数退避重试的请求执行
    pub async fn execute_request_with_retry(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
        max_retries: u32,
    ) -> Result<(u16, String), ProviderError>;
}
```

### 重试策略

- **重试**: 仅网络错误
- **不重试**: 业务错误（凭证无效、记录已存在等）
- **退避**: 指数退避 (100ms, 200ms, 400ms, ... 最大 10s)

### 共享 HTTP 客户端

使用 `OnceLock` 在所有 Provider 间共享全局 `reqwest::Client`：

```rust
pub fn create_http_client() -> Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }).clone()
}
```

## Feature Flags

该 crate 使用 Cargo feature flags 进行条件编译：

```toml
[features]
default = ["rustls", "all-providers"]

# TLS 后端（二选一）
native-tls = ["reqwest/native-tls"]
rustls = ["reqwest/rustls-tls"]        # 默认

# Provider 选择
cloudflare = []
aliyun = []
dnspod = []
huaweicloud = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud"]
```

### 使用示例

```bash
# 默认（所有 providers，rustls）
cargo build -p dns-orchestrator-provider

# 使用 native-tls
cargo build -p dns-orchestrator-provider --no-default-features --features "native-tls,all-providers"

# 仅单个 provider
cargo build -p dns-orchestrator-provider --no-default-features --features "rustls,cloudflare"
```

## 工厂模式

### 创建 Provider

```rust
use dns_orchestrator_provider::{create_provider, ProviderCredentials};

// 从凭证枚举创建
let credentials = ProviderCredentials::Cloudflare {
    api_token: "your-api-token".to_string(),
};
let provider = create_provider(credentials)?;

// 使用 provider
let domains = provider.list_domains(&PaginationParams::default()).await?;
```

### Builder 模式

每个 Provider 支持通过 builder 进行可选配置：

```rust
let provider = CloudflareProvider::builder("api-token")
    .max_retries(3)
    .build();
```

### Provider 元数据

```rust
use dns_orchestrator_provider::get_all_provider_metadata;

let metadata = get_all_provider_metadata();
// 返回 Vec<ProviderMetadata>，包含：
// - id: Provider 类型标识
// - name: 显示名称
// - description: Provider 描述
// - required_fields: 凭证字段定义
// - features: 支持的功能（如 Cloudflare 的 CDN 代理）
// - limits: 分页限制
```

## 测试

### 集成测试框架

测试组织在 `tests/` 目录，使用共享的测试上下文：

```rust
pub struct TestContext {
    pub provider: Arc<dyn DnsProvider>,
    pub domain: String,
    pub domain_id: Option<String>,
}

impl TestContext {
    pub fn cloudflare() -> Option<Self>;
    pub fn aliyun() -> Option<Self>;
    pub fn dnspod() -> Option<Self>;
    pub fn huaweicloud() -> Option<Self>;
}
```

### 运行测试

```bash
# 设置环境变量
export TEST_DOMAIN=example.com
export CLOUDFLARE_API_TOKEN=xxx
export ALIYUN_ACCESS_KEY_ID=xxx
export ALIYUN_ACCESS_KEY_SECRET=xxx
# ... 其他 provider 凭证

# 运行所有测试
cargo test -p dns-orchestrator-provider

# 运行特定 provider 测试
cargo test -p dns-orchestrator-provider --test cloudflare_test
```

详细测试文档请参阅 [TESTING.zh-CN.md](./TESTING.zh-CN.md)。

## 设计原则

### 1. 类型安全

- `RecordData` 枚举确保 DNS 记录的编译时类型检查
- `ProviderCredentials` 枚举防止跨 Provider 的凭证误用
- 所有 API 返回强类型的 `Result<T, ProviderError>`

### 2. 统一接口

- 所有 Provider 使用单一 `DnsProvider` trait
- 一致的分页、搜索和错误处理
- 无论底层 Provider 差异如何，API 保持一致

### 3. 错误规范化

- 原始 API 错误映射为标准化的 `ProviderError` 变体
- 前端可以统一处理所有 Provider 的错误
- 保留原始错误信息用于调试

### 4. 性能

- 共享 HTTP 客户端，复用连接池
- 可配置的指数退避重试
- Feature flags 实现最小二进制体积

### 5. 平台灵活性

- TLS 后端可选，支持跨平台
- 默认使用 `rustls`（纯 Rust 实现，无系统依赖）
- 可选 `native-tls` 用于需要平台原生 TLS 的场景

## 待实现功能

以下功能目前为占位实现（`unimplemented!`）：

- **批量创建记录**: Cloudflare 支持同步批量，DNSPod 使用异步任务
- **批量更新记录**: 华为云有 `BatchUpdateRecordSetWithLine` API
- **批量删除记录**: 各 Provider 调研中

## 依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| `async-trait` | 0.1 | 异步 trait 支持 |
| `reqwest` | 0.12 | HTTP 客户端 |
| `serde` / `serde_json` | 1.0 | 序列化 |
| `tokio` | 1.0 | 异步运行时（重试延迟） |
| `hmac` / `sha2` | 0.12 / 0.10 | 加密签名 |
| `chrono` | 0.4 | 时间处理 |
| `thiserror` | 2.0 | 错误定义 |
| `urlencoding` | 2.1 | URL 编码 |

## 许可证

本 crate 是 DNS Orchestrator 项目的一部分。
