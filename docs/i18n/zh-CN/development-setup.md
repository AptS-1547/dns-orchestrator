# 开发指南

本指南将帮助您设置开发环境并理解代码库结构，以便为 DNS Orchestrator 做出贡献。

## 目录

- [前置要求](#前置要求)
- [快速开始](#快速开始)
- [项目结构](#项目结构)
- [开发工作流](#开发工作流)
- [添加新的 DNS 服务商](#添加新的-dns-服务商)
- [多平台构建](#多平台构建)
- [测试](#测试)
- [常见问题](#常见问题)

## 前置要求

### 必需工具

- **Node.js**: 22+（推荐使用 LTS 版本）
- **pnpm**: 10+（包管理器）
- **Rust**: 最新稳定版（通过 [rustup](https://rustup.rs/) 安装）
- **Git**: 用于版本控制

### 平台特定依赖

#### macOS
```bash
xcode-select --install
```

#### Windows
安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/zh-hans/downloads/)，选择 C++ 开发工具。

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  xdg-utils \
  build-essential \
  curl \
  wget
```

#### Android 开发
```bash
# 通过 Android Studio 或命令行安装 Android SDK 和 NDK
# 设置环境变量
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>

# 初始化 Tauri Android
pnpm tauri android init
```

其他发行版请参阅 [Tauri 前置要求](https://v2.tauri.app/start/prerequisites/)。

## 快速开始

### 克隆仓库

```bash
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator
```

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# Rust 依赖由 Cargo 管理，首次构建时会自动安装
```

### 启动开发服务器

```bash
# 桌面端：以开发模式启动 Tauri，支持热重载
pnpm tauri dev

# Android：启动 Android 开发模式
pnpm tauri android dev

# Web 模式：启动前端 HTTP 传输模式（需要 actix-web 后端）
pnpm dev:web
```

这将会：
1. 启动 Vite 开发服务器（React 前端）
2. 编译 Rust 后端
3. 启动应用窗口并启用热重载

### 生产构建

```bash
# 桌面端构建
pnpm tauri build

# Android 构建
pnpm tauri android build

# Web 前端构建
pnpm build:web
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 项目结构

```
dns-orchestrator/
├── src/                              # 前端 (React + TypeScript)
│   ├── components/                   # 按功能组织的 React 组件
│   │   ├── account/                  # 账号管理 UI
│   │   ├── dns/                      # DNS 记录管理
│   │   ├── domain/                   # 域名管理
│   │   ├── domains/                  # 域名选择页面
│   │   ├── home/                     # 首页仪表盘
│   │   ├── toolbox/                  # 网络工具箱
│   │   ├── settings/                 # 设置页面
│   │   ├── layout/                   # 布局组件
│   │   ├── navigation/               # 导航组件
│   │   ├── error/                    # 错误边界
│   │   └── ui/                       # 可复用 UI 组件
│   ├── services/                     # 服务层
│   │   ├── transport/                # 传输抽象
│   │   │   ├── types.ts              # ITransport、CommandMap
│   │   │   ├── tauri.transport.ts    # Tauri IPC 实现
│   │   │   └── http.transport.ts     # HTTP REST 实现
│   │   ├── account.service.ts
│   │   ├── dns.service.ts
│   │   ├── domain.service.ts
│   │   └── toolbox.service.ts
│   ├── stores/                       # Zustand 状态管理
│   ├── types/                        # TypeScript 类型定义
│   ├── i18n/                         # 国际化
│   ├── lib/                          # 工具函数
│   ├── constants/                    # 应用常量
│   └── hooks/                        # 自定义 React Hooks
│
├── dns-orchestrator-provider/        # 独立 Provider 库
│   ├── src/
│   │   ├── lib.rs                    # 库入口，re-exports
│   │   ├── traits.rs                 # DnsProvider trait
│   │   ├── types.rs                  # Domain、DnsRecord 等类型
│   │   ├── error.rs                  # ProviderError 枚举
│   │   ├── factory.rs                # create_provider()、元数据
│   │   └── providers/                # Provider 实现
│   │       ├── mod.rs
│   │       ├── cloudflare.rs
│   │       ├── aliyun.rs
│   │       ├── dnspod.rs
│   │       └── huaweicloud.rs
│   └── Cargo.toml
│
├── src-tauri/                        # Tauri 后端 (桌面/移动)
│   ├── src/
│   │   ├── commands/                 # Tauri 命令处理器
│   │   │   ├── mod.rs
│   │   │   ├── account.rs
│   │   │   ├── dns.rs
│   │   │   ├── domain.rs
│   │   │   ├── toolbox.rs
│   │   │   └── updater.rs
│   │   ├── providers/                # Provider 注册表
│   │   │   └── mod.rs                # ProviderRegistry + re-exports
│   │   ├── credentials/              # 凭证存储
│   │   │   ├── mod.rs
│   │   │   ├── keychain.rs           # 桌面端钥匙串
│   │   │   └── android.rs            # Android Stronghold
│   │   ├── storage/                  # 本地数据持久化
│   │   ├── crypto.rs                 # 加密工具
│   │   ├── error.rs                  # 错误类型
│   │   ├── types.rs                  # Rust 类型定义
│   │   ├── lib.rs                    # Tauri 库入口
│   │   └── main.rs                   # 应用入口
│   ├── tauri-plugin-apk-installer/   # Android APK 安装插件
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src-actix-web/                    # Web 后端 (开发中)
│   ├── src/main.rs                   # Actix-web 服务入口
│   ├── migration/                    # SeaORM 数据库迁移
│   └── Cargo.toml
│
├── scripts/
│   └── sync-version.mjs              # 版本同步脚本
├── package.json
├── vite.config.ts                    # 平台感知的 Vite 配置
└── tsconfig.json
```

### 关键架构组件

#### 前端
- **Services**: 通过 `ITransport` 接口抽象后端通信
- **Transport**: 桌面/移动端使用 Tauri IPC，Web 使用 HTTP
- **Stores**: Zustand stores 用于状态管理（每个功能域一个）
- **Components**: 按功能组织

#### 后端 (Tauri)
- **Commands**: Tauri 命令处理器，暴露给前端
- **Providers**: 从 `dns-orchestrator-provider` re-export + `ProviderRegistry`
- **Credentials**: 平台特定的安全存储

#### Provider 库
- **独立 crate**: 可在 Tauri 和 Web 后端复用
- **Feature flags**: 按需启用 Provider 和 TLS 后端
- **统一错误**: 所有 Provider 特定错误使用 `ProviderError`

## 开发工作流

### 热重载

- **前端更改**：即时重载，不丢失状态
- **后端更改**：需要手动重启 `pnpm tauri dev`
- **Provider 库更改**：需要重启

### 调试

#### 前端调试
在应用窗口中打开开发者工具：
- **macOS/Linux**: `Cmd+Option+I` 或 `Ctrl+Shift+I`
- **Windows**: `F12`

#### 后端调试
```bash
# 启用调试日志
RUST_LOG=debug pnpm tauri dev

# 更详细的日志
RUST_LOG=dns_orchestrator=trace pnpm tauri dev
```

### 版本同步

```bash
pnpm sync-version
```

这将更新以下文件中的版本号：
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src-actix-web/Cargo.toml`

创建发布前务必运行此命令。

### 代码质量

```bash
# 前端
pnpm lint          # 运行 Biome linter
pnpm format:fix    # 格式化代码

# 后端
pnpm lint:rust     # 运行 Clippy
pnpm format:rust   # 格式化 Rust 代码

# 所有检查
pnpm check
```

## 添加新的 DNS 服务商

自 v1.1.0 起，Provider 实现在独立的 `dns-orchestrator-provider` 库中。

### 步骤 1：创建 Provider 实现

创建 `dns-orchestrator-provider/src/providers/your_provider.rs`：

```rust
use async_trait::async_trait;
use reqwest::Client;

use crate::error::{ProviderError, Result};
use crate::traits::DnsProvider;
use crate::types::*;

pub struct YourProvider {
    client: Client,
    api_key: String,
}

impl YourProvider {
    pub fn new(credentials: ProviderCredentials) -> Result<Self> {
        let ProviderCredentials::YourProvider { api_key } = credentials else {
            return Err(ProviderError::InvalidCredentials {
                provider: "your_provider".to_string(),
            });
        };

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    fn provider_name() -> &'static str {
        "your_provider"
    }
}

#[async_trait]
impl DnsProvider for YourProvider {
    async fn validate_credentials(&self) -> Result<()> {
        // 进行一个简单的 API 调用来验证凭证
        todo!()
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        todo!()
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        todo!()
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        todo!()
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        todo!()
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        todo!()
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        todo!()
    }
}
```

### 步骤 2：添加 Feature Flag

更新 `dns-orchestrator-provider/Cargo.toml`：

```toml
[features]
your_provider = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud", "your_provider"]
```

### 步骤 3：注册 Provider

更新 `dns-orchestrator-provider/src/providers/mod.rs`：

```rust
#[cfg(feature = "your_provider")]
mod your_provider;
#[cfg(feature = "your_provider")]
pub use your_provider::YourProvider;
```

更新 `dns-orchestrator-provider/src/factory.rs`：

```rust
pub fn create_provider(credentials: ProviderCredentials) -> Result<Arc<dyn DnsProvider>> {
    match &credentials {
        // ... 现有 providers
        #[cfg(feature = "your_provider")]
        ProviderCredentials::YourProvider { .. } => {
            Ok(Arc::new(YourProvider::new(credentials)?))
        }
    }
}
```

### 步骤 4：添加凭证类型

更新 `dns-orchestrator-provider/src/types.rs`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderCredentials {
    // ... 现有变体
    YourProvider { api_key: String },
}

// 添加到 ProviderType 枚举
pub enum ProviderType {
    // ...
    YourProvider,
}
```

### 步骤 5：添加 Provider 元数据

更新 `dns-orchestrator-provider/src/factory.rs`：

```rust
pub fn get_all_provider_metadata() -> Vec<ProviderMetadata> {
    vec![
        // ... 现有 providers
        #[cfg(feature = "your_provider")]
        ProviderMetadata {
            id: "your_provider".to_string(),
            name: "你的服务商".to_string(),
            description: "你的 DNS 服务商描述".to_string(),
            required_fields: vec![
                ProviderCredentialField {
                    key: "api_key".to_string(),
                    label: "API Key".to_string(),
                    field_type: FieldType::Password,
                    placeholder: Some("输入 API Key".to_string()),
                    help_text: Some("从服务商控制台获取".to_string()),
                },
            ],
            features: ProviderFeatures::default(),
        },
    ]
}
```

### 步骤 6：添加前端翻译

**`src/i18n/locales/en-US.ts`：**
```typescript
providers: {
  your_provider: 'Your Provider',
}
```

**`src/i18n/locales/zh-CN.ts`：**
```typescript
providers: {
  your_provider: '你的服务商',
}
```

### 步骤 7：添加 Provider 图标（可选）

如果有自定义图标，更新 `src/components/account/ProviderIcon.tsx`。

### 步骤 8：测试

```bash
# 运行 Provider 库测试
cargo test -p dns-orchestrator-provider

# 启动开发服务器并测试 UI
pnpm tauri dev
```

## 多平台构建

### 桌面端 (macOS, Windows, Linux)

```bash
pnpm tauri build
```

### Android

```bash
# 初始化（仅首次）
pnpm tauri android init

# 开发模式
pnpm tauri android dev

# 发布构建
pnpm tauri android build
```

**注意**：Android 使用 `rustls` 而非 `native-tls`，以避免 OpenSSL 交叉编译问题。

### Web 模式

```bash
# 开发模式（需要运行 actix-web 后端）
pnpm dev:web

# 构建
pnpm build:web
```

### GitHub Actions 发布

推送标签以触发自动构建：

```bash
git tag v1.1.0
git push origin v1.1.0
```

**支持的平台：**
- macOS（Apple Silicon + Intel）
- Windows（x64 + ARM64）
- Linux（x64 + ARM64）
- Android（ARM64、ARM32、x64）

## 测试

### 运行测试

```bash
# Provider 库测试
cargo test -p dns-orchestrator-provider

# Tauri 后端测试
cargo test -p dns-orchestrator

# 所有 Rust 测试
cargo test --workspace
```

### 手动测试清单

发布前，手动测试：

- [ ] 所有服务商的账号创建
- [ ] 凭证验证（有效和无效凭证）
- [ ] 域名列表与分页
- [ ] DNS 记录 CRUD 操作
- [ ] 搜索和过滤功能
- [ ] 带加密的账号导入导出
- [ ] DNS 查询工具
- [ ] WHOIS 查询工具
- [ ] 主题切换
- [ ] 语言切换
- [ ] Android 构建和基本功能

## 常见问题

### 构建错误

**问题**：找不到 `webkit2gtk`（Linux）
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

**问题**：Android 上的 OpenSSL 错误
```bash
# 确保 Android target 使用 rustls feature
# 检查 src-tauri/Cargo.toml 中 Android target 是否有 default-features = false
```

**问题**：Rust 链接器错误
```bash
rustup update stable
cargo clean
```

**问题**：pnpm 安装失败
```bash
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### 运行时错误

**问题**："加载凭证失败"
- 确保系统钥匙串服务正在运行（Linux：`gnome-keyring` 或 `kwallet`）
- 在 Android 上，确保 Stronghold 已正确初始化

**问题**：服务商 API 错误
- 检查 API 凭证是否正确
- 启用调试日志：`RUST_LOG=debug pnpm tauri dev`

### 开发技巧

1. **使用 React DevTools**：检查 Zustand stores 和组件状态
2. **查看 Rust 日志**：后端错误在开发模式下会记录到控制台
3. **使用真实凭证测试**：尽可能使用测试/沙盒 API 密钥
4. **增量编译**：保持 `pnpm tauri dev` 运行以加快迭代速度
5. **遇到奇怪错误时清理构建**：`cargo clean && pnpm tauri dev`

## 获取帮助

- **文档**：[Tauri 文档](https://v2.tauri.app/)、[React 文档](https://react.dev/)
- **问题**：[GitHub Issues](https://github.com/AptS-1547/dns-orchestrator/issues)
- **讨论**：[GitHub Discussions](https://github.com/AptS-1547/dns-orchestrator/discussions)

---

祝编码愉快！🚀
